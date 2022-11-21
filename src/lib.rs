#![allow(dead_code)] // It's a fresh project and this isn't helpful
#![allow(mixed_script_confusables)] // Hell yeah, math!
#![allow(clippy::nonminimal_bool)] // The compiler can reduce this, let me write it for humans

use fermium::prelude::*;
use ultraviolet::{Vec2, Vec3};

mod gfx;

mod math;
use math::*;

embed_plist::embed_info_plist!("../Info.plist");

#[derive(Clone, Default)]
struct World {
    balls: Vec<Ball>,
    bricks: Vec<Quad>,
    paddle: Quad,
}

#[derive(Copy, Clone, Debug, Default)]
struct Quad {
    pos: Vec2,
    vel: Vec2,
    dims: Vec2,
    color: Vec3,
}

#[derive(Copy, Clone, Debug, Default)]
struct Ball {
    pos: Vec2,
    vel: Vec2,
    radius: f32,
}

impl World {
    fn reset(&mut self) {
        self.balls.clear();
        self.bricks.clear();
        self.paddle = Default::default();
    }

    fn create_ball(&mut self, pos: Vec2) {
        self.balls.push(Ball {
            pos,
            vel: 135. * random_direction(),
            radius: 1.,
        });
    }
}

mod color {
    use super::*;

    pub const WHITE: Vec3 = Vec3::new(0.84, 0.84, 0.84);

    pub const RED: Vec3 = Vec3::new(0.60, 0., 0.);
    pub const ORANGE: Vec3 = Vec3::new(0.84, 0.60, 0.);
    pub const GREEN: Vec3 = Vec3::new(0., 0.60, 0.);
    pub const YELLOW: Vec3 = Vec3::new(0.80, 0.80, 0.);

    pub const OHNO_PINK: Vec3 = Vec3::new(1., 0., 1.);
}

fn poll_event() -> Option<SDL_Event> {
    let mut e = SDL_Event::default();
    if unsafe { SDL_PollEvent(&mut e) == 1 } {
        Some(e)
    } else {
        None
    }
}

pub fn app_main() {
    let window_width: i32 = 500;
    let window_height: i32 = 750;

    let window = gfx::Window::new(window_width, window_height);
    let mut gpu = gfx::GpuDevice::new(&window);
    let mut world = World::default();
    let mut next = World::default();

    // Shape of a brick & the paddle
    let dims = Vec2::new(5., 1.);

    // for the board
    let view_x = (dims.x + 1.) * 14. + 1.;
    let view_y = view_x * (window_height as f32 / window_width as f32);
    gpu.set_view(view_x, view_y);

    // (x, y) are position in the grid
    for y in 0..55 {
        let color: Vec3 = match y {
            0..=1 => color::RED,
            2..=3 => color::ORANGE,
            4..=5 => color::GREEN,
            6..=7 => color::YELLOW,
            _ => color::OHNO_PINK,
        };
        for x in 0..14 {
            // Note: Our x coordinate here must match the calculation© for view_x above
            let pos_x = (dims.x + 1.) * (x as f32) + 1.;
            let pos_y = view_y - (dims.y + 1.) * (y as f32 + 1.);
            let pos = Vec2::new(pos_x, pos_y);

            world.bricks.push(Quad {
                pos,
                vel: Vec2::zero(),
                dims,
                color,
            });
        }
    }

    // Add a user-controlled paddle
    let paddle_pos = Vec2::new(0.5 * view_x - dims.x / 2., 0.05 * view_y);
    let paddle_dims = dims;
    world.paddle = Quad {
        pos: paddle_pos,
        vel: Vec2::zero(),
        dims: paddle_dims,
        color: color::WHITE,
    };

    // Spawn a starter ball
    let init_ball_pos = paddle_pos + Vec2::new(0.5 * paddle_dims.x - 0.5, 3. * paddle_dims.y);
    world.create_ball(init_ball_pos);

    // Hoisted out of the loop to reuse the allocation
    let mut quads = Vec::with_capacity(
        world.bricks.len() + world.bricks.len() + 1, /* world.paddle */
    );

    let mut paused = false;
    window.show();

    let mut capture: Option<gfx::GpuCapture> = None;

    let mut pressed_left = false;
    let mut pressed_right = false;

    'main_loop: loop {
        let mut paddle_x_vel = 0.;

        // Handle events
        while let Some(e) = poll_event() {
            // Access to unions is unsafe, so this match block is going to get spicy
            let type_ = unsafe { e.type_ };
            match type_ {
                // Immediately quit everything - unhandled events are forever ignored
                SDL_QUIT => {
                    break 'main_loop;
                }

                SDL_KEYDOWN => {
                    let key = unsafe { e.key };

                    match key.keysym.sym {
                        keycode::SDLK_q => {
                            // Quit the app when "Q" is pressed
                            break 'main_loop;
                        }

                        keycode::SDLK_SPACE if key.repeat == 0 => {
                            // Toggle the simulation update when SPACE is pressed
                            paused = !paused;
                        }

                        keycode::SDLK_b => {
                            // Spawn a ball on the paddle when "B" is pressed
                            let pos = world.paddle.pos
                                + Vec2::new(0.5 * paddle_dims.x - 0.5, 3. * paddle_dims.y);

                            world.create_ball(pos);
                        }

                        keycode::SDLK_c => {
                            // Clear all balls when "C" is pressed
                            let ball_count = world.balls.len();
                            world.balls.clear();
                            println!("Removed {ball_count} balls");
                        }

                        keycode::SDLK_LEFT if key.repeat == 0 => {
                            pressed_left = true;
                        }

                        keycode::SDLK_RIGHT if key.repeat == 0 => {
                            pressed_right = true;
                        }

                        _ => {}
                    }
                }

                SDL_KEYUP => {
                    let key = unsafe { e.key };

                    match key.keysym.sym {
                        keycode::SDLK_LEFT => {
                            pressed_left = false;
                        }

                        keycode::SDLK_RIGHT => {
                            pressed_right = false;
                        }

                        keycode::SDLK_t if key.repeat == 0 => {
                            assert!(capture.is_none());
                            capture = gpu.prepare_capture();
                        }

                        _ => {}
                    }
                }
                // On tap or drag, spawn a ball!
                SDL_FINGERDOWN | SDL_FINGERMOTION => {
                    let _tfinger: SDL_TouchFingerEvent = unsafe { e.tfinger };

                    world.create_ball(init_ball_pos);
                }

                // Ignore all other events
                _ => {}
            }
        }

        // Update gamestate
        const DELAY_MS: u32 = 5;
        let dt = (DELAY_MS as f32) * 1e-3;

        // Advance the simulation
        if !paused {
            // Update movement from events - this skips the OS keyboard delay
            const PADDLE_X_VEL: f32 = 400.;
            if pressed_left {
                paddle_x_vel -= PADDLE_X_VEL;
            }
            if pressed_right {
                paddle_x_vel += PADDLE_X_VEL;
            }

            // Update the paddle
            {
                next.paddle = world.paddle;

                // Update movement
                next.paddle.pos.x = (world.paddle.pos.x + dt * paddle_x_vel)
                    // Keep the paddle in bounds
                    .clamp(0., view_x - paddle_dims.x);
                // The paddle only slides left & right, so don't modify pos.y
                next.paddle.pos.y = world.paddle.pos.y;
            }

            // Update bricks by checking if a ball has hit them
            // Update ball velocities by checking if they hit a brick OR the paddle -- IN PLACE
            {
                fn bounce_against_quad(ball: &mut Ball, brick: &Quad) -> bool {
                    let radius_sq = ball.radius * ball.radius;

                    let aabb = Aabb::new_from_quad(brick.pos, brick.dims);
                    let center = aabb.center();
                    let extents = aabb.half_extents();

                    let dist_clamped = (ball.pos - center).clamped(-extents, extents);
                    let closest_on_or_in_aabb = center + dist_clamped;

                    if (closest_on_or_in_aabb - ball.pos).mag_sq() < radius_sq {
                        // TODO: Compute bounce on the ball

                        let x_delta;
                        let y_delta;

                        if ball.pos.x <= aabb.min.x {
                            x_delta = ball.pos.x - aabb.min.x;
                        } else if ball.pos.x >= aabb.max.x {
                            x_delta = ball.pos.x - aabb.max.x;
                        } else {
                            x_delta = ball.radius;
                        }

                        if ball.pos.y <= aabb.min.y {
                            y_delta = ball.pos.y - aabb.min.y;
                        } else if ball.pos.y >= aabb.max.y {
                            y_delta = ball.pos.y - aabb.max.y;
                        } else {
                            y_delta = ball.radius;
                        }

                        let normal = if x_delta.abs() < y_delta.abs() {
                            Vec2::new(sign(x_delta), 0.)
                        } else if y_delta.abs() < x_delta.abs() {
                            Vec2::new(0., sign(y_delta))
                        } else {
                            Vec2::new(-1., -1.).normalized()
                        };

                        ball.vel = ball.vel.reflected(normal);

                        true
                    } else {
                        false
                    }
                }

                for brick in &world.bricks {
                    let mut brick_breaks = false;

                    for ball in &mut world.balls {
                        // If a ball hit this brick, then it will break

                        brick_breaks |= bounce_against_quad(ball, brick);
                    }

                    // If no ball hit this brick, then we delete it (by omission)
                    if !brick_breaks {
                        next.bricks.push(*brick);
                    }
                }

                for ball in &mut world.balls {
                    bounce_against_quad(ball, &world.paddle);
                }
            }

            // Update all balls' position from velocity
            {
                for ball in &world.balls {
                    let mut next_ball = *ball;
                    let Ball { pos, vel, .. } = *ball;

                    // Basic physics step
                    next_ball.pos = pos + dt * vel;

                    // If it's still in bounds, copy it to the next frameq
                    // (TODO: include radius in this math)
                    if (0. < pos.x && pos.x < view_x) && (0. < pos.y && pos.y < view_y) {
                        next.balls.push(next_ball);
                    }
                }
            }

            std::mem::swap(&mut world, &mut next);
            next.reset();
        }

        // Render
        use gfx::shaders::PerQuad;

        // Draw Quads
        {
            for ball in &world.balls {
                quads.push(PerQuad {
                    pos: ball.pos,
                    dims: Vec2::new(ball.radius, ball.radius),
                    color: color::WHITE,
                });
            }

            for brick in &world.bricks {
                quads.push(PerQuad {
                    pos: brick.pos,
                    dims: brick.dims,
                    color: brick.color,
                });
            }

            quads.push(PerQuad {
                pos: world.paddle.pos,
                dims: world.paddle.dims,
                color: color::WHITE,
            });
        }

        gpu.render_and_present(&quads);
        quads.clear();

        if let Some(mut c) = capture.take() {
            c.mark_frame_done();

            if c.frames_left() != 0 {
                // oops put it back
                capture = Some(c);
            } else {
                // Pause things, since we're about to switch to viewing the trace
                paused = true;

                // Finish and view the trace
                c.stop();
            }
        }

        // TODO: Better delay
        unsafe {
            SDL_Delay(DELAY_MS);
        }
    }

    unsafe {
        SDL_Quit();
    }
}
