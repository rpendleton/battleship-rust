use crate::generator::{board_mask::BoardMask, point::{Direction, Point}};
use std::sync::Once;

pub struct CommonMasks {
    horizontal_three_long_hit_masks: Vec<BoardMask>,
    horizontal_three_long_miss_masks: Vec<BoardMask>,
    horizontal_four_long_hit_masks: Vec<BoardMask>,
    horizontal_four_long_miss_masks: Vec<BoardMask>,
    vertical_three_long_hit_masks: Vec<BoardMask>,
    vertical_three_long_miss_masks: Vec<BoardMask>,
    vertical_four_long_hit_masks: Vec<BoardMask>,
    vertical_four_long_miss_masks: Vec<BoardMask>,
}

static mut MASKS: Option<CommonMasks> = None;
static INIT: Once = Once::new();

impl CommonMasks {
    fn instance() -> &'static CommonMasks {
        unsafe {
            INIT.call_once(|| {
                MASKS = Some(CommonMasks::new());
            });
            MASKS.as_ref().unwrap()
        }
    }

    fn new() -> Self {
        let mut masks = CommonMasks {
            horizontal_three_long_hit_masks: Vec::with_capacity(81),
            horizontal_three_long_miss_masks: Vec::with_capacity(81),
            horizontal_four_long_hit_masks: Vec::with_capacity(81),
            horizontal_four_long_miss_masks: Vec::with_capacity(81),
            vertical_three_long_hit_masks: Vec::with_capacity(81),
            vertical_three_long_miss_masks: Vec::with_capacity(81),
            vertical_four_long_hit_masks: Vec::with_capacity(81),
            vertical_four_long_miss_masks: Vec::with_capacity(81),
        };

        for i in 0..81 {
            let point = BoardMask::point_of(i);

            masks.horizontal_three_long_hit_masks.push(Self::generate_mask_for_ship_hit(3, point, Direction::Horizontal));
            masks.horizontal_three_long_miss_masks.push(Self::generate_mask_for_ship_outline(3, point, Direction::Horizontal));
            masks.horizontal_four_long_hit_masks.push(Self::generate_mask_for_ship_hit(4, point, Direction::Horizontal));
            masks.horizontal_four_long_miss_masks.push(Self::generate_mask_for_ship_outline(4, point, Direction::Horizontal));

            masks.vertical_three_long_hit_masks.push(Self::generate_mask_for_ship_hit(3, point, Direction::Vertical));
            masks.vertical_three_long_miss_masks.push(Self::generate_mask_for_ship_outline(3, point, Direction::Vertical));
            masks.vertical_four_long_hit_masks.push(Self::generate_mask_for_ship_hit(4, point, Direction::Vertical));
            masks.vertical_four_long_miss_masks.push(Self::generate_mask_for_ship_outline(4, point, Direction::Vertical));
        }

        masks
    }

    pub fn mask_for_ship_hit(length: i32, starting_point: Point, direction: Direction) -> BoardMask {
        let masks = Self::instance();
        let index = BoardMask::index_of(starting_point);

        match (direction, length) {
            (Direction::Horizontal, 3) => masks.horizontal_three_long_hit_masks[index],
            (Direction::Horizontal, 4) => masks.horizontal_four_long_hit_masks[index],
            (Direction::Vertical, 3) => masks.vertical_three_long_hit_masks[index],
            (Direction::Vertical, 4) => masks.vertical_four_long_hit_masks[index],
            _ => panic!("Invalid ship length or direction"),
        }
    }

    pub fn mask_for_ship_outline(length: i32, starting_point: Point, direction: Direction) -> BoardMask {
        let masks = Self::instance();
        let index = BoardMask::index_of(starting_point);

        match (direction, length) {
            (Direction::Horizontal, 3) => masks.horizontal_three_long_miss_masks[index],
            (Direction::Horizontal, 4) => masks.horizontal_four_long_miss_masks[index],
            (Direction::Vertical, 3) => masks.vertical_three_long_miss_masks[index],
            (Direction::Vertical, 4) => masks.vertical_four_long_miss_masks[index],
            _ => panic!("Invalid ship length or direction"),
        }
    }

    fn generate_mask_for_ship_hit(length: i32, starting_point: Point, direction: Direction) -> BoardMask {
        let mut mask = BoardMask::EMPTY;

        let start = starting_point;
        let end = starting_point + direction * (length - 1);

        for x in start.x..=end.x {
            for y in start.y..=end.y {
                let point = Point::new(x, y);

                if BoardMask::contains(point) {
                    mask.set(point, true);
                }
                else {
                    return BoardMask::FULL; // If any point is out of bounds, return FULL mask
                }
            }
        }

        mask
    }

    fn generate_mask_for_ship_outline(length: i32, starting_point: Point, direction: Direction) -> BoardMask {
        let start = starting_point - Point::new(1, 1);
        let end = starting_point + direction * (length - 1) + Point::new(1, 1);

        let hit_mask = Self::generate_mask_for_ship_hit(length, starting_point, direction);

        if hit_mask.raw_value() == BoardMask::FULL.raw_value() {
            return BoardMask::FULL; // If the hit mask is FULL, return FULL mask
        }

        let mut mask = BoardMask::EMPTY;

        for x in start.x..=end.x {
            for y in start.y..=end.y {
                let point = Point::new(x, y);

                if BoardMask::contains(point) {
                    mask.set(point, true);
                }
            }
        }

        mask = mask & !hit_mask;
        mask
    }
}
