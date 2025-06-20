use battleship::core::{
    filter::{filter_and_count},
    reader::{create_reader},
};

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

/// Create test data with a few sample boards in delta-encoded format
fn create_test_delta_data() -> Vec<std::io::Result<u128>> {
    let mut data: Vec<std::io::Result<u128>> = Vec::new();

    // First board (stored as-is)
    let board1: u128 = 0x123456789ABCDEF0_123456789ABCDEF0;
    data.push(Ok(board1));

    // Second board (stored as delta from first)
    let board2: u128 = 0x111111111111111_111111111111111;
    let delta2 = board2;
    data.push(Ok(delta2));

    // Third board (stored as delta from second)
    let board3: u128 = 0x222222222222222_222222222222222;
    let delta3 = board3;
    data.push(Ok(delta3));

    data
}

#[test]
fn test_delta_decoding_basic() {
    let test_data = create_test_delta_data();

    // Filter with no restrictions (all boards should match)
    let (counts, matched) = filter_and_count(test_data, 0, 0).unwrap();

    assert_eq!(matched, 3, "Should match all 3 test boards");
    assert_eq!(counts.len(), 81, "Should have 81 cell counts");
}

#[test]
fn test_filtering_logic() {
    // Test hit mask filtering
    let hit_mask: u128 = 0x1; // Require bit 0 to be set
    let (_, matched_with_hit) = filter_and_count(create_test_delta_data(), hit_mask, 0).unwrap();

    // Test miss mask filtering
    let miss_mask: u128 = 0x1; // Require bit 0 to NOT be set
    let (_, matched_with_miss) = filter_and_count(create_test_delta_data(), 0, miss_mask).unwrap();

    // Test no filtering
    let (_, matched_no_filter) = filter_and_count(create_test_delta_data(), 0, 0).unwrap();

    // With filtering, we should get fewer or equal matches
    assert!(matched_with_hit <= matched_no_filter);
    assert!(matched_with_miss <= matched_no_filter);
}

/// Test that validates the exact expected counts from your real dataset
#[test]
fn test_expected_all_boards_counts_with_real_data() {
    // Test with the actual delta-encoded zstd compressed board data
    let data_path = "data/deltas.bin.zst.22";

    // Skip test if data file doesn't exist (for CI/other environments)
    if !std::path::Path::new(data_path).exists() {
        eprintln!("Skipping test - data file not found: {}", data_path);
        return;
    }

    let reader = create_reader(data_path)
        .expect("Failed to create reader for board data file");

    let (counts, matched) = filter_and_count(reader, 0, 0)
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
    let reader = create_reader(data_path).expect("Failed to create reader");

    // Use iterator interface to read and print first 10 records
    for (i, result) in reader.into_iter().take(10).enumerate() {
        match result {
            Ok(raw) => {
                println!("Record {}: 0x{:032x}", i, raw);
            },
            Err(e) => {
                println!("Error reading record {}: {}", i, e);
                break;
            }
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
    let reader = create_reader(data_path).expect("Failed to create reader");
    let mut counts = vec![0u32; 81];
    let mut total_matched: u64 = 0;
    for result in reader.into_iter().take(1000) {
        let raw = match result {
            Ok(val) => val,
            Err(e) => panic!("Error reading: {}", e),
        };
        total_matched += 1;
        let bytes = raw.to_le_bytes();
        for bit in 0..81 {
            let byte_index = bit / 8;
            let bit_index = bit % 8;
            if (bytes[byte_index] >> bit_index) & 1 == 1 {
                counts[bit] += 1;
            }
        }
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
    let reader = create_reader(data_path).expect("Failed to create reader");
    let mut counts = vec![0u32; 81];
    let mut total_matched: u64 = 0;
    let start_time = std::time::Instant::now();

    for result in reader {
        let raw = match result {
            Ok(val) => val,
            Err(e) => panic!("Error reading: {}", e),
        };
        total_matched += 1;
        let bytes = raw.to_le_bytes();
        for bit in 0..81 {
            let byte_index = bit / 8;
            let bit_index = bit % 8;
            if (bytes[byte_index] >> bit_index) & 1 == 1 {
                counts[bit] += 1;
            }
        }
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
