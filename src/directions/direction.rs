#[derive(Debug, Clone, Copy, Default)]
pub struct Direction {
    pub x: i32,
    pub y: i32,
}

impl std::ops::AddAssign<Direction> for &mut Direction {
    fn add_assign(&mut self, rhs: Direction) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}
impl std::ops::AddAssign for Direction {
    fn add_assign(mut self: &mut Self, rhs: Self) {
        self += rhs
    }
}

#[derive(Debug, Clone)]
pub enum CardinalDirection {
    North,
    East,
    South,
    West,
}
