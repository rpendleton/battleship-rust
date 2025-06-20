use crate::filter::filter_and_count;

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
