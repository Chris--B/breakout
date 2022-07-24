use metal::*;

use fermium::prelude::*;

use ultraviolet::{Vec2, Vec3};

use std::os::raw::c_void;

mod gfx;

/// Returns true when there is not error. Think:
/// ```rust
/// let ok = check_sdl_error("SDL_Foo");
/// ```
fn check_sdl_error(func: &str) -> bool {
    // We can't use `c_char` in literals, we HAVE to cast
    #![allow(clippy::unnecessary_cast)]

    use fermium::prelude::*;
    use std::ffi::CStr;

    unsafe {
        let mut msg_buf = [0 as c_char; 512];
        SDL_GetErrorMsg(&mut msg_buf as *mut c_char, msg_buf.len() as i32);

        // If the buffer stays empty, there's nothing to display
        if msg_buf[0] == b'\0' as c_char {
            return true;
        }

        // Otherwise, print the error message as a c string
        let msg = CStr::from_ptr(msg_buf.as_ptr());
        let msg = msg.to_str().unwrap();
        println!();
        println!("**********************************************************************");
        println!("*** {func}: {msg}");
        println!("**********************************************************************");
        println!();

        // And clear the error since we're done with it
        SDL_ClearError();

        false
    }
}

fn poll_event() -> Option<SDL_Event> {
    let mut e = SDL_Event::default();
    if unsafe { SDL_PollEvent(&mut e) == 1 } {
        Some(e)
    } else {
        None
    }
}

fn main() {
    let window_width: i32 = 1_000;
    let window_height: i32 = 1_000;

    // gfx init
    let device = Device::system_default().unwrap();
    gfx::print_device_info(&device);

    // SDL init
    let metal_layer: MetalLayer;
    let p_window: *mut SDL_Window;
    let p_renderer: *mut SDL_Renderer;
    let p_swapchain: *mut metal::CAMetalLayer;
    unsafe {
        use cstr::cstr;
        use foreign_types_shared::ForeignType;
        use std::ffi::CStr;

        let hint_render_driver: &CStr =
            CStr::from_ptr(std::mem::transmute(SDL_HINT_RENDER_DRIVER.as_ptr()));
        SDL_SetHint(hint_render_driver.as_ptr(), cstr!("metal").as_ptr());
        check_sdl_error("SDL_SetHint");

        SDL_Init(SDL_INIT_VIDEO | SDL_INIT_EVENTS);
        check_sdl_error("SDL_Init");

        p_window = SDL_CreateWindow(
            cstr!("Breakout!").as_ptr(),
            SDL_WINDOWPOS_CENTERED,
            SDL_WINDOWPOS_CENTERED,
            window_width,
            window_height,
            (SDL_WINDOW_ALLOW_HIGHDPI | SDL_WINDOW_METAL | SDL_WINDOW_RESIZABLE).0,
        );
        check_sdl_error("SDL_CreateWindow");
        assert_ne!(p_window, std::ptr::null_mut());

        println!("SDL Video Drivers:");
        for idx in 0..SDL_GetNumVideoDrivers() {
            let p_name: *const c_char = SDL_GetVideoDriver(idx);
            let name = CStr::from_ptr(std::mem::transmute(p_name));
            let name = name.to_str().unwrap();
            println!("  + {name}",);
        }
        println!();

        println!("SDL Render Drivers:");
        for idx in 0..SDL_GetNumRenderDrivers() {
            let mut info = SDL_RendererInfo::default();
            SDL_GetRenderDriverInfo(idx, &mut info);

            let name = CStr::from_ptr(info.name);
            let name = name.to_str().unwrap();

            print!("  + {name:<15} ");

            let flags: SDL_RendererFlags = std::mem::transmute(info.flags);
            if SDL_RendererFlags(0) != flags & SDL_RENDERER_ACCELERATED {
                print!("ACCELERATED   ");
            } else {
                print!("              ");
            }
            if SDL_RendererFlags(0) != flags & SDL_RENDERER_PRESENTVSYNC {
                print!("PRESENTVSYNC  ");
            } else {
                print!("              ");
            }
            if SDL_RendererFlags(0) != flags & SDL_RENDERER_SOFTWARE {
                print!("SOFTWARE      ");
            } else {
                print!("              ");
            }
            if SDL_RendererFlags(0) != flags & SDL_RENDERER_TARGETTEXTURE {
                print!("TARGETTEXTURE ");
            } else {
                print!("              ");
            }
            println!();
        }
        println!();

        p_renderer = SDL_CreateRenderer(p_window, -1, 0);
        check_sdl_error("SDL_CreateRenderer");
        assert_ne!(p_renderer, std::ptr::null_mut());
        SDL_SetRenderDrawColor(p_renderer, 0, 0, 0, 255);

        p_swapchain = SDL_RenderGetMetalLayer(p_renderer) as *mut _;
        check_sdl_error("SDL_RenderGetMetalLayer");
        assert_ne!(p_swapchain, std::ptr::null_mut());

        metal_layer = MetalLayer::from_ptr(p_swapchain);
    }

    metal_layer.set_device(&device);
    metal_layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
    metal_layer.set_framebuffer_only(true);

    // Create Pipeline State
    let pipeline_state: RenderPipelineState;
    {
        let render_pipeline_state_desc = RenderPipelineDescriptor::new();

        let default_lib = device
            .new_library_with_data(gfx::shaders::SHADERS_BIN)
            .unwrap();
        let func_vs = default_lib.get_function("vs_instanced_quad", None).unwrap();
        render_pipeline_state_desc.set_vertex_function(Some(&func_vs));

        let func_fs = default_lib.get_function("fs_instanced_quad", None).unwrap();
        render_pipeline_state_desc.set_fragment_function(Some(&func_fs));

        let color_attachment = render_pipeline_state_desc
            .color_attachments()
            .object_at(0)
            .unwrap();
        color_attachment.set_pixel_format(MTLPixelFormat::BGRA8Unorm);

        pipeline_state = device
            .new_render_pipeline_state(&render_pipeline_state_desc)
            .unwrap();
    }

    // Acquire drawable surface
    let drawable = metal_layer.next_drawable().unwrap();

    let cmd_queue = device.new_command_queue();
    let cmd_buffer = cmd_queue.new_command_buffer();

    // Create Encoder
    let encoder: &RenderCommandEncoderRef;
    {
        let render_pass_desc = RenderPassDescriptor::new();

        let color_attachment = render_pass_desc.color_attachments().object_at(0).unwrap();
        color_attachment.set_texture(Some(drawable.texture()));
        color_attachment.set_load_action(MTLLoadAction::Clear);
        color_attachment.set_clear_color(MTLClearColor {
            red: 0.,
            green: 0.,
            blue: 0.,
            alpha: 1.,
        });

        encoder = cmd_buffer.new_render_command_encoder(render_pass_desc);
    }

    // Record Encoder
    {
        encoder.set_render_pipeline_state(&pipeline_state);

        let per_quad_data = vec![
            gfx::PerQuad {
                pos: Vec2::new(-0.8, 0.8),
                color: Vec3::new(1., 0., 0.),
                ..Default::default()
            },
            gfx::PerQuad {
                pos: Vec2::new(0.8, -0.8),
                color: Vec3::new(0., 1., 0.),
                ..Default::default()
            },
            gfx::PerQuad {
                pos: Vec2::new(-0.8, -0.8),
                color: Vec3::new(0., 0., 1.),
                ..Default::default()
            },
            gfx::PerQuad {
                pos: Vec2::new(0.8, 0.8),
                color: Vec3::new(0.65, 0., 1.00),
                ..Default::default()
            },
        ];
        let per_quad_buffer = device.new_buffer_with_data(
            per_quad_data.as_ptr() as *const c_void,
            (std::mem::size_of_val(&per_quad_data[0]) * per_quad_data.len()) as u64,
            MTLResourceOptions::empty(),
        );
        // 6 vertices per quad
        let tri_count = 6 * per_quad_data.len() as u64;

        const PER_QUAD_BUFFER_IDX: u64 = 2;
        encoder.set_vertex_buffer(PER_QUAD_BUFFER_IDX, Some(&per_quad_buffer), 0);
        encoder.draw_primitives(MTLPrimitiveType::Triangle, 0, tri_count);

        encoder.end_encoding();
    }

    cmd_buffer.present_drawable(drawable);
    cmd_buffer.commit();

    println!(
        "current_allocated_size = {}",
        device.current_allocated_size()
    );

    // Everything is initialized, let's kick off our main loop
    unsafe {
        SDL_ShowWindow(p_window);
        check_sdl_error("SDL_ShowWindow");
    }

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
            // TODO: Draw things

            SDL_Delay(100); // TODO: Delay better
        }
    }

    // Cleanup
    unsafe {
        SDL_DestroyWindow(p_window);
        SDL_Quit();
    }
}
