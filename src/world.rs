use ultraviolet::{Vec2, Vec3};

#[derive(Clone, Default)]
pub struct World {
    pub balls: Vec<Ball>,
    pub bricks: Vec<Quad>,
    pub unbreakable_bricks: Vec<Quad>,
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
        self.unbreakable_bricks.clear();
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

fn random_direction() -> Vec2 {
    use rand::prelude::*;

    // Random angle from (π/2, 3π/4) - this is the center half of the hemisphere
    // facing up in the simulation
    let t: f32 = rand::thread_rng().gen();
    let θ: f32 = 0.5 * std::f32::consts::PI * t + 0.25 * std::f32::consts::PI;

    Vec2::new(f32::cos(θ), f32::sin(θ))
}
