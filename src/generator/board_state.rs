use crate::generator::{board_mask::BoardMask, common_masks::CommonMasks, point::{Direction, Point}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    Open,
    Hit,
    Miss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardState {
    hit_mask: BoardMask,
    miss_mask: BoardMask,
    three_count_remaining: usize,
    four_count_remaining: usize,
}

impl BoardState {
    pub const EMPTY: Self = Self {
        hit_mask: BoardMask::EMPTY,
        miss_mask: BoardMask::EMPTY,
        three_count_remaining: 5,
        four_count_remaining: 3,
    };

    pub fn hit_mask(&self) -> BoardMask {
        self.hit_mask
    }

    pub fn miss_mask(&self) -> BoardMask {
        self.miss_mask
    }

    pub fn open_mask(&self) -> BoardMask {
        BoardMask::FULL & !self.hit_mask & !self.miss_mask
    }

    pub fn three_count_remaining(&self) -> usize {
        self.three_count_remaining
    }

    pub fn four_count_remaining(&self) -> usize {
        self.four_count_remaining
    }

    pub fn get(&self, point: Point) -> CellState {
        if self.hit_mask.get(point) {
            CellState::Hit
        }
        else if self.miss_mask.get(point) {
            CellState::Miss
        }
        else {
            CellState::Open
        }
    }

    pub fn set(&mut self, point: Point, state: CellState) {
        match state {
            CellState::Hit => {
                self.hit_mask.set(point, true);
                self.miss_mask.set(point, false);
            }
            CellState::Miss => {
                self.hit_mask.set(point, false);
                self.miss_mask.set(point, true);
            }
            CellState::Open => {
                self.hit_mask.set(point, false);
                self.miss_mask.set(point, false);
            }
        }
    }

    pub fn placing_ship(&self, length: i32, starting_point: Point, direction: Direction) -> Option<BoardState> {
        let mut copy = *self;

        match length {
            3 => {
                if copy.three_count_remaining == 0 {
                    return None;
                }
                else {
                    copy.three_count_remaining -= 1;
                }
            }
            4 => {
                if copy.four_count_remaining == 0 {
                    return None;
                }
                else {
                    copy.four_count_remaining -= 1;
                }
            }
            _ => return None,
        }

        let move_hit_mask = CommonMasks::mask_for_ship_hit(length, starting_point, direction);
        let move_miss_mask = CommonMasks::mask_for_ship_outline(length, starting_point, direction);

        if (self.hit_mask & move_hit_mask).raw_value() != 0 {
            return None;
        }

        if (self.hit_mask & move_miss_mask).raw_value() != 0 {
            return None;
        }

        copy.hit_mask = self.hit_mask | move_hit_mask;
        copy.miss_mask = self.miss_mask | move_miss_mask;

        Some(copy)
    }

    pub fn place_ship(&mut self, length: i32, starting_point: Point, direction: Direction) -> bool {
        if let Some(new_state) = self.placing_ship(length, starting_point, direction) {
            *self = new_state;
            true
        }
        else {
            false
        }
    }

    pub fn debug_description(&self) -> String {
        let mut result = String::from("┌───────────────────┐\n");

        for y in 0..9 {
            result.push('│');

            for x in 0..9 {
                let point = Point::new(x, y);
                match self.get(point) {
                    CellState::Hit => result.push_str(" X"),
                    CellState::Miss => result.push_str(" •"),
                    CellState::Open => result.push_str("  "),
                }
            }

            result.push_str(" │\n");
        }

        result.push_str("└───────────────────┘");
        result
    }
}
