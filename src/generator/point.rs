use std::ops::{Add, Mul, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Horizontal,
    Vertical,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
        Point::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, rhs: Point) -> Self::Output {
        Point::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Add<Direction> for Point {
    type Output = Point;

    fn add(self, rhs: Direction) -> Self::Output {
        match rhs {
            Direction::Horizontal => Point::new(self.x + 1, self.y),
            Direction::Vertical => Point::new(self.x, self.y + 1),
        }
    }
}

impl Sub<Direction> for Point {
    type Output = Point;

    fn sub(self, rhs: Direction) -> Self::Output {
        match rhs {
            Direction::Horizontal => Point::new(self.x - 1, self.y),
            Direction::Vertical => Point::new(self.x, self.y - 1),
        }
    }
}

impl Mul<i32> for Direction {
    type Output = Point;

    fn mul(self, length: i32) -> Point {
        match self {
            Direction::Horizontal => Point::new(length, 0),
            Direction::Vertical => Point::new(0, length),
        }
    }
}
