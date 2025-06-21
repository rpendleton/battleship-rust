pub type Bitboard = u128;

const BOARD_SIZE: usize = 9;

fn index(x: usize, y: usize) -> usize {
    y * BOARD_SIZE + x
}

fn get_bit(board: Bitboard, x: usize, y: usize) -> bool {
    (board >> index(x, y)) & 1 == 1
}

fn set_bit(board: &mut Bitboard, x: usize, y: usize) {
    *board |= 1u128 << index(x, y);
}

fn transform<F>(original: Bitboard, transform_fn: F) -> Bitboard
where
    F: Fn(usize, usize) -> (usize, usize),
{
    let mut result: Bitboard = 0;
    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            if get_bit(original, x, y) {
                let (nx, ny) = transform_fn(x, y);
                set_bit(&mut result, nx, ny);
            }
        }
    }
    result
}

pub fn generate_symmetries(board: Bitboard) -> Vec<Bitboard> {
    vec![
        board,
        transform(board, |x, y| (BOARD_SIZE - 1 - x, y)),              // horizontal flip
        transform(board, |x, y| (x, BOARD_SIZE - 1 - y)),              // vertical flip
        transform(board, |x, y| (BOARD_SIZE - 1 - x, BOARD_SIZE - 1 - y)), // rotate 180°
        transform(board, |x, y| (y, x)),                              // transpose (main diag)
        transform(board, |x, y| (BOARD_SIZE - 1 - y, x)),              // rotate 90°
        transform(board, |x, y| (y, BOARD_SIZE - 1 - x)),              // rotate 270°
        transform(board, |x, y| (BOARD_SIZE - 1 - y, BOARD_SIZE - 1 - x)), // anti-diagonal mirror
    ]
}

pub fn canonicalize(board: Bitboard) -> Bitboard {
    generate_symmetries(board).into_iter().min().unwrap()
}

pub fn is_canonical(board: Bitboard) -> bool {
    let symmetries = generate_symmetries(board);
    board == *symmetries.iter().min().unwrap()
}
