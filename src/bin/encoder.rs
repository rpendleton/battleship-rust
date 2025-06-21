use std::io::{self, Read, Write};

const RECORD_SIZE: usize = 16;

struct ChunkIndex {
    offset: u64,
    count: u64,
    union: u128,
    intersection: u128,
}

pub fn write_chunk<R: Read, W: Write>(reader: &mut R, writer: &mut W, max_records: usize) -> io::Result<(u32, u128, u128)> {
    let mut buffer = [0u8; RECORD_SIZE];

    let mut intersection = !0u128; // Start with all bits set
    let mut union = 0u128; // Start with no bits set
    let mut last_record = 0u128;
    let mut count = 0u64;

    for _ in 0..max_records {
        match reader.read_exact(&mut buffer) {
            Ok(()) => {
                let record_value = u128::from_le_bytes(buffer.try_into().unwrap());
                count += 1;

                // Update union and intersection
                union |= record_value;
                intersection &= record_value;

                // Calculate the delta
                let delta = record_value ^ last_record;
                last_record = record_value;

                // Write the delta to the writer
                writer.write_all(&delta.to_le_bytes())?;
            }

            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                // EOF reached, stop reading
                break;
            }

            Err(e) => return Err(e),
        }
    }

    // Flush the writer to ensure all data is written
    writer.flush()?;

    // Return the number of records processed, union, and intersection
    Ok((count as u32, union, intersection))
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();

    let mut reader = stdin.lock();
    let mut writer = stdout.lock();

    loop {
        match write_chunk(&mut reader, &mut writer, 500_000_000) {
            Ok((count, union, intersection)) => {
                if count == 0 {
                    // No more records to process, exit the loop
                    break;
                }

                // Print the results for this chunk
                eprintln!("Processed {} records. Union: {:x}, Intersection: {:x}", count, union, intersection);
            }

            Err(e) => {
                eprintln!("Error processing chunk: {}", e);
                return Err(e);
            }
        }
    }

    writer.flush()?;
    Ok(())
}
