use ultraviolet::{Vec2, Vec3};

use crate::math::*;

#[derive(Clone, Default)]
pub struct World {
    pub balls: Vec<Ball>,
    pub bricks: Vec<Quad>,
    pub paddle: Quad,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Quad {
    pub pos: Vec2,
    pub vel: Vec2,
    pub dims: Vec2,
    pub color: Vec3,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Ball {
    pub pos: Vec2,
    pub vel: Vec2,
    pub radius: f32,
}

impl World {
    pub fn reset(&mut self) {
        self.balls.clear();
        self.bricks.clear();
        self.paddle = Default::default();
    }

    pub fn create_ball(&mut self, pos: Vec2) {
        self.balls.push(Ball {
            pos,
            vel: 135. * random_direction(),
            radius: 1.,
        });
    }
}
