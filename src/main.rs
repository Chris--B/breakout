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

const COLOR_RED: Vec3 = Vec3::new(1., 0., 0.);
const COLOR_GREEN: Vec3 = Vec3::new(0., 1., 0.);
const COLOR_BLUE: Vec3 = Vec3::new(0., 0., 1.);
const COLOR_PURPLE: Vec3 = Vec3::new(0.65, 0., 1.00);

fn main() {
    let window_width: i32 = 1_000;
    let window_height: i32 = 1_000;
    let window = gfx::Window::new(window_width, window_height);

    let gpu = gfx::GpuDevice::new(&window);

    let quads = vec![
        PerQuad {
            pos: Vec2::new(-0.8, 0.8),
            color: COLOR_RED,
            ..Default::default()
        },
        PerQuad {
            pos: Vec2::new(0.8, -0.8),
            color: COLOR_GREEN,
            ..Default::default()
        },
        PerQuad {
            pos: Vec2::new(-0.8, -0.8),
            color: COLOR_BLUE,
            ..Default::default()
        },
        PerQuad {
            pos: Vec2::new(0.8, 0.8),
            color: COLOR_PURPLE,
            ..Default::default()
        },
    ];

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
