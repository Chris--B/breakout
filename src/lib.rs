#![allow(dead_code)] // It's a fresh project and this isn't helpful
#![allow(mixed_script_confusables)] // Hell yeah, math!

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

                SDL_KEYDOWN | SDL_KEYUP => {
                    let key = unsafe { e.key };

                    // Quit the app when "Q" is pressed
                    if key.keysym.sym == SDLK_q {
                        break 'main_loop;
                    }

                    // Toggle the simulation update when SPACE is pressed
                    if (type_ == SDL_KEYDOWN) && (key.repeat == 0) && (key.keysym.sym == SDLK_SPACE)
                    {
                        paused = !paused;
                    }

                    // Spawn a ball when "B" is pressed
                    if (type_ == SDL_KEYDOWN) && (key.keysym.sym == SDLK_b) {
                        new_ball(&mut world, init_ball_pos);
                    }

                    // Clear all balls when "C" is pressed
                    if (type_ == SDL_KEYDOWN) && (key.repeat == 0) && (key.keysym.sym == SDLK_c) {
                        let mut query = <(Entity, &HitableBall)>::query();
                        let balls: Vec<_> = query.iter(&world).map(|(e, _)| *e).collect();
                        let ball_count = balls.len();
                        for ball in balls {
                            world.remove(ball);
                        }

                        BALL_COUNT.store(0, SeqCst);

                        println!("Removed {ball_count} balls");
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
            // Advance anything with velocity
            let mut query = <(&mut Position, &mut Velocity)>::query();
            for (Position(pos), Velocity(vel)) in query.iter_mut(&mut world) {
                *pos += dt * *vel;
            }

            let mut ball_query = <(Entity, &Position, &Velocity, &HitableBall)>::query();
            let mut _bricks_query =
                <(Entity, &Position, &HitableQuad, Option<&Breakable>)>::query();

            let breakables_hit = vec![];

            for (_ball, Position(_ball_pos), Velocity(_ball_vel), HitableBall { radius: _ }) in
                ball_query.iter(&world)
            {
                // TODO: Ball-AABB intersectionb
            }

            // Remove anything that broke
            for breakable in breakables_hit {
                world.remove(breakable);
            }
        }

        // Render
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

        // TODO: Better delay
        unsafe {
            SDL_Delay(DELAY_MS);
        }
    }

    unsafe {
        SDL_Quit();
    }
}
