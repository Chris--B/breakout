use ultraviolet::Vec2;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Aabb {
    pub min: Vec2,
    pub max: Vec2,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HitInfo {
    pub t: f32,
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

    pub fn contains_point(&self, point: Vec2) -> bool {
        for i in 0..2 {
            if (self.min[i] <= point[i]) && (point[i] <= self.max[i]) {
                continue;
            } else {
                return false;
            }
        }

        true
    }

    pub fn intersects_with_line(
        &self,
        origin: Vec2,
        dir: Vec2,
        padding: Option<Vec2>,
    ) -> Option<HitInfo> {
        let padding = padding.unwrap_or_default();
        let min = self.min - padding;
        let max = self.max + padding;

        let mut inside = true;

        const Q_RIGHT: f32 = 0.;
        const Q_LEFT: f32 = 1.;
        const Q_MIDDLE: f32 = 2.;

        let mut quadrant = Vec2::new(0., 0.);
        let mut planes = Vec2::new(0., 0.);

        for i in 0..2 {
            if origin[i] < min[i] {
                // check left quadrant
                quadrant[i] = Q_LEFT;
                planes[i] = min[i];
                inside = false;
            } else if origin[i] > max[i] {
                // check right quadrant
                quadrant[i] = Q_RIGHT;
                planes[i] = max[i];
                inside = false;
            } else {
                // check middle quadrant
                quadrant[i] = Q_MIDDLE;
            }
        }

        if inside {
            // It's already intersecting
            return Some(HitInfo {
                t: 0.,
                pos: origin,  // the hit location is the origin
                normal: -dir, // the normal is the opposite of the direction?
            });
        }

        // Compute t distances to each plane
        let mut t_max = Vec2::new(-1., -1.);
        for i in 0..2 {
            if quadrant[i] != Q_MIDDLE && dir[i] != 0. {
                t_max[i] = (planes[i] - origin[i]) / dir[i];
            } else {
                t_max[i] = -1.;
            }
        }

        // Final candidate:
        let t = t_max.component_max();

        let pos = origin + t * dir;
        let normal = planes.normalized(); // ???

        if !(Aabb { min, max }).contains_point(pos) {
            return None;
        }

        Some(HitInfo { t, pos, normal })
    }

    pub fn intersects_with_aabb(&self, aabb: &Self) -> bool {
        let d = (aabb.center() - self.center()).abs();
        let p = (aabb.half_extents() + self.half_extents()) - d;

        // If p is positive (or 0) in a component, then the boxes overlap along that axis
        // We need all axes to overlap to consider this an intersection
        p.x >= 0. && p.y >= 0.
    }

    pub fn intersects_with_aabb_sweep(&self, aabb: &Self, sweep: Vec2) -> Option<HitInfo> {
        // To perform a sweep, we'll pad out the other box with our half extents, and use a line test
        self.intersects_with_line(aabb.center(), sweep, Some(aabb.half_extents()))
    }
}

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

pub fn random_direction() -> Vec2 {
    use rand::prelude::*;

    // Random angle from (0, π) - this is the hemisphere facing up in the simulation
    let t: f32 = rand::thread_rng().gen();
    let θ: f32 = std::f32::consts::PI * t;

    Vec2::new(f32::cos(θ), f32::sin(θ))
}

#[cfg(test)]
mod t {
    use super::*;

    const UNIT_AABB: Aabb = Aabb {
        min: Vec2::new(-0.5, -0.5),
        max: Vec2::new(0.5, 0.5),
    };

    #[cfg(test)]
    mod check_intersects_with_line {
        use super::*;
        use pretty_assertions::assert_eq;

        // Lines from OUTSIDE

        #[test]
        fn check_x_axis_left_to_right() {
            // Line starting LEFT of the box and moving RIGHT
            assert_eq!(
                UNIT_AABB.intersects_with_line(Vec2::new(-1., 0.), Vec2::new(1., 0.), None),
                Some(HitInfo {
                    t: 0.5,
                    pos: Vec2::new(-0.5, 0.),
                    normal: Vec2::new(-1., 0.),
                }),
            );
        }

        #[test]
        fn check_x_axis_right_to_left() {
            // Line starting RIGHT of the box and moving LEFT
            assert_eq!(
                UNIT_AABB.intersects_with_line(Vec2::new(1., 0.), Vec2::new(-1., 0.), None),
                Some(HitInfo {
                    t: 0.5,
                    pos: Vec2::new(0.5, 0.),
                    normal: Vec2::new(1., 0.),
                }),
            );
        }

        #[test]
        fn check_y_axis_top_to_bottom() {
            // Line starting ABOVE the box and moving DOWN
            assert_eq!(
                UNIT_AABB.intersects_with_line(Vec2::new(0., 1.), Vec2::new(0., -1.), None),
                Some(HitInfo {
                    t: 0.5,
                    pos: Vec2::new(0., 0.5),
                    normal: Vec2::new(0., 1.),
                }),
            );
        }

        #[test]
        fn check_y_axis_bottom_to_top() {
            // Line starting BELOW the box and moving UP
            assert_eq!(
                UNIT_AABB.intersects_with_line(Vec2::new(0., -1.), Vec2::new(0., 1.), None),
                Some(HitInfo {
                    t: 0.5,
                    pos: Vec2::new(0., -0.5),
                    normal: Vec2::new(0., -1.),
                }),
            );
        }

        #[test]
        fn check_corner_hit() {
            // Line starting ABOVE and LEFT of the box, moving DOWN and to the RIGHT
            assert_eq!(
                UNIT_AABB.intersects_with_line(Vec2::new(-1., 1.), Vec2::new(1., -1.), None),
                Some(HitInfo {
                    t: 0.5,
                    pos: Vec2::new(-0.5, 0.5),
                    normal: Vec2::new(-1., 1.).normalized(),
                }),
            );
        }

        // Lines from INSIDE
        #[test]
        fn check_x_axis_origin_to_right() {
            // Line starting inside the box and moving RIGHT
            assert_eq!(
                UNIT_AABB.intersects_with_line(Vec2::new(0., 0.), Vec2::new(1., 0.), None),
                Some(HitInfo {
                    t: 0.,
                    pos: Vec2::new(0., 0.),
                    normal: -Vec2::new(1., 0.),
                }),
            );
        }

        #[test]
        fn check_x_axis_origin_to_left() {
            // Line starting inside the box and moving LEFT
            assert_eq!(
                UNIT_AABB.intersects_with_line(Vec2::new(0., 0.), Vec2::new(-1., 0.), None),
                Some(HitInfo {
                    t: 0.,
                    pos: Vec2::new(0., 0.),
                    normal: -Vec2::new(-1., 0.),
                }),
            );
        }

        #[test]
        fn check_y_axis_origin_to_bottom() {
            // Line starting inside the box and moving DOWN
            assert_eq!(
                UNIT_AABB.intersects_with_line(Vec2::new(0., 0.), Vec2::new(0., -1.), None),
                Some(HitInfo {
                    t: 0.,
                    pos: Vec2::new(0., 0.),
                    normal: -Vec2::new(0., -1.),
                }),
            );
        }

        #[test]
        fn check_y_axis_origin_to_top() {
            // Line starting inside the box and moving UP
            assert_eq!(
                UNIT_AABB.intersects_with_line(Vec2::new(0., 0.), Vec2::new(0., 1.), None),
                Some(HitInfo {
                    t: 0.,
                    pos: Vec2::new(0., 0.),
                    normal: -Vec2::new(0., 1.),
                }),
            );
        }
    }

    #[cfg(test)]
    mod check_intersects_with_line_and_padding {
        use super::*;
        use pretty_assertions::assert_eq;

        const PADDING: Vec2 = Vec2::new(0.25, 0.25);

        #[test]
        fn check_x_axis_left_to_right() {
            // Line starting LEFT of the box and moving RIGHT
            assert_eq!(
                UNIT_AABB.intersects_with_line(
                    Vec2::new(-1., 0.),
                    Vec2::new(1., 0.),
                    Some(PADDING)
                ),
                Some(HitInfo {
                    t: 0.25,
                    pos: Vec2::new(-0.75, 0.),
                    normal: Vec2::new(-1., 0.),
                }),
            );
        }

        #[test]
        fn check_x_axis_right_to_left() {
            // Line starting RIGHT of the box and moving LEFT
            assert_eq!(
                UNIT_AABB.intersects_with_line(
                    Vec2::new(1., 0.),
                    Vec2::new(-1., 0.),
                    Some(PADDING)
                ),
                Some(HitInfo {
                    t: 0.25,
                    pos: Vec2::new(0.75, 0.),
                    normal: Vec2::new(1., 0.),
                }),
            );
        }

        #[test]
        fn check_y_axis_top_to_bottom() {
            // Line starting ABOVE the box and moving DOWN
            assert_eq!(
                UNIT_AABB.intersects_with_line(
                    Vec2::new(0., 1.),
                    Vec2::new(0., -1.),
                    Some(PADDING)
                ),
                Some(HitInfo {
                    t: 0.25,
                    pos: Vec2::new(0., 0.75),
                    normal: Vec2::new(0., 1.),
                }),
            );
        }

        #[test]
        fn check_y_axis_bottom_to_top() {
            // Line starting BELOW the box and moving UP
            assert_eq!(
                UNIT_AABB.intersects_with_line(
                    Vec2::new(0., -1.),
                    Vec2::new(0., 1.),
                    Some(PADDING)
                ),
                Some(HitInfo {
                    t: 0.25,
                    pos: Vec2::new(0., -0.75),
                    normal: Vec2::new(0., -1.),
                }),
            );
        }

        #[test]
        fn check_corner_hit() {
            // Line starting ABOVE and LEFT of the box, moving DOWN and to the RIGHT
            assert_eq!(
                UNIT_AABB.intersects_with_line(
                    Vec2::new(-1., 1.),
                    Vec2::new(1., -1.),
                    Some(PADDING)
                ),
                Some(HitInfo {
                    t: 0.25,
                    pos: Vec2::new(-0.75, 0.75),
                    // Floating point rounding gets us here, so we have to type out this exactly
                    normal: Vec2::new(-0.7071068, 0.7071068).normalized(),
                }),
            );
        }
    }
}
