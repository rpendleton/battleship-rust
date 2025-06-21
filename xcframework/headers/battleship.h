#ifndef BATTLESHIP_H
#define BATTLESHIP_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Filters and counts boards based on hit/miss bitmasks.
 *
 * @param path_ptr       A pointer to a null-terminated UTF-8 path string.
 * @param hit_mask_low   Lower 64 bits of the hit mask.
 * @param hit_mask_high  Upper 64 bits of the hit mask.
 * @param miss_mask_low  Lower 64 bits of the miss mask.
 * @param miss_mask_high Upper 64 bits of the miss mask.
 * @param out_counts     Pointer to a 81-element u32 array for cell counts.
 * @return               The number of matching boards.
 */
uint64_t filter_and_count_ffi(
    const char *path_ptr,
    uint64_t hit_mask_low,
    uint64_t hit_mask_high,
    uint64_t miss_mask_low,
    uint64_t miss_mask_high,
    uint32_t *out_counts
);

#ifdef __cplusplus
}
#endif

#endif // BATTLESHIP_H
