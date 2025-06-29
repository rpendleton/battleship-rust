use rayon::prelude::*;

/// Reads an iterator of u128 hit masks, filters records by hit/miss masks,
/// and accumulates counts of hits per cell (81 cells).
pub fn filter_and_count<I>(reader: I, hit_mask: u128, miss_mask: u128) -> std::io::Result<([u32; 81], u64)>
where
    I: IntoIterator<Item = std::io::Result<u128>>,
{
    const CHUNK_SIZE: usize = 1_000_000;
    let mut counts = [0u32; 81];
    let mut total_matched = 0u64;
    let mut chunk = Vec::with_capacity(CHUNK_SIZE);

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

        chunk.push(board);

        if chunk.len() == CHUNK_SIZE {
            let local_counts = process_chunk(&chunk);
            chunk.clear();

            for i in 0..81 {
                counts[i] += local_counts[i];
            }
        }
    }

    if !chunk.is_empty() {
        let local_counts = process_chunk(&chunk);
        for i in 0..81 {
            counts[i] += local_counts[i];
        }
    }

    Ok((counts, total_matched))
}

fn process_chunk(chunk: &[u128]) -> [u32; 81] {
    chunk.par_iter()
        .map(|&board| {
            let mut cell_counts = [0u32; 81];

            // Count hits per cell (only consider bits 0-80 for 81-cell board)
            let mut mask = board & ((1u128 << 81) - 1); // Mask to only consider first 81 bits
            while mask != 0 {
                let bit = mask.trailing_zeros() as usize;
                if bit < 81 {
                    cell_counts[bit] += 1;
                }
                mask &= mask - 1; // Faster way to clear lowest set bit
            }

            cell_counts
        })
        .reduce(
            || [0u32; 81],
            |mut acc_counts, counts| {
                for i in 0..81 {
                    acc_counts[i] += counts[i];
                }
                acc_counts
            },
        )
}
