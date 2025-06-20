use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

/// The zstd magic number (little endian: [0x28, 0xB5, 0x2F, 0xFD])
const ZSTD_MAGIC: [u8; 4] = [0x28, 0xB5, 0x2F, 0xFD];

/// Creates a reader that automatically handles zstd compression by chaining magic bytes back.
fn create_reader_with_magic_detection<R: Read + 'static>(mut reader: R) -> io::Result<Box<dyn Read>> {
    let mut magic = [0u8; 4];

    match reader.read_exact(&mut magic) {
        Ok(()) => {
            if magic == ZSTD_MAGIC {
                // It's zstd compressed, prepend magic bytes and wrap with decoder
                let chained = std::io::Cursor::new(magic).chain(reader);
                let decoder = zstd::stream::Decoder::new(chained)?;
                Ok(Box::new(decoder))
            } else {
                // Not zstd, prepend the magic bytes we consumed
                let chained = std::io::Cursor::new(magic).chain(reader);
                Ok(Box::new(chained))
            }
        }
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
            // Empty or too small, return as-is
            Ok(Box::new(reader))
        }
        Err(e) => Err(e),
    }
}

/// Creates a reader for a file path that automatically handles zstd compression.
pub fn create_file_reader<P: AsRef<Path>>(path: P) -> io::Result<Box<dyn Read>> {
    let file = File::open(path)?;
    create_reader_with_magic_detection(file)
}

/// Creates a reader for stdin that automatically handles zstd compression.
pub fn create_stdin_reader() -> io::Result<Box<dyn Read>> {
    let stdin = io::stdin();
    create_reader_with_magic_detection(stdin)
}
