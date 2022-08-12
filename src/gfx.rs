use fermium::prelude::*;
use metal::*;
use objc::*;

use ultraviolet::projection::lh_yup::orthographic_vk as orthographic;
use ultraviolet::{Mat4, Vec2, Vec3};

use std::os::raw::c_void;
use std::sync::Arc;

pub mod shaders {
    use super::*;
    use static_assertions::{assert_eq_align, assert_eq_size};

    pub const SHADERS_BIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/Shaders.metallib"));

    pub const BUFFER_IDX_VIEW: u64 = 1;
    pub const BUFFER_IDX_PER_QUAD: u64 = 2;

    #[repr(C, align(16))]
    #[derive(Copy, Clone, Debug)]
    pub struct View {
        pub mat_view_proj: Mat4,
    }
    assert_eq_size!(View, [f32; 16]);
    // We force alignment, and no native types have 16-byte alignment, so skip the assert
    // assert_eq_align!(View, X);

    impl Default for View {
        fn default() -> Self {
            Self {
                mat_view_proj: Mat4::identity(),
            }
        }
    }

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
    println!("    name                    = {}", device.name());
    println!("    class                   = {}", class_name(device));
    println!("    registry_id             = 0x{:x}", device.registry_id());
    if cfg!(target_os = "macos") {
        println!("    location                = {:?}", device.location());
        println!("    location_number         = {}", device.location_number());
    }
    println!();

    if cfg!(target_os = "macos") {
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
    }

    if cfg!(target_os = "macos") {
        println!(
            "    max_transfer_rate                = {}",
            device.max_transfer_rate()
        );
    }
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
        MTLGPUFamily::Common1,
        MTLGPUFamily::Common2,
        MTLGPUFamily::Common3,

        MTLGPUFamily::Mac2,

        MTLGPUFamily::Apple1,
        MTLGPUFamily::Apple2,
        MTLGPUFamily::Apple3,
        MTLGPUFamily::Apple4,
        MTLGPUFamily::Apple5,
        MTLGPUFamily::Apple6,
        MTLGPUFamily::Apple7,
        MTLGPUFamily::Apple8,
        MTLGPUFamily::Apple9,
    ];

    if let Some(family) = FAMILIES
        .iter()
        .copied()
        .rev()
        .find(|family| device.supports_family(*family))
    {
        println!("    supported family: {family:?}");
    } else {
        println!("    supported family: 🤷‍♀️");
    }

    println!("    supported  texture sample count:");
    for count in 1.. {
        println!(
            "      {}? {}",
            count,
            check_or_x(device.supports_texture_sample_count(count))
        );

        if !device.supports_texture_sample_count(count) {
            break;
        }
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

    unsafe {
        extern "C" {
            fn os_proc_available_memory() -> usize;
        }

        let bytes = os_proc_available_memory();
        if bytes == 0 {
            println!("    os_proc_available_memory()      No limit");
        } else {
            println!("    os_proc_available_memory()      {bytes} bytes");
        }
    }
}

/// Returns true when there is not error. Think:
/// ```rust,ignore
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

            let mut wm_info = SDL_SysWMinfo::default();
            SDL_VERSION(&mut wm_info.version);
            SDL_GetWindowWMInfo(p_window, &mut wm_info);

            let SDL_version {
                major,
                minor,
                patch,
            } = wm_info.version;
            println!("SDL Version: {major}.{minor}.{patch}");

            // Minor usability nits
            SDL_SetWindowMinimumSize(p_window, (3 * width) / 4, (3 * height) / 4);
            check_sdl_error("SDL_SetWindowMinimumSize");

            {
                use objc::runtime::Object;
                let subsystem = match wm_info.subsystem {
                    SDL_SYSWM_UNKNOWN => "SDL_SYSWM_UNKNOWN",
                    SDL_SYSWM_WINDOWS => "SDL_SYSWM_WINDOWS",
                    SDL_SYSWM_X11 => "SDL_SYSWM_X11",
                    SDL_SYSWM_DIRECTFB => "SDL_SYSWM_DIRECTFB",
                    SDL_SYSWM_COCOA => "SDL_SYSWM_COCOA",
                    SDL_SYSWM_UIKIT => "SDL_SYSWM_UIKIT",
                    SDL_SYSWM_WAYLAND => "SDL_SYSWM_WAYLAND",
                    SDL_SYSWM_MIR => "SDL_SYSWM_MIR",
                    SDL_SYSWM_WINRT => "SDL_SYSWM_WINRT",
                    SDL_SYSWM_ANDROID => "SDL_SYSWM_ANDROID",
                    SDL_SYSWM_VIVANTE => "SDL_SYSWM_VIVANTE",

                    _ => "SDL_SYSWM_UNKNOWN",
                };
                println!("SDL subsystem: {subsystem}");

                // We need to assign the return value so msg_send!() can infer the right types.
                // But there is no return value on a setter like this, so silence clippy's warning.
                #[allow(clippy::let_unit_value)]
                if wm_info.subsystem == SDL_SYSWM_COCOA {
                    // If we're using Cocoa, do some sketchy message sending to fix the aspect ratio on resize
                    #[repr(C)]
                    #[derive(Copy, Clone, Debug)]
                    struct NSSize {
                        width: f64,
                        height: f64,
                    }

                    let cocoa_window: &Object = &*(wm_info.info.cocoa.window as *const _);
                    let aspect_ratio = NSSize {
                        width: width as f64,
                        height: height as f64,
                    };
                    let _: () = msg_send![cocoa_window, setAspectRatio: aspect_ratio];
                } else {
                    // Other window managers are ignored, and resizing can look funny instead.
                    println!("SDL WM subsystem isn't cocoa, so we're not locking aspect ratio");
                }
            }

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

    view_width: f32,
    view_height: f32,
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

            view_width: 100.,
            view_height: 100.,
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

        if !quads.is_empty() {
            let encoder = cmd_buffer.new_render_command_encoder(render_pass_desc);
            encoder.set_render_pipeline_state(&self.pipeline_state);

            // TODO: Don't re-create buffers per-frame
            let view = shaders::View {
                // scale our dimensions by half because this expects Vk's system which is larger than ours
                // TODO: Don't do that.
                mat_view_proj: orthographic(
                    0.,                      // left
                    0.5 * self.view_width,   // right
                    0.,                      // bottom
                    -0.5 * self.view_height, // top
                    0.,                      // near
                    1.,                      // far
                ),
            };
            let view_buffer = self.device.new_buffer_with_data(
                &view as *const _ as *const c_void,
                std::mem::size_of_val(&view) as u64,
                MTLResourceOptions::empty(),
            );

            let quads_buffer = self.device.new_buffer_with_data(
                quads.as_ptr() as *const c_void,
                (std::mem::size_of_val(&quads[0]) * quads.len()) as u64,
                MTLResourceOptions::empty(),
            );

            encoder.set_vertex_buffer(shaders::BUFFER_IDX_VIEW, Some(&view_buffer), 0);
            encoder.set_vertex_buffer(shaders::BUFFER_IDX_PER_QUAD, Some(&quads_buffer), 0);

            // 6 vertices per quad
            let tri_count = 6 * quads.len() as u64;
            encoder.draw_primitives(MTLPrimitiveType::Triangle, 0, tri_count);

            encoder.end_encoding();
        }

        cmd_buffer.present_drawable(drawable);
        cmd_buffer.commit();
    }

    pub fn set_view(&mut self, width: f32, height: f32) {
        self.view_width = width;
        self.view_height = height;
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
