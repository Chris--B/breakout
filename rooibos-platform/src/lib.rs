#[cfg(target_vendor = "apple")]
mod apple {
    use core::ffi::CStr;
    use metal::{MTLCaptureManager, MTLDevice};

    extern "C" {
        pub fn GpuCaptureManager_start(
            capture_manager: *const MTLCaptureManager,
            device: *const MTLDevice,
            trace_url: &CStr,
        ) -> bool;
        pub fn GpuCaptureManager_stop(capture_manager: *const MTLCaptureManager);
    }
}

pub mod prelude {
    #[cfg(target_vendor = "apple")]
    pub use crate::apple::*;
}
