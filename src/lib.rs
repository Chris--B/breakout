#![allow(dead_code)] // It's a fresh project and this isn't helpful
#![allow(mixed_script_confusables)] // Hell yeah, math!
#![allow(clippy::nonminimal_bool)] // The compiler can reduce this, let me write it for humans

use fermium::prelude::*;
use legion::*;
use ultraviolet::{Vec2, Vec3};

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;

mod ecs;
mod gfx;
mod math;

use ecs::*;
use math::*;

embed_plist::embed_info_plist!("../Info.plist");

fn poll_event() -> Option<SDL_Event> {
    let mut e = SDL_Event::default();
    if unsafe { SDL_PollEvent(&mut e) == 1 } {
        Some(e)
    } else {
        None
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

static BALL_COUNT: AtomicUsize = AtomicUsize::new(0);

fn ball_count() -> usize {
    BALL_COUNT.load(SeqCst)
}

fn new_ball(world: &mut World, ball_pos: Vec2) -> Entity {
    let count = BALL_COUNT.fetch_add(1, SeqCst);

    world.push((
        Name(format!("Ball-#{count}")),
        Position(ball_pos),
        Velocity(135. * random_direction()),
        HitableBall { radius: 1. },
        DrawableColoredBall {
            radius: 1.,
            color: color::WHITE,
        },
    ))
}

pub fn app_main() {
    let window_width: i32 = 500;
    let window_height: i32 = 750;

    let window = gfx::Window::new(window_width, window_height);
    let mut gpu = gfx::GpuDevice::new(&window);
    let mut world = World::default();

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

            world.push((
                Name(format!("Brick@({pos_x}, {pos_y})")),
                Position(pos),
                HitableQuad { dims },
                DrawableColoredQuad { dims, color },
                Breakable,
            ));
        }
    }

    // Add a user-controlled paddle
    let paddle_pos = Vec2::new(0.5 * view_x - dims.x / 2., 0.05 * view_y);
    let paddle_dims = dims;
    world.push((
        Name("Paddle".to_string()),
        Position(paddle_pos),
        HitableQuad { dims: paddle_dims },
        DrawableColoredQuad {
            dims: paddle_dims,
            color: color::WHITE,
        },
        Paddle,
    ));

    // Spawn a starter ball
    let init_ball_pos = paddle_pos + Vec2::new(0.5 * paddle_dims.x - 0.5, 3. * paddle_dims.y);
    new_ball(&mut world, init_ball_pos);

    // Hoisted out of the loop to reuse the allocation
    let mut quads = Vec::with_capacity(world.len());

    let mut paused = false;
    window.show();

    let mut capture: Option<gfx::GpuCaptureManager> = None;

    'main_loop: loop {
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

                        keycode::SDLK_t if key.repeat == 0 => {
                            assert!(capture.is_none());
                            capture = gpu.prepare_capture();
                        }

                        keycode::SDLK_SPACE if key.repeat == 0 => {
                            // Toggle the simulation update when SPACE is pressed
                            paused = !paused;
                        }

                        keycode::SDLK_b => {
                            // Spawn a ball when "B" is pressed
                            new_ball(&mut world, init_ball_pos);
                        }

                        keycode::SDLK_c => {
                            // Clear all balls when "C" is pressed
                            let mut query = <(Entity, &HitableBall)>::query();
                            let balls: Vec<_> = query.iter(&world).map(|(e, _)| *e).collect();
                            let ball_count = balls.len();
                            for ball in balls {
                                world.remove(ball);
                            }

                            BALL_COUNT.store(0, SeqCst);

                            if ball_count > 0 {
                                println!("Removed {ball_count} balls");
                            }
                        }

                        _ => {}
                    }
                }

                // On tap or drag, spawn a ball!
                SDL_FINGERDOWN | SDL_FINGERMOTION => {
                    let _tfinger: SDL_TouchFingerEvent = unsafe { e.tfinger };

                    new_ball(&mut world, init_ball_pos);
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
            let mut out_of_bounds: Vec<Entity> = vec![];

            // Advance anything with velocity
            let mut query = <(Entity, &mut Position, &mut Velocity)>::query();
            for (entity, Position(pos), Velocity(vel)) in query.iter_mut(&mut world) {
                *pos += dt * *vel;

                if !(0. < pos.x && pos.x < view_x) || !(0. < pos.y && pos.y < view_y) {
                    out_of_bounds.push(*entity);
                }
            }

            // Check if any of the balls hit a brick
            let mut ball_query = <(Entity, &Position, &HitableBall)>::query();
            let mut bricks_query = <(Entity, &Position, &HitableQuad, Option<&Breakable>)>::query();

            let mut needs_to_break: Vec<Entity> = vec![];
            let mut needs_to_bounce: Vec<(Entity, Bounce)> = vec![];

            for (ball, Position(ball_pos), HitableBall { radius }) in ball_query.iter(&world) {
                let radius_sq = radius * radius;

                for (brick, Position(brick_pos), hitable, maybe_breakable) in
                    bricks_query.iter(&world)
                {
                    let aabb = Aabb::new_from_quad(*brick_pos, hitable.dims);
                    let center = aabb.center();
                    let extents = aabb.half_extents();

                    let dist_clamped = (*ball_pos - center).clamped(-extents, extents);
                    let closest_on_or_in_aabb = center + dist_clamped;

                    if (closest_on_or_in_aabb - *ball_pos).mag_sq() < radius_sq {
                        if maybe_breakable.is_some() {
                            needs_to_break.push(*brick);
                        }

                        let x_delta;
                        let y_delta;

                        if ball_pos.x <= aabb.min.x {
                            x_delta = ball_pos.x - aabb.min.x;
                        } else if ball_pos.x >= aabb.max.x {
                            x_delta = ball_pos.x - aabb.max.x;
                        } else {
                            x_delta = *radius;
                        }

                        if ball_pos.y <= aabb.min.y {
                            y_delta = ball_pos.y - aabb.min.y;
                        } else if ball_pos.y >= aabb.max.y {
                            y_delta = ball_pos.y - aabb.max.y;
                        } else {
                            y_delta = *radius;
                        }

                        let normal = if x_delta.abs() < y_delta.abs() {
                            Vec2::new(sign(x_delta), 0.)
                        } else if y_delta.abs() < x_delta.abs() {
                            Vec2::new(0., sign(y_delta))
                        } else {
                            Vec2::new(-1., -1.).normalized()
                        };

                        needs_to_bounce.push((*ball, Bounce(normal)));
                    }
                }
            }

            // Remove anything that broke
            for breakable in needs_to_break {
                world.remove(breakable);
            }

            // Bounce anything that needs to bounce
            {
                // TODO: This can't be the best way to do this...
                // Add component to track bounces
                for (entity, bounce) in &needs_to_bounce {
                    let mut entry = world.entry(*entity).unwrap();
                    entry.add_component(*bounce);
                }

                // Update velocity for anything with our Bounce component
                let mut bounce_query = <(&mut Velocity, &Bounce)>::query();
                for (Velocity(vel), Bounce(bounce)) in bounce_query.iter_mut(&mut world) {
                    *vel = vel.reflected(*bounce);
                }

                // Remove the bounce components
                for (entity, _bounce) in needs_to_bounce {
                    let mut entry = world.entry(entity).unwrap();
                    entry.remove_component::<Bounce>();
                }
            }

            // Remove anything out of bounds
            for entity in out_of_bounds {
                // let entry = world.entry(entity).unwrap();
                // if let Ok(name) = entry.get_component::<Name>() {
                //     println!("Removing out of bounds entity \"{name}\"");
                // }

                world.remove(entity);
            }
        }

        // Render
        if let Some(c) = &mut capture {
            c.start();
        }
        use gfx::shaders::PerQuad;

        // Draw Quads
        {
            let mut query = <(&Position, &DrawableColoredQuad)>::query();
            for (Position(pos), drawable) in query.iter(&world) {
                quads.push(PerQuad {
                    pos: *pos,
                    dims: drawable.dims,
                    color: drawable.color,
                });
            }
        }

        // Draw Balls
        {
            let mut query = <(&Position, &DrawableColoredBall)>::query();
            for (Position(pos), drawable) in query.iter(&world) {
                quads.push(PerQuad {
                    pos: *pos,
                    dims: Vec2::new(drawable.radius, drawable.radius),
                    color: drawable.color,
                });
            }
        }

        gpu.render_and_present(&quads);
        quads.clear();

        if let Some(mut c) = capture.take() {
            c.frames_left -= 1;

            if c.frames_left != 0 {
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
