use ultraviolet::{Vec2, Vec3};

#[derive(Clone, Debug)]
pub struct Name(pub String);

#[derive(Copy, Clone, Debug)]
pub struct Position(pub Vec2);

#[derive(Copy, Clone, Debug)]
pub struct Velocity(pub Vec2);

#[derive(Copy, Clone, Debug)]
pub struct HitableQuad {
    pub dims: Vec2,
}

#[derive(Copy, Clone, Debug)]
pub struct DrawableColoredQuad {
    pub dims: Vec2,
    pub color: Vec3,
}

#[derive(Copy, Clone, Debug)]
pub struct Ball;

#[derive(Copy, Clone, Debug)]
pub struct Paddle;
