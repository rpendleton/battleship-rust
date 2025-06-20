use std::io::{self, Read};
use std::convert::TryInto;

const RECORD_SIZE: usize = 16;
const BOARD_BITS: usize = 81;

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    let mut prev = [0u8; RECORD_SIZE];
    let mut current = [0u8; RECORD_SIZE];
    let mut first = true;

    let mut counts = [0u32; BOARD_BITS];

    while let Ok(_) = reader.read_exact(&mut current) {
        // Reconstruct full mask
        if first {
            first = false;
        } else {
            for i in 0..RECORD_SIZE {
                current[i] ^= prev[i];
            }
        }

        // Count bits in 81-bit mask
        for bit in 0..BOARD_BITS {
            let byte_index = bit / 8;
            let bit_index = bit % 8;
            if (current[byte_index] >> bit_index) & 1 == 1 {
                counts[bit] += 1;
            }
        }

        prev.copy_from_slice(&current);
    }

    // Print as a 9×9 grid
    for y in 0..9 {
        for x in 0..9 {
            let idx = y * 9 + x;
            print!("{:>7}", counts[idx]);
            if x < 8 {
                print!(",");
            }
        }
        println!();
    }

    Ok(())
}
