use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

/// The zstd magic number (little endian: [0x28, 0xB5, 0x2F, 0xFD])
const ZSTD_MAGIC: [u8; 4] = [0x28, 0xB5, 0x2F, 0xFD];

/// Creates a reader that automatically handles zstd compression.
/// Takes a closure that provides the raw reader, allowing reuse for both files and stdin.
fn create_reader_with_compression<F, R>(reader_factory: F) -> io::Result<Box<dyn Read>>
where
    F: Fn() -> io::Result<R>,
    R: Read + 'static,
{
    // First, get a reader to peek at the magic bytes
    let mut peek_reader = reader_factory()?;
    let mut magic = [0u8; 4];

    match peek_reader.read_exact(&mut magic) {
        Ok(()) => {
            if magic == ZSTD_MAGIC {
                // It's zstd compressed, create a fresh reader and wrap with decoder
                let fresh_reader = reader_factory()?;
                let decoder = zstd::stream::Decoder::new(fresh_reader)?;
                Ok(Box::new(decoder))
            } else {
                // Not zstd, create a fresh reader and prepend the magic bytes we consumed
                let fresh_reader = reader_factory()?;
                let reader = std::io::Cursor::new(magic).chain(fresh_reader);
                Ok(Box::new(reader))
            }
        }
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
            // Empty or too small, treat as uncompressed
            let fresh_reader = reader_factory()?;
            Ok(Box::new(fresh_reader))
        }
        Err(e) => Err(e),
    }
}

/// Creates a reader for a file path that automatically handles zstd compression.
fn create_file_reader<P: AsRef<Path>>(path: P) -> io::Result<Box<dyn Read>> {
    let path = path.as_ref().to_path_buf();
    create_reader_with_compression(move || File::open(&path))
}

/// Creates a reader for stdin that automatically handles zstd compression.
/// Since stdin can't be rewound, we handle the magic byte detection differently.
fn create_stdin_reader() -> io::Result<Box<dyn Read>> {
    let stdin = io::stdin();
    let mut magic = [0u8; 4];

    match stdin.lock().read_exact(&mut magic) {
        Ok(()) => {
            if magic == ZSTD_MAGIC {
                // It's zstd compressed, prepend the magic bytes and wrap with decoder
                let reader = std::io::Cursor::new(magic).chain(stdin);
                let decoder = zstd::stream::Decoder::new(reader)?;
                Ok(Box::new(decoder))
            } else {
                // Not zstd, prepend the magic bytes we consumed
                let reader = std::io::Cursor::new(magic).chain(stdin);
                Ok(Box::new(reader))
            }
        }
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
            // Empty stdin, return stdin directly
            Ok(Box::new(stdin))
        }
        Err(e) => Err(e),
    }
}

/// Reads binary data of 16-byte hit masks from any reader, filters records by hit/miss masks,
/// and accumulates counts of hits per cell (81 cells).
/// Automatically handles delta decoding if the data is delta-encoded.
pub fn filter_and_count_reader<R: Read>(
    mut reader: R,
    hit_mask: u128,
    miss_mask: u128,
) -> io::Result<(Vec<u32>, u64)> {
    let mut buf = [0u8; 16];
    let mut counts = vec![0u32; 81];
    let mut total_matched: u64 = 0;
    let mut prev_record = [0u8; 16];
    let mut first_record = true;

    loop {
        // Read one record (16 bytes)
        match reader.read_exact(&mut buf) {
            Ok(()) => {},
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        }

        // Handle delta decoding
        let current_record = if first_record {
            // First record is stored as-is
            first_record = false;
            buf
        } else {
            // XOR with previous record to get the actual value
            let mut decoded = [0u8; 16];
            for i in 0..16 {
                decoded[i] = buf[i] ^ prev_record[i];
            }
            decoded
        };

        // Parse u128 in little endian
        let raw = u128::from_le_bytes(current_record);

        // Filter
        if (raw & hit_mask) != hit_mask { continue; }
        if (raw & miss_mask) != 0 { continue; }

        // Count bits (using byte-by-byte approach to match original bit ordering)
        total_matched += 1;
        for bit in 0..81 {
            let byte_index = bit / 8;
            let bit_index = bit % 8;
            if (current_record[byte_index] >> bit_index) & 1 == 1 {
                counts[bit] += 1;
            }
        }

        // Update previous record for next iteration
        prev_record.copy_from_slice(&current_record);
    }
    Ok((counts, total_matched))
}

/// Reads binary data of 16-byte hit masks from any reader (raw format, no delta decoding),
/// filters records by hit/miss masks, and accumulates counts of hits per cell (81 cells).
pub fn filter_and_count_reader_raw<R: Read>(
    mut reader: R,
    hit_mask: u128,
    miss_mask: u128,
) -> io::Result<(Vec<u32>, u64)> {
    let mut buf = [0u8; 16];
    let mut counts = vec![0u32; 81];
    let mut total_matched: u64 = 0;

    loop {
        // Read one record (16 bytes)
        match reader.read_exact(&mut buf) {
            Ok(()) => {},
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        }

        // Parse u128 in little endian
        let raw = u128::from_le_bytes(buf);

        // Filter
        if (raw & hit_mask) != hit_mask { continue; }
        if (raw & miss_mask) != 0 { continue; }

        // Count bits (using byte-by-byte approach to match original bit ordering)
        total_matched += 1;
        for bit in 0..81 {
            let byte_index = bit / 8;
            let bit_index = bit % 8;
            if (buf[byte_index] >> bit_index) & 1 == 1 {
                counts[bit] += 1;
            }
        }
    }
    Ok((counts, total_matched))
}

/// Reads a binary file of 16-byte hit masks, filters records by hit/miss masks,
/// and accumulates counts of hits per cell (81 cells).
/// Supports both uncompressed and zstd-compressed files.
/// Pass "-" as the path to read from stdin.
/// By default assumes delta-encoded format.
pub fn filter_and_count<P: AsRef<Path>>(
    path: P,
    hit_mask: u128,
    miss_mask: u128,
) -> io::Result<(Vec<u32>, u64)> {
    filter_and_count_with_format(path, hit_mask, miss_mask, true)
}

/// Reads a binary file of 16-byte hit masks with explicit format specification.
/// Supports both uncompressed and zstd-compressed files.
/// Pass "-" as the path to read from stdin.
pub fn filter_and_count_with_format<P: AsRef<Path>>(
    path: P,
    hit_mask: u128,
    miss_mask: u128,
    is_delta_encoded: bool,
) -> io::Result<(Vec<u32>, u64)> {
    let path_str = path.as_ref().to_string_lossy();

    let reader = if path_str == "-" {
        create_stdin_reader()?
    } else {
        create_file_reader(path)?
    };

    if is_delta_encoded {
        filter_and_count_reader(reader, hit_mask, miss_mask)
    } else {
        filter_and_count_reader_raw(reader, hit_mask, miss_mask)
    }
}

/// C-compatible FFI export for filter_and_count.
///
/// The 128-bit masks are passed as two 64-bit values each (high and low parts).
///
/// # Safety
/// `out_counts` must point to a buffer of at least 81 u32 entries.
#[no_mangle]
pub unsafe extern "C" fn filter_and_count_ffi(
    path_ptr: *const std::os::raw::c_char,
    hit_mask_low: u64,
    hit_mask_high: u64,
    miss_mask_low: u64,
    miss_mask_high: u64,
    out_counts: *mut u32,
) -> u64 {
    use std::ffi::CStr;
    let cstr = CStr::from_ptr(path_ptr);
    let path = match cstr.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    // Reconstruct u128 values from high and low parts
    let hit_mask = ((hit_mask_high as u128) << 64) | (hit_mask_low as u128);
    let miss_mask = ((miss_mask_high as u128) << 64) | (miss_mask_low as u128);

    match filter_and_count(path, hit_mask, miss_mask) {
        Ok((counts, matched)) => {
            let slice = std::slice::from_raw_parts_mut(out_counts, 81);
            slice.copy_from_slice(&counts[..]);
            matched
        }
        Err(_) => 0,
    }
}

/// C-compatible FFI export for filter_and_count with format specification.
///
/// The 128-bit masks are passed as two 64-bit values each (high and low parts).
/// Set `is_delta_encoded` to 1 for delta-encoded format, 0 for raw format.
///
/// # Safety
/// `out_counts` must point to a buffer of at least 81 u32 entries.
#[no_mangle]
pub unsafe extern "C" fn filter_and_count_with_format_ffi(
    path_ptr: *const std::os::raw::c_char,
    hit_mask_low: u64,
    hit_mask_high: u64,
    miss_mask_low: u64,
    miss_mask_high: u64,
    is_delta_encoded: u8,
    out_counts: *mut u32,
) -> u64 {
    use std::ffi::CStr;
    let cstr = CStr::from_ptr(path_ptr);
    let path = match cstr.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    // Reconstruct u128 values from high and low parts
    let hit_mask = ((hit_mask_high as u128) << 64) | (hit_mask_low as u128);
    let miss_mask = ((miss_mask_high as u128) << 64) | (miss_mask_low as u128);

    match filter_and_count_with_format(path, hit_mask, miss_mask, is_delta_encoded != 0) {
        Ok((counts, matched)) => {
            let slice = std::slice::from_raw_parts_mut(out_counts, 81);
            slice.copy_from_slice(&counts[..]);
            matched
        }
        Err(_) => 0,
    }
}

/// Expected counts for all boards with no filtering (hit_mask=0, miss_mask=0)
/// This represents the heatmap of ship placement frequency across all valid boards
pub const EXPECTED_ALL_BOARDS_COUNTS: [u32; 81] = [
    91828984, 81901859, 117097056, 93138304, 90403381, 93138304, 117097056, 81901859, 91828984,
    81901859, 29572998, 54989301, 27344104, 37308200, 27344104, 54989301, 29572998, 81901859,
    117097056, 54989301, 105220336, 70069997, 89165356, 70069997, 105220336, 54989301, 117097056,
    93138304, 27344104, 70069997, 32555654, 56735290, 32555654, 70069997, 27344104, 93138304,
    90403381, 37308200, 89165356, 56735290, 83039340, 56735290, 89165356, 37308200, 90403381,
    93138304, 27344104, 70069997, 32555654, 56735290, 32555654, 70069997, 27344104, 93138304,
    117097056, 54989301, 105220336, 70069997, 89165356, 70069997, 105220336, 54989301, 117097056,
    81901859, 29572998, 54989301, 27344104, 37308200, 27344104, 54989301, 29572998, 81901859,
    91828984, 81901859, 117097056, 93138304, 90403381, 93138304, 117097056, 81901859, 91828984,
];

/// Helper function to validate counts match expected pattern for all boards (no filtering)
/// Returns Ok(()) if counts match exactly, Err(description) if they don't match
pub fn validate_expected_counts(actual_counts: &[u32]) -> Result<(), String> {
    if actual_counts.len() != 81 {
        return Err(format!("Expected 81 counts, got {}", actual_counts.len()));
    }

    for (i, (&actual, &expected)) in actual_counts.iter().zip(EXPECTED_ALL_BOARDS_COUNTS.iter()).enumerate() {
        if actual != expected {
            return Err(format!("Count mismatch at position {} (row {}, col {}): expected {}, got {}",
                       i, i / 9, i % 9, expected, actual));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Create test data with a few sample boards in delta-encoded format
    fn create_test_delta_data() -> Vec<u8> {
        let mut data = Vec::new();

        // First board (stored as-is)
        let board1: u128 = 0x123456789ABCDEF0_123456789ABCDEF0;
        data.extend_from_slice(&board1.to_le_bytes());

        // Second board (stored as delta from first)
        let board2: u128 = 0x111111111111111_111111111111111;
        let delta2 = board1 ^ board2;
        data.extend_from_slice(&delta2.to_le_bytes());

        // Third board (stored as delta from second)
        let board3: u128 = 0x222222222222222_222222222222222;
        let delta3 = board2 ^ board3;
        data.extend_from_slice(&delta3.to_le_bytes());

        data
    }

    #[test]
    fn test_delta_decoding_basic() {
        let test_data = create_test_delta_data();
        let cursor = Cursor::new(test_data);

        // Filter with no restrictions (all boards should match)
        let (counts, matched) = filter_and_count_reader(cursor, 0, 0).unwrap();

        assert_eq!(matched, 3, "Should match all 3 test boards");
        assert_eq!(counts.len(), 81, "Should have 81 cell counts");
    }

    #[test]
    fn test_raw_vs_delta_same_data() {
        // Create the same boards in both raw and delta format
        let board1: u128 = 0x123456789ABCDEF0_123456789ABCDEF0;
        let board2: u128 = 0x111111111111111_111111111111111;
        let board3: u128 = 0x222222222222222_222222222222222;

        // Raw format
        let mut raw_data = Vec::new();
        raw_data.extend_from_slice(&board1.to_le_bytes());
        raw_data.extend_from_slice(&board2.to_le_bytes());
        raw_data.extend_from_slice(&board3.to_le_bytes());

        // Delta format
        let delta_data = create_test_delta_data();

        // Process both formats
        let (raw_counts, raw_matched) = filter_and_count_reader_raw(Cursor::new(raw_data), 0, 0).unwrap();
        let (delta_counts, delta_matched) = filter_and_count_reader(Cursor::new(delta_data), 0, 0).unwrap();

        assert_eq!(raw_matched, delta_matched, "Both formats should match same number of boards");
        assert_eq!(raw_counts, delta_counts, "Both formats should produce identical counts");
    }

    #[test]
    fn test_filtering_logic() {
        let test_data = create_test_delta_data();

        // Test hit mask filtering
        let hit_mask: u128 = 0x1; // Require bit 0 to be set
        let (_, matched_with_hit) = filter_and_count_reader(Cursor::new(test_data.clone()), hit_mask, 0).unwrap();

        // Test miss mask filtering
        let miss_mask: u128 = 0x1; // Require bit 0 to NOT be set
        let (_, matched_with_miss) = filter_and_count_reader(Cursor::new(test_data.clone()), 0, miss_mask).unwrap();

        // Test no filtering
        let (_, matched_no_filter) = filter_and_count_reader(Cursor::new(test_data), 0, 0).unwrap();

        // With filtering, we should get fewer or equal matches
        assert!(matched_with_hit <= matched_no_filter);
        assert!(matched_with_miss <= matched_no_filter);
    }    /// Test that validates the exact expected counts from your real dataset
    #[test]
    fn test_expected_all_boards_counts_with_real_data() {
        // Test with the actual delta-encoded zstd compressed board data
        let data_path = "data/deltas.bin.zst.22";

        // Skip test if data file doesn't exist (for CI/other environments)
        if !std::path::Path::new(data_path).exists() {
            eprintln!("Skipping test - data file not found: {}", data_path);
            return;
        }

        let (counts, matched) = filter_and_count(data_path, 0, 0)
            .expect("Failed to process board data file");

        println!("Processed {} total boards", matched);
        println!("Validating against expected counts...");

        // Use the public validation function
        validate_expected_counts(&counts)
            .expect("Counts don't match expected values!");

        println!("✅ All counts match expected values perfectly!");
    }

    /// Quick smoke test to ensure the data file can be read
    #[test]
    fn test_data_file_smoke_test() {
        let data_path = "data/deltas.bin.zst.22";

        // Skip test if data file doesn't exist
        if !std::path::Path::new(data_path).exists() {
            eprintln!("Skipping smoke test - data file not found: {}", data_path);
            return;
        }

        // Test reading just the first few records
        let mut reader = create_file_reader(data_path).expect("Failed to create reader");

        // Read with delta decoding for first few records
        let mut buf = [0u8; 16];
        let mut prev_record = [0u8; 16];
        let mut first_record = true;

        for i in 0..10 {
            match reader.read_exact(&mut buf) {
                Ok(()) => {
                    // Handle delta decoding for first few records
                    let current_record = if first_record {
                        first_record = false;
                        buf
                    } else {
                        let mut decoded = [0u8; 16];
                        for j in 0..16 {
                            decoded[j] = buf[j] ^ prev_record[j];
                        }
                        decoded
                    };

                    let raw = u128::from_le_bytes(current_record);
                    println!("Record {}: 0x{:032x}", i, raw);
                    prev_record.copy_from_slice(&current_record);
                },
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    println!("Reached end of file at record {}", i);
                    break;
                },
                Err(e) => panic!("Error reading record {}: {}", i, e),
            }
        }

        println!("✅ Successfully read first 10 records from data file");
    }

    #[test]
    fn test_validate_expected_counts_function() {
        // Test the validation function itself
        assert!(validate_expected_counts(&EXPECTED_ALL_BOARDS_COUNTS).is_ok());

        // Test with wrong counts
        let mut wrong_counts = EXPECTED_ALL_BOARDS_COUNTS.to_vec();
        wrong_counts[0] += 1;
        assert!(validate_expected_counts(&wrong_counts).is_err());

        // Test with wrong length
        let short_counts = vec![0u32; 80];
        assert!(validate_expected_counts(&short_counts).is_err());
    }

    /// Test with a limited number of records to verify counting logic is working
    #[test]
    fn test_limited_records_counting() {
        let data_path = "data/deltas.bin.zst.22";

        // Skip test if data file doesn't exist
        if !std::path::Path::new(data_path).exists() {
            eprintln!("Skipping limited test - data file not found: {}", data_path);
            return;
        }

        // Create a limited reader that only processes first 1000 records
        let mut reader = create_file_reader(data_path).expect("Failed to create reader");
        let mut buf = [0u8; 16];
        let mut counts = vec![0u32; 81];
        let mut total_matched: u64 = 0;
        let mut prev_record = [0u8; 16];
        let mut first_record = true;

        const MAX_RECORDS: usize = 1000;

        for _ in 0..MAX_RECORDS {
            match reader.read_exact(&mut buf) {
                Ok(()) => {},
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => panic!("Error reading: {}", e),
            }

            // Handle delta decoding
            let current_record = if first_record {
                first_record = false;
                buf
            } else {
                let mut decoded = [0u8; 16];
                for i in 0..16 {
                    decoded[i] = buf[i] ^ prev_record[i];
                }
                decoded
            };

            // Parse u128 in little endian
            let raw = u128::from_le_bytes(current_record);

            // No filtering for this test (hit_mask=0, miss_mask=0)
            // Count bits
            total_matched += 1;
            for bit in 0..81 {
                let byte_index = bit / 8;
                let bit_index = bit % 8;
                if (current_record[byte_index] >> bit_index) & 1 == 1 {
                    counts[bit] += 1;
                }
            }

            prev_record.copy_from_slice(&current_record);
        }

        println!("Processed {} records from data file", total_matched);
        println!("Sample counts (first 9 positions): {:?}", &counts[0..9]);

        // Basic sanity checks
        assert!(total_matched > 0, "Should have processed some records");
        assert!(counts.iter().any(|&c| c > 0), "Should have some non-zero counts");

        println!("✅ Limited counting test passed!");
    }

    /// Test with progress reporting for the full dataset
    #[test]
    fn test_full_data_with_progress() {
        let data_path = "data/deltas.bin.zst.22";

        // Skip test if data file doesn't exist
        if !std::path::Path::new(data_path).exists() {
            eprintln!("Skipping progress test - data file not found: {}", data_path);
            return;
        }

        // Create a reader for the full dataset
        let mut reader = create_file_reader(data_path).expect("Failed to create reader");
        let mut buf = [0u8; 16];
        let mut counts = vec![0u32; 81];
        let mut total_matched: u64 = 0;
        let mut prev_record = [0u8; 16];
        let mut first_record = true;
        let start_time = std::time::Instant::now();

        loop {
            match reader.read_exact(&mut buf) {
                Ok(()) => {},
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => panic!("Error reading: {}", e),
            }

            // Handle delta decoding
            let current_record = if first_record {
                first_record = false;
                buf
            } else {
                let mut decoded = [0u8; 16];
                for i in 0..16 {
                    decoded[i] = buf[i] ^ prev_record[i];
                }
                decoded
            };

            // No filtering for this test (hit_mask=0, miss_mask=0)
            // Count bits
            total_matched += 1;
            for bit in 0..81 {
                let byte_index = bit / 8;
                let bit_index = bit % 8;
                if (current_record[byte_index] >> bit_index) & 1 == 1 {
                    counts[bit] += 1;
                }
            }

            prev_record.copy_from_slice(&current_record);

            // Progress reporting every 10 million records
            if total_matched % 10_000_000 == 0 {
                let elapsed = start_time.elapsed();
                let rate = total_matched as f64 / elapsed.as_secs_f64();
                println!("Processed {} million records ({:.1}M records/sec)",
                         total_matched / 1_000_000, rate / 1_000_000.0);
            }
        }

        let elapsed = start_time.elapsed();
        println!("Processed {} total records in {:.2} seconds", total_matched, elapsed.as_secs_f64());
        println!("Average rate: {:.1}M records/sec", total_matched as f64 / elapsed.as_secs_f64() / 1_000_000.0);

        // Validate against expected counts
        println!("Validating against expected counts...");
        validate_expected_counts(&counts)
            .expect("Counts don't match expected values!");

        println!("✅ Full dataset validation passed!");
    }
}
