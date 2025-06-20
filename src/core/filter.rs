use std::io::{self, Read};
use std::path::Path;

use crate::core::reader::{create_file_reader, create_stdin_reader};

/// Reads binary data of 16-byte hit masks from any reader, handles delta encoding,
/// filters records by hit/miss masks, and accumulates counts of hits per cell (81 cells).
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

/// Reads a binary file of 16-byte hit masks, filters records by hit/miss masks,
/// and accumulates counts of hits per cell (81 cells).
/// Supports both uncompressed and zstd-compressed files.
/// Pass "-" as the path to read from stdin.
/// Assumes delta-encoded format.
pub fn filter_and_count<P: AsRef<Path>>(
    path: P,
    hit_mask: u128,
    miss_mask: u128,
) -> io::Result<(Vec<u32>, u64)> {
    let path_str = path.as_ref().to_string_lossy();

    let reader = if path_str == "-" {
        create_stdin_reader()?
    } else {
        create_file_reader(path)?
    };
    filter_and_count_reader(reader, hit_mask, miss_mask)
}
