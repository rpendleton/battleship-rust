use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

/// The zstd magic number (little endian: [0x28, 0xB5, 0x2F, 0xFD])
const ZSTD_MAGIC: [u8; 4] = [0x28, 0xB5, 0x2F, 0xFD];

/// A reader that yields delta-XOR decoded u128s from an underlying reader.
pub struct DeltaDecodingReader<R: Read> {
    inner: R,
    prev: [u8; 16],
    first: bool,
}

impl<R: Read> DeltaDecodingReader<R> {
    pub fn new(inner: R) -> Self {
        Self { inner, prev: [0u8; 16], first: true }
    }
}

impl<R: Read> Iterator for DeltaDecodingReader<R> {
    type Item = io::Result<u128>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = [0u8; 16];
        match self.inner.read_exact(&mut buf) {
            Ok(()) => {
                let decoded = if self.first {
                    self.first = false;
                    buf.clone()
                } else {
                    let mut out = [0u8; 16];
                    for i in 0..16 {
                        out[i] = buf[i] ^ self.prev[i];
                    }
                    out
                };
                self.prev.copy_from_slice(&decoded);
                Some(Ok(u128::from_le_bytes(decoded)))
            }
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => None,
            Err(e) => Some(Err(e)),
        }
    }
}

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

/// Creates a delta-decoding iterator for a file path that automatically handles zstd compression.
fn create_file_reader<P: AsRef<Path>>(path: P) -> io::Result<DeltaDecodingReader<Box<dyn Read>>> {
    let file = File::open(path)?;
    let reader = create_reader_with_magic_detection(file)?;
    Ok(DeltaDecodingReader::new(reader))
}

/// Creates a delta-decoding iterator for stdin that automatically handles zstd compression.
fn create_stdin_reader() -> io::Result<DeltaDecodingReader<Box<dyn Read>>> {
    let stdin = io::stdin();
    let reader = create_reader_with_magic_detection(stdin)?;
    Ok(DeltaDecodingReader::new(reader))
}

/// Creates a delta-decoding iterator for a given path, handling both file and stdin input, as well as zstd compression.
pub fn create_reader<P: AsRef<Path>>(path: P) -> io::Result<impl IntoIterator<Item = io::Result<u128>>> {
    let path_str = path.as_ref().to_string_lossy();
    if path_str == "-" {
        create_stdin_reader()
    } else {
        create_file_reader(path)
    }
}