use std::io::{self, Read, Write};

const RECORD_SIZE: usize = 16;

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();

    let mut reader = stdin.lock();
    let mut writer = stdout.lock();

    let mut prev_record = [0u8; RECORD_SIZE];
    let mut first_record = true;

    let mut buffer = [0u8; RECORD_SIZE];

    loop {
        // Read exactly RECORD_SIZE bytes
        match reader.read_exact(&mut buffer) {
            Ok(()) => {
                if first_record {
                    // Write first record as-is
                    writer.write_all(&buffer)?;
                    first_record = false;
                } else {
                    // XOR with previous record and write delta
                    let mut delta = [0u8; RECORD_SIZE];
                    for i in 0..RECORD_SIZE {
                        delta[i] = buffer[i] ^ prev_record[i];
                    }
                    writer.write_all(&delta)?;
                }
                prev_record.copy_from_slice(&buffer);
            }
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                // EOF reached, done
                break;
            }
            Err(e) => return Err(e),
        }
    }

    writer.flush()?;
    Ok(())
}
