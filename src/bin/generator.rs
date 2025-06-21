use battleship::generator::board_mask::BoardMask;
use battleship::generator::board_state::{BoardState, CellState};
use battleship::generator::point::{Direction, Point};
use battleship::generator::symmetries::is_canonical;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::Instant;

fn main() {
    write_all_valid_boards("/Users/rpendleton/sd/battleship/battleship-data/workspace/latest.bin");
}

fn time<F, R>(action: F) -> R
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    println!("Starting: {:?}", std::time::SystemTime::now());
    let result = action();
    println!("Done: {:?} (took {:?})", std::time::SystemTime::now(), start.elapsed());
    result
}

#[allow(dead_code)]
fn print_sample_placed_ship() {
    let mut board = BoardState::EMPTY;
    let placed = board.place_ship(3, Point::new(1, 0), Direction::Horizontal);
    assert!(placed);
    println!("{}", board.debug_description());
}

#[allow(dead_code)]
fn print_all_possible_ships() {
    print_all_possible_ships_inner(3, Direction::Horizontal);
    print_all_possible_ships_inner(3, Direction::Vertical);

    print_all_possible_ships_inner(4, Direction::Horizontal);
    print_all_possible_ships_inner(4, Direction::Vertical);
}

fn print_all_possible_ships_inner(length: i32, direction: Direction) {
    println!("Length: {}, Direction: {}", length, match direction {
        Direction::Horizontal => "horizontal",
        Direction::Vertical => "vertical",
    });

    for i in 0..81 {
        let point = BoardMask::point_of(i);

        let mut board = BoardState::EMPTY;
        board.place_ship(length, point, direction);

        if board.hit_mask() != BoardMask::FULL {
            println!("{}", board.debug_description());
            println!("");
        }
    }
}

fn write_all_valid_boards(path: &str) {
    let file = File::create(path).expect("Failed to create file");
    let mut writer = BufWriter::new(file);

    let mut data = Vec::with_capacity(4096);
    let mut written = 0u128;
    let mut last_percentage = 0;

    let start = BoardState::EMPTY;

    let total_valid_count = time(|| count_of_valid_endings(&start, &mut |board| {
        written += 1;

        let new_percentage = written * 100 / 213_723_152;
        if new_percentage > last_percentage {
            println!("{}% at {:?}", new_percentage, std::time::SystemTime::now());
            last_percentage = new_percentage;
        }

        let is_canonical_board = is_canonical(board.hit_mask().raw_value());

        if is_canonical_board {
            let bytes = board.hit_mask().raw_value().to_le_bytes();
            data.extend_from_slice(&bytes);
        }

        if data.len() >= 4096 {
            writer.write_all(&data).expect("Failed to write data");
            data.clear();
        }
    }));

    if !data.is_empty() {
        writer.write_all(&data).expect("Failed to write data");
    }

    println!("Total Valid: {}", total_valid_count);
}

fn count_of_valid_endings<F>(state: &BoardState, save_board: &mut F) -> usize
where
    F: FnMut(&BoardState),
{
    if let Some(point) = state.open_mask().first_set_position() {
        let mut valid = 0;

        if let Some(placed_state) = state.placing_ship(3, point, Direction::Horizontal) {
            valid += count_of_valid_endings(&placed_state, save_board);
        }

        if let Some(placed_state) = state.placing_ship(3, point, Direction::Vertical) {
            valid += count_of_valid_endings(&placed_state, save_board);
        }

        if let Some(placed_state) = state.placing_ship(4, point, Direction::Horizontal) {
            valid += count_of_valid_endings(&placed_state, save_board);
        }

        if let Some(placed_state) = state.placing_ship(4, point, Direction::Vertical) {
            valid += count_of_valid_endings(&placed_state, save_board);
        }

        // Try marking the point as a miss
        let mut unplaced_state = *state;
        unplaced_state.set(point, CellState::Miss);
        valid += count_of_valid_endings(&unplaced_state, save_board);

        valid
    }
    else {
        // No more open positions
        if state.three_count_remaining() == 0 && state.four_count_remaining() == 0 {
            save_board(state);
            1
        }
        else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use battleship::generator::board_mask::BoardMask;
    use battleship::generator::board_state::{BoardState, CellState};
    use battleship::generator::point::{Direction, Point};

    #[test]
    fn test_point_operations() {
        let p1 = Point::new(1, 2);
        let p2 = Point::new(3, 4);
        let sum = p1 + p2;
        assert_eq!(sum.x, 4);
        assert_eq!(sum.y, 6);

        let diff = p2 - p1;
        assert_eq!(diff.x, 2);
        assert_eq!(diff.y, 2);
    }

    #[test]
    fn test_board_mask_basic() {
        let mut mask = BoardMask::EMPTY;
        let point = Point::new(4, 4);

        assert!(!mask.get(point));
        mask.set(point, true);
        assert!(mask.get(point));

        mask.set(point, false);
        assert!(!mask.get(point));
    }

    #[test]
    fn test_board_state_ship_placement() {
        let mut board = BoardState::EMPTY;
        let point = Point::new(1, 2);

        // Should be able to place a 3-length ship
        assert!(board.place_ship(3, point, Direction::Horizontal));

        // Should be in hit state
        assert_eq!(board.get(point), CellState::Hit);
        assert_eq!(board.get(Point::new(2, 2)), CellState::Hit);
        assert_eq!(board.get(Point::new(3, 2)), CellState::Hit);

        // Should have miss outline
        assert_eq!(board.get(Point::new(0, 1)), CellState::Miss);
        assert_eq!(board.get(Point::new(4, 3)), CellState::Miss);
    }

    #[test]
    fn test_board_mask_index_conversion() {
        let point = Point::new(4, 5);
        let index = BoardMask::index_of(point);
        let converted_back = BoardMask::point_of(index);

        assert_eq!(point.x, converted_back.x);
        assert_eq!(point.y, converted_back.y);
    }

    #[test]
    fn test_symmetry_detection() {
        // Test that a symmetric board is detected as canonical
        let board: u128 = 0b101_000_101; // Simple symmetric pattern
        assert!(is_canonical(board));
    }
}
