pub mod reader;
pub mod filter;
pub mod ffi;
pub mod constants;

#[cfg(test)]
mod tests;

// Re-export the main public API
pub use filter::{
    filter_and_count,
    filter_and_count_with_format,
    filter_and_count_reader,
    filter_and_count_reader_raw,
};

pub use constants::{
    EXPECTED_ALL_BOARDS_COUNTS,
    validate_expected_counts,
};

// FFI functions are not re-exported as they should be used directly via C bindings
