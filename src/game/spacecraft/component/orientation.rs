use crate::prelude::*;

#[derive(Clone, Serialize, Deserialize, Debug, Copy, Default, PartialEq, Eq)]
/// This needs adjusting such that orientation is 0 for right, for up it's PI/2, for left it's PI, for down it's -PI/2
pub enum Orientation {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

impl Orientation {
    pub fn to_radians(&self) -> f32 {
        match self {
            Orientation::Up => std::f32::consts::PI / 2.,
            Orientation::Down => -std::f32::consts::PI / 2.,
            Orientation::Left => std::f32::consts::PI,
            Orientation::Right => 0.,
        }
    }
    pub fn next(&self) -> Self {
        match &self {
            Orientation::Up => Orientation::Right,
            Orientation::Right => Orientation::Down,
            Orientation::Down => Orientation::Left,
            Orientation::Left => Orientation::Up,
        }
    }
}

pub trait Orient {
    fn orient(&self, orientation: Orientation) -> Self;
}

impl Orient for IVec2 {
    fn orient(&self, orientation: Orientation) -> Self {
        match orientation {
            Orientation::Right => *self,
            Orientation::Up => ivec2(-self.y, self.x),
            Orientation::Left => ivec2(-self.x, -self.y),
            Orientation::Down => ivec2(self.y, -self.x),
        }
    }
}

impl Orient for Vec2 {
    fn orient(&self, orientation: Orientation) -> Self {
        match orientation {
            Orientation::Right => *self,
            Orientation::Up => Vec2::new(-self.y, self.x),
            Orientation::Left => Vec2::new(-self.x, -self.y),
            Orientation::Down => Vec2::new(self.y, -self.x),
        }
    }
}
