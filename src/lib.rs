#[path = "lib/ffi.rs"]
pub mod ffi;
#[path = "lib/filter.rs"]
pub mod filter;
#[path = "lib/reader.rs"]
pub mod reader;

#[cfg(test)]
#[path = "lib/tests.rs"]
mod tests;
