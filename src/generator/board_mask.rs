use crate::generator::point::Point;
use std::ops::{BitAnd, BitOr, Not, Shl, Shr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardMask {
    raw_value: u128,
}

impl BoardMask {
    pub const FULL: BoardMask = BoardMask {
        raw_value: (1u128 << 81) - 1,
    };

    pub const EMPTY: BoardMask = BoardMask {
        raw_value: 0,
    };

    pub fn new(raw_value: u128) -> Self {
        Self { raw_value }
    }

    pub fn raw_value(&self) -> u128 {
        self.raw_value
    }

    pub fn get(&self, point: Point) -> bool {
        let index = Self::index_of(point);
        (self.raw_value & (1u128 << index)) != 0
    }

    pub fn set(&mut self, point: Point, value: bool) {
        let index = Self::index_of(point);

        if value {
            self.raw_value |= 1u128 << index;
        }
        else {
            self.raw_value &= !(1u128 << index);
        }
    }

    pub fn first_set_position(&self) -> Option<Point> {
        if self.raw_value == 0 {
            None
        }
        else {
            Some(Self::point_of(self.raw_value.trailing_zeros() as usize))
        }
    }

    pub fn contains(point: Point) -> bool {
        (0..9).contains(&point.x) && (0..9).contains(&point.y)
    }

    pub fn index_of(point: Point) -> usize {
        assert!(Self::contains(point), "Point {:?} is out of bounds", point);
        (point.y * 9 + point.x) as usize
    }

    pub fn point_of(index: usize) -> Point {
        assert!(index < 81, "Index {} is out of bounds", index);
        let x = (index % 9) as i32;
        let y = (index / 9) as i32;
        Point::new(x, y)
    }
}

impl BitAnd for BoardMask {
    type Output = BoardMask;

    fn bitand(self, rhs: BoardMask) -> Self::Output {
        BoardMask::new(self.raw_value & rhs.raw_value)
    }
}

impl BitOr for BoardMask {
    type Output = BoardMask;

    fn bitor(self, rhs: BoardMask) -> Self::Output {
        BoardMask::new(self.raw_value | rhs.raw_value)
    }
}

impl Shl<usize> for BoardMask {
    type Output = BoardMask;

    fn shl(self, rhs: usize) -> Self::Output {
        BoardMask::new(self.raw_value << rhs)
    }
}

impl Shr<usize> for BoardMask {
    type Output = BoardMask;

    fn shr(self, rhs: usize) -> Self::Output {
        BoardMask::new(self.raw_value >> rhs)
    }
}

impl Not for BoardMask {
    type Output = BoardMask;

    fn not(self) -> Self::Output {
        BoardMask::new(!self.raw_value & BoardMask::FULL.raw_value)
    }
}
