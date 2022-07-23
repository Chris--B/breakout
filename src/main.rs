use metal::*;

use fermium::prelude::*;

use std::os::raw::c_void;
use std::ptr;

mod gfx;

fn check_sdl_error(func: &str) {
    // We can't use `c_char` in literals, we HAVE to cast
    #![allow(clippy::unnecessary_cast)]

    use fermium::prelude::*;
    use std::ffi::CStr;

    unsafe {
        let mut msg_buf = [0 as c_char; 64];
        SDL_GetErrorMsg(&mut msg_buf as *mut c_char, msg_buf.len() as i32);

        // If the buffer stays empty, there's nothing to display
        if msg_buf[0] == b'\0' as c_char {
            return;
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
    }
}

fn main() {
    let window_width: i32 = 1_000;
    let window_height: i32 = 1_000;

    // gfx init
    let device = Device::system_default().unwrap();
    gfx::print_device_info(&device);

    let metal_layer: MetalLayer;
    let p_window: *mut SDL_Window;
    unsafe {
        use cstr::cstr;
        use foreign_types_shared::ForeignType;
        use std::ffi::CStr;

        let hint_render_driver: &CStr =
            CStr::from_ptr(std::mem::transmute(SDL_HINT_RENDER_DRIVER.as_ptr()));
        SDL_SetHint(hint_render_driver.as_ptr(), cstr!("metal").as_ptr());
        check_sdl_error("SDL_SetHint");

        SDL_InitSubSystem(SDL_INIT_VIDEO);
        check_sdl_error("SDL_InitSubSystem");

        p_window = SDL_CreateWindow(
            cstr!("Metal Sandbox").as_ptr(),
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

        let p_renderer = SDL_CreateRenderer(p_window, -1, 0);
        check_sdl_error("SDL_CreateRenderer");
        assert_ne!(p_renderer, std::ptr::null_mut());

        let p_swapchain: *mut metal::CAMetalLayer = SDL_RenderGetMetalLayer(p_renderer) as *mut _;
        check_sdl_error("SDL_RenderGetMetalLayer");
        assert_ne!(p_swapchain, std::ptr::null_mut());

        metal_layer = MetalLayer::from_ptr(p_swapchain);

        SDL_ShowWindow(p_window);
        check_sdl_error("SDL_ShowWindow");

        // TODO: We're leaking some pointers, we should fix that.
    }

    metal_layer.set_device(&device);
    metal_layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
    metal_layer.set_framebuffer_only(true);

    #[rustfmt::skip]
    let vertex_data = [
         0.,  1.,  0.,
        -1., -1.,  0.,
         1., -1.,  0.
    ];

    let vertex_buffer_size_in_bytes = std::mem::size_of_val(&vertex_data[0]) * vertex_data.len();
    let vertex_buffer = device.new_buffer_with_data(
        vertex_data.as_ptr() as *const c_void,
        vertex_buffer_size_in_bytes as u64,
        MTLResourceOptions::empty(),
    );

    let default_lib = device.new_library_with_file("Shaders.metallib").unwrap();
    let vertex_function = default_lib.get_function("basic_vertex", None).unwrap();
    let fragment_function = default_lib.get_function("basic_fragment", None).unwrap();

    // Create Pipeline State
    let pipeline_state: RenderPipelineState;
    {
        let render_pipeline_state_desc = RenderPipelineDescriptor::new();
        render_pipeline_state_desc.set_vertex_function(Some(&vertex_function));
        render_pipeline_state_desc.set_fragment_function(Some(&fragment_function));

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
            green: 104. / 255.,
            blue: 55. / 255.,
            alpha: 1.,
        });

        encoder = cmd_buffer.new_render_command_encoder(render_pass_desc);
    }

    // Record Encoder
    encoder.set_render_pipeline_state(&pipeline_state);
    encoder.set_vertex_buffer(0, Some(&vertex_buffer), 0);
    encoder.draw_primitives(MTLPrimitiveType::Triangle, 0, 3);
    encoder.end_encoding();

    cmd_buffer.present_drawable(drawable);
    cmd_buffer.commit();

    println!(
        "current_allocated_size = {}",
        device.current_allocated_size()
    );
}
