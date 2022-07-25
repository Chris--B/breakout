use fermium::prelude::*;

mod gfx;

fn poll_event() -> Option<SDL_Event> {
    let mut e = SDL_Event::default();
    if unsafe { SDL_PollEvent(&mut e) == 1 } {
        Some(e)
    } else {
        None
    }
}

fn main() {
    let gpu = gfx::GpuDevice::new();

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
            SDL_Delay(100); // TODO: Delay better
        }
    }

    unsafe {
        SDL_Quit();
    }
}
