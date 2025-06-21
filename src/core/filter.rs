use std::io::{self};

/// Reads an iterator of u128 hit masks, filters records by hit/miss masks,
/// and accumulates counts of hits per cell (81 cells).
pub fn filter_and_count<I>(reader: I, hit_mask: u128, miss_mask: u128) -> io::Result<(Vec<u32>, u64)>
where
    I: IntoIterator<Item = io::Result<u128>>,
{
    let mut counts = vec![0u32; 81];
    let mut total_matched: u64 = 0;

    for board in reader {
        let board = match board {
            Ok(val) => val,
            Err(e) => return Err(e),
        };

        // Filter
        if (board & hit_mask) != hit_mask { continue; }
        if (board & miss_mask) != 0 { continue; }

        // Count matched board
        total_matched += 1;

        // Count hits per cell
        let mut mask = board;
        while mask != 0 {
            let tz = mask.trailing_zeros() as usize;
            counts[tz] += 1;
            mask &= mask - 1; // Faster way to clear lowest set bit
        }
    }

    Ok((counts, total_matched))
}
