use ultraviolet::{Vec2, Vec3};

#[derive(Copy, Clone, Debug)]
pub struct Aabb {
    pub min: Vec2,
    pub max: Vec2,
}

#[derive(Copy, Clone, Debug)]
pub struct HitInfo {
    pub time: f32,
    pub pos: Vec2,
    pub normal: Vec2,
}

impl Aabb {
    pub fn new_from_quad(pos: Vec2, dims: Vec2) -> Self {
        let min = Vec2::min_by_component(pos, pos + dims);
        let max = Vec2::max_by_component(pos, pos + dims);

        Self { min, max }
    }

    pub fn center(&self) -> Vec2 {
        0.5 * (self.max + self.min)
    }

    pub fn half_extents(&self) -> Vec2 {
        0.5 * (self.max - self.min)
    }

    pub fn intersects_with_line(
        &self,
        pos: Vec2,
        delta: Vec2,
        padding: Option<Vec2>,
    ) -> Option<HitInfo> {
        let padding = padding.unwrap_or_default();

        fn sign(x: f32) -> f32 {
            if x < 0. {
                return -1.;
            }
            if x > 0. {
                return 1.;
            }

            // 0. and NaNs
            0.
        }

        let sign = delta.map(sign);
        let near_time = (self.center() - sign * (self.half_extents() + padding) - pos) / delta;
        let far_time = (self.center() + sign * (self.half_extents() + padding) - pos) / delta;

        if (near_time.x > far_time.y) || (near_time.y > far_time.x) {
            return None;
        }

        let normal = if near_time.x > near_time.y {
            Vec2::new(-sign.x, 0.)
        } else {
            Vec2::new(0., -sign.y)
        };

        let near_time: f32 = near_time.component_max();
        let far_time: f32 = far_time.component_min();

        if (near_time >= 1.) || (far_time <= 0.) {
            return None;
        }

        Some(HitInfo {
            pos: pos + far_time * delta,
            normal,
            time: far_time,
        })
    }

    pub fn intersects_with_aabb(&self, aabb: &Self) -> bool {
        let d = (aabb.center() - self.center()).abs();
        let p = (aabb.half_extents() + self.half_extents()) - d;

        // If p is positive (or 0) in a component, then the boxes overlap along that axis
        // We need all axes to overlap to consider this an intersection
        p.x >= 0. && p.y >= 0.
    }

    pub fn intersects_with_aabb_sweep(&self, aabb: &Self, sweep: Vec2) -> Option<HitInfo> {
        // For small enough sweeps, we know they won't collide and can skip the sweep
        if sweep.as_array() == &[0., 0.] {
            // if self.intersects_with_aabb(aabb) {
            //     let normal = (aabb.center() - self.center()).normalized();
            //     return Some(HitInfo {
            //         pos: self.center(),
            //         normal,
            //         time: 0.,
            //     });
            // }

            return None;
        }

        self.intersects_with_line(aabb.center(), sweep, Some(aabb.half_extents()))
    }
}

pub fn random_direction() -> Vec2 {
    use rand::prelude::*;

    // Random angle from (0, π) - this is the hemisphere facing up in the simulation
    let t: f32 = rand::thread_rng().gen();
    let θ: f32 = std::f32::consts::PI * t;

    Vec2::new(f32::cos(θ), f32::sin(θ))
}
