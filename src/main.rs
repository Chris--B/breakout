// It's a fresh project and this isn't helpful
#![allow(dead_code)]

use fermium::prelude::*;
use ultraviolet::{Vec2, Vec3};

mod gfx;
use gfx::shaders::PerQuad;

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

    let mut quads = vec![];

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

            quads.push(PerQuad { pos, color, dims });
        }
    }

    // paddle
    let paddle_pos = Vec2::new(0.5 * view_x - dims.x / 2., 0.05 * view_y);
    quads.push(PerQuad {
        pos: paddle_pos,
        color: color::WHITE,
        dims,
    });

    // ball
    let ball_pos = paddle_pos + Vec2::new(0.5 * dims.x - 0.5, 3. * dims.y);
    quads.push(PerQuad {
        pos: ball_pos,
        color: color::WHITE,
        dims: Vec2::new(1., 1.),
    });

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

        // Render
        unsafe {
            gpu.render_and_present(&quads);

            SDL_Delay(100); // TODO: Delay better
        }
    }

    unsafe {
        SDL_Quit();
    }
}
