use fermium::prelude::*;
use metal::*;
use objc::*;

use ultraviolet::{Vec2, Vec3};

use std::os::raw::c_void;
use std::sync::Arc;

pub mod shaders {
    use super::*;
    use static_assertions::{assert_eq_align, assert_eq_size};

    pub const SHADERS_BIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/Shaders.metallib"));

    pub const BUFFER_IDX_PER_QUAD: u64 = 2;

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct View {
        pub todo: f32,
    }
    assert_eq_size!(View, [f32; 1]);
    assert_eq_align!(View, f32);

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct PerQuad {
        pub pos: Vec2,
        pub dims: Vec2,
        pub color: Vec3,
    }
    assert_eq_size!(PerQuad, [f32; 2 + 2 + 3]);
    assert_eq_align!(PerQuad, f32);

    impl Default for PerQuad {
        fn default() -> Self {
            Self {
                pos: Vec2::new(0., 0.),
                dims: Vec2::new(1., 1.),
                color: Vec3::new(1., 0., 1.),
            }
        }
    }
}

fn check_or_x(p: bool) -> &'static str {
    if p {
        "✅"
    } else {
        "❌"
    }
}

fn class_name<C: objc::Message>(obj: &C) -> String {
    unsafe fn nsstring_as_str(nsstr: &objc::runtime::Object) -> &str {
        let bytes: *const std::os::raw::c_char = msg_send![nsstr, UTF8String];
        let len: NSUInteger = msg_send![nsstr, length];

        let utf8 = std::slice::from_raw_parts(bytes as *const u8, len as usize);

        std::str::from_utf8(utf8).unwrap()
    }

    unsafe {
        let x: &objc::runtime::Object = msg_send![obj, className];
        let s: String = nsstring_as_str(x).to_owned();

        s
    }
}

pub fn print_device_info(device: &DeviceRef) {
    println!("MTL Device Info");
    println!("    class                   = {}", class_name(device));
    println!("    registry_id             = 0x{:x}", device.registry_id());
    println!("    location                = {:?}", device.location());
    println!("    location_number         = {}", device.location_number());
    println!();

    println!(
        "    is_low_power            = {}",
        check_or_x(device.is_low_power())
    );
    println!(
        "    is_headless             = {}",
        check_or_x(device.is_headless())
    );
    println!(
        "    is_removable            = {}",
        check_or_x(device.is_removable())
    );
    println!(
        "    has_unified_memory      = {}",
        check_or_x(device.has_unified_memory())
    );
    println!();

    println!(
        "    max_transfer_rate                = {}",
        device.max_transfer_rate()
    );
    println!(
        "    max_threadgroup_memory_length    = {:?}",
        device.max_threadgroup_memory_length()
    );
    println!(
        "    max_threads_per_threadgroup      = {:?}",
        device.max_threads_per_threadgroup()
    );
    println!(
        "    recommended_max_working_set_size = {}",
        device.recommended_max_working_set_size()
    );
    println!();

    #[rustfmt::skip]
    const FAMILIES: &[MTLGPUFamily] = &[
        MTLGPUFamily::Apple1,
        MTLGPUFamily::Apple2,
        MTLGPUFamily::Apple3,
        MTLGPUFamily::Apple4,
        MTLGPUFamily::Apple5,
        MTLGPUFamily::Apple6,
        // Not exposed through bindings yet
        MTLGPUFamily::Apple7,
        MTLGPUFamily::Apple8,
        MTLGPUFamily::Apple9,
    ];

    let family = FAMILIES
        .iter()
        .copied()
        .rev()
        .find(|family| device.supports_family(*family))
        .unwrap();
    println!("    supported family: {family:?}");

    println!("    supported  texture sample count:");
    for count in 1.. {
        if !device.supports_texture_sample_count(count) {
            break;
        }

        println!(
            "      {}? {}",
            count,
            check_or_x(device.supports_texture_sample_count(count))
        );
    }
    println!();

    let b = check_or_x(device.supports_shader_barycentric_coordinates());
    println!("    shader barycentric coordinates? {}", b);

    let b = check_or_x(device.supports_function_pointers());
    println!("    function pointers?              {}", b);

    let b = check_or_x(device.supports_dynamic_libraries());
    println!("    dynamic libraries?              {}", b);

    let b = check_or_x(device.supports_raytracing());
    println!("    raytracing?                     {}", b);

    println!();
}

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

#[derive(Clone)]
pub struct Window(Arc<WindowImpl>);

pub struct WindowImpl {
    p_window: *mut SDL_Window,
    p_renderer: *mut SDL_Renderer,
}

impl Window {
    pub fn new(width: i32, height: i32) -> Self {
        use cstr::cstr;
        use std::ffi::CStr;

        unsafe {
            let hint_render_driver: &CStr =
                CStr::from_ptr(std::mem::transmute(SDL_HINT_RENDER_DRIVER.as_ptr()));
            SDL_SetHint(hint_render_driver.as_ptr(), cstr!("metal").as_ptr());
            check_sdl_error("SDL_SetHint");

            SDL_Init(SDL_INIT_VIDEO | SDL_INIT_EVENTS);
            check_sdl_error("SDL_Init");

            let p_window = SDL_CreateWindow(
                cstr!("Breakout!").as_ptr(),
                SDL_WINDOWPOS_CENTERED,
                SDL_WINDOWPOS_CENTERED,
                width,
                height,
                (SDL_WINDOW_ALLOW_HIGHDPI | SDL_WINDOW_METAL | SDL_WINDOW_RESIZABLE).0,
            );
            check_sdl_error("SDL_CreateWindow");
            assert_ne!(p_window, std::ptr::null_mut());

            let p_renderer = SDL_CreateRenderer(p_window, -1, 0);
            check_sdl_error("SDL_CreateRenderer");
            assert_ne!(p_renderer, std::ptr::null_mut());

            Self(Arc::new(WindowImpl {
                p_window,
                p_renderer,
            }))
        }
    }

    pub fn show(&self) {
        unsafe {
            SDL_ShowWindow(self.0.p_window);
            check_sdl_error("SDL_ShowWindow");
        }
    }
}

impl Window {
    fn get_metal_layer(&self) -> MetalLayer {
        use foreign_types_shared::ForeignType;

        unsafe {
            let p_metal_layer = SDL_RenderGetMetalLayer(self.0.p_renderer) as *mut _;
            check_sdl_error("SDL_RenderGetMetalLayer");
            assert_ne!(p_metal_layer, std::ptr::null_mut());

            MetalLayer::from_ptr(p_metal_layer)
        }
    }
}

pub struct GpuDevice {
    device: Device,
    cmd_queue: CommandQueue,

    // Pipeline state for {vs,fs}_instanced_quad
    pipeline_state: RenderPipelineState,

    metal_layer: MetalLayer,
    window: Window,
}

impl GpuDevice {
    pub fn new(window: &Window) -> Self {
        let window = window.clone();
        let device = Device::system_default().unwrap();
        print_device_info(&device);

        let metal_layer: MetalLayer = window.get_metal_layer();
        metal_layer.set_device(&device);
        metal_layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        metal_layer.set_framebuffer_only(true);

        // Create Pipeline State
        let pipeline_state: RenderPipelineState;
        {
            let render_pipeline_state_desc = RenderPipelineDescriptor::new();

            let default_lib = device.new_library_with_data(shaders::SHADERS_BIN).unwrap();
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

        let cmd_queue = device.new_command_queue();

        Self {
            device,
            cmd_queue,
            pipeline_state,

            metal_layer,
            window,
        }
    }

    pub fn render_and_present(&self, quads: &[shaders::PerQuad]) {
        let drawable = self.metal_layer.next_drawable().unwrap();
        let cmd_buffer = self.cmd_queue.new_command_buffer();

        // Create & record Encoder
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

        let encoder = cmd_buffer.new_render_command_encoder(render_pass_desc);
        encoder.set_render_pipeline_state(&self.pipeline_state);

        // TODO: Don't re-create a buffer per-frame
        let quads_buffer = self.device.new_buffer_with_data(
            quads.as_ptr() as *const c_void,
            (std::mem::size_of_val(&quads[0]) * quads.len()) as u64,
            MTLResourceOptions::empty(),
        );
        // 6 vertices per quad
        let tri_count = 6 * quads.len() as u64;

        encoder.set_vertex_buffer(shaders::BUFFER_IDX_PER_QUAD, Some(&quads_buffer), 0);
        encoder.draw_primitives(MTLPrimitiveType::Triangle, 0, tri_count);

        encoder.end_encoding();

        cmd_buffer.present_drawable(drawable);
        cmd_buffer.commit();
    }
}

impl Drop for GpuDevice {
    fn drop(&mut self) {
        // TODO: Shutdown correctly
        // unsafe {
        //     SDL_DestroyRenderer(self.p_renderer);
        //     SDL_DestroyWindow(self.p_window);
        // }
    }
}
