use ultraviolet::{Vec2, Vec3};

use std::fmt;

#[derive(Clone, Debug)]
pub struct Name(pub String);

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Position(pub Vec2);

#[derive(Copy, Clone, Debug)]
pub struct Velocity(pub Vec2);

#[derive(Copy, Clone, Debug)]
pub struct HitableQuad {
    pub dims: Vec2,
}

#[derive(Copy, Clone, Debug)]
pub struct HitableBall {
    pub radius: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct DrawableColoredQuad {
    pub dims: Vec2,
    pub color: Vec3,
}

#[derive(Copy, Clone, Debug)]
pub struct DrawableColoredBall {
    pub radius: f32,
    pub color: Vec3,
}

#[derive(Copy, Clone, Debug)]
pub struct Paddle;

#[derive(Copy, Clone, Debug)]
pub struct Breakable;

#[derive(Copy, Clone, Debug)]
pub struct Bounce(pub Vec2);
