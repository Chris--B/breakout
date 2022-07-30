// It's a fresh project and this isn't helpful
#![allow(dead_code)]

use fermium::prelude::*;
use legion::*;
use ultraviolet::{Vec2, Vec3};

mod ecs;
mod gfx;

use ecs::*;

fn poll_event() -> Option<SDL_Event> {
    let mut e = SDL_Event::default();
    if unsafe { SDL_PollEvent(&mut e) == 1 } {
        Some(e)
    } else {
        None
    }
}

pub mod color {
    use super::*;

    pub const WHITE: Vec3 = Vec3::new(0.84, 0.84, 0.84);

    pub const RED: Vec3 = Vec3::new(0.60, 0., 0.);
    pub const ORANGE: Vec3 = Vec3::new(0.84, 0.60, 0.);
    pub const GREEN: Vec3 = Vec3::new(0., 0.60, 0.);
    pub const YELLOW: Vec3 = Vec3::new(0.80, 0.80, 0.);

    pub const OHNO_PINK: Vec3 = Vec3::new(1., 0., 1.);
}

fn main() {
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
            // Note: Our x coordinate here must match the calculation for view_x above
            let pos_x = (dims.x + 1.) * (x as f32) + 1.;
            let pos_y = view_y - (dims.y + 1.) * (y as f32 + 1.);
            let pos = Vec2::new(pos_x, pos_y);

            world.push((
                Name(format!("Brick@({pos_x}, {pos_y})")),
                Position(pos),
                HitableQuad { dims },
                DrawableColoredQuad { dims, color },
            ));
        }
    }

    // paddle
    let paddle_pos = Vec2::new(0.5 * view_x - dims.x / 2., 0.05 * view_y);
    let _paddle = world.push((
        Name("Paddle".to_string()),
        Position(paddle_pos),
        HitableQuad { dims },
        DrawableColoredQuad {
            dims,
            color: color::WHITE,
        },
        Paddle,
    ));

    // ball
    let ball_pos = paddle_pos + Vec2::new(0.5 * dims.x - 0.5, 3. * dims.y);
    let ball_dims = Vec2::new(1., 1.);
    let ball_count = 1;
    let _ball = world.push((
        Name(format!("Ball-#{ball_count}")),
        Position(ball_pos),
        Velocity(Vec2::new(1., 40.)),
        HitableQuad { dims: ball_dims },
        DrawableColoredQuad {
            dims: ball_dims,
            color: color::WHITE,
        },
        Ball,
    ));

    // Hoisted out of the loop to reuse the allocation
    let mut quads = Vec::with_capacity(world.len());

    window.show();

    'main_loop: loop {
        // Handle events
        while let Some(e) = poll_event() {
            // Access to unions is unsafe, so this match block is going to get spicy
            match unsafe { e.type_ } {
                // Immediately quit everything - unhandled events are forever ignored
                SDL_QUIT => {
                    break 'main_loop;
                }

                SDL_KEYDOWN | SDL_KEYUP => {
                    let key = unsafe { e.key };
                    if key.keysym.sym == SDLK_q {
                        break 'main_loop;
                    }
                }

                // Ignore unhandled events
                _ => {}
            }
        }

        // Update gamestate
        const DELAY_MS: u32 = 5;
        let dt = (DELAY_MS as f32) * 1e-3;
        {
            // Update ball positions
            {
                let mut query = <(&mut Position, &Velocity)>::query();
                for (Position(pos), Velocity(vel)) in query.iter_mut(&mut world) {
                    (*pos) += dt * *vel;
                }
            }
        }

        // Render
        use gfx::shaders::PerQuad;

        // Build quads for the renderer
        {
            let mut query = <(&Position, &DrawableColoredQuad)>::query();
            for (pos, drawable) in query.iter(&world) {
                quads.push(PerQuad {
                    pos: pos.0,
                    dims: drawable.dims,
                    color: drawable.color,
                });
            }

            gpu.render_and_present(&quads);
            quads.clear();
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
