use metal::*;
use objc::*;

pub mod shaders {
    use static_assertions::{assert_eq_align, assert_eq_size};
    use ultraviolet::{Vec2, Vec3};

    pub const SHADERS_BIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/Shaders.metallib"));

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
        pub scale: Vec2,
        pub color: Vec3,
    }
    assert_eq_size!(PerQuad, [f32; 2 + 2 + 3]);
    assert_eq_align!(PerQuad, f32);

    impl Default for PerQuad {
        fn default() -> Self {
            Self {
                pos: Vec2::new(0., 0.),
                scale: Vec2::new(1., 1.),
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
