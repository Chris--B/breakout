use crate::check_sdl_error;

use fermium::prelude::*;
// use fermium::prelude::audio::*;

use static_assertions::*;

use std::fmt;
use std::sync::Arc;
use std::sync::Mutex;

pub trait Waveform {
    /// Produce the next samples and write them out to `out_samples`.
    fn next_samples(&mut self, out_samples: &mut [u16]);
}

#[derive(Copy, Clone, Debug)]
pub struct SawtoothWaveform {
    /// Counter
    pub t: u32,

    /// Samples per second
    pub sample_freq: u32,

    /// Waveforms per second
    pub wave_freq: u32,
}

impl SawtoothWaveform {
    pub fn new(sample_freq: u32, wave_freq: u32) -> Self {
        Self {
            t: 0,
            sample_freq,
            wave_freq,
        }
    }
}

impl Waveform for SawtoothWaveform {
    fn next_samples(&mut self, out_samples: &mut [u16]) {
        // The wave drops back to 0 after this many samples
        let wave_sample_width = self.sample_freq / self.wave_freq;

        // Each successive sample increases amplitude this much
        let step = (u16::MAX as u32) / wave_sample_width;

        for out in out_samples {
            *out = self.t as u16;
            self.t = self.t.wrapping_add(step);
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SquareWaveform {
    /// Counter
    pub t: u32,

    /// Samples per second
    pub sample_freq: u32,

    /// Waveforms per second
    pub wave_freq: u32,

    pub f: f32,
}

impl SquareWaveform {
    pub fn new(sample_freq: u32, wave_freq: u32) -> Self {
        Self {
            t: 0,
            sample_freq,
            wave_freq,
            f: 0.5,
        }
    }
}

impl Waveform for SquareWaveform {
    fn next_samples(&mut self, out_samples: &mut [u16]) {
        // The wave drops back to 0 after this many samples
        let wave_sample_width = self.sample_freq / self.wave_freq;

        for out in out_samples {
            let t = self.t % wave_sample_width;
            if t < (self.f * wave_sample_width as f32) as u32 {
                *out = 0;
            } else {
                *out = i16::MAX as u16;
            }
            self.t = self.t.wrapping_add(1);
        }
    }
}

unsafe extern "C" fn audio_callback(p_userdata: *mut c_void, p_stream: *mut u8, nbytes: i32) {
    use core::mem::{size_of, transmute, ManuallyDrop};

    let out_samples: &mut [u16];
    let player: ManuallyDrop<AudioPlayer>;
    unsafe {
        // Note: We're given the buffer length in BYTES
        let len: usize = nbytes as usize / size_of::<u16>();
        out_samples = core::slice::from_raw_parts_mut(p_stream as *mut u16, len);

        // Note: It's very important that we do not drop this Arc here. Doing so will drop the ref count
        // and cause the next audio_callback call to use freed memory
        player = ManuallyDrop::new(transmute(p_userdata));
    };

    player.audio_callback(out_samples);
}

#[derive(Clone)]
pub struct AudioPlayer(Arc<Mutex<AudioInner>>);

struct AudioInner {
    waveform: SquareWaveform,
    spec: AudioSpec,
}

impl AudioPlayer {
    pub fn new(sample_freq: u32, channels: u8, waveform: SquareWaveform) -> Self {
        assert_eq!(
            sample_freq, waveform.sample_freq,
            "Resampling is not supported yet - Waveform must use exact sample rate as Player"
        );

        let mut want = AudioSpec::new();
        want.freq = sample_freq as i32;
        want.format = AUDIO_U16;
        want.channels = channels;
        want.samples = 4_096;
        want.callback = Some(audio_callback);

        // Allocate our inner object here
        // We do this to get a valid userdata pointer (the Arc)
        // This is modified below to get a final and valid object
        let inner = Arc::new(Mutex::new(AudioInner {
            waveform,
            spec: AudioSpec::new(),
        }));

        unsafe {
            // Note! We clone here, so AudioPlayer MUST retrieve this in Drop
            assert_eq_size!(AudioPlayer, *mut c_void);
            want.userdata = core::mem::transmute(inner.clone());

            let mut have = AudioSpec::new();
            // TODO: Use SDL_OpenAudioDevice()
            SDL_OpenAudio(&mut *want, &mut *have);
            check_sdl_error("SDL_OpenAudio");

            println!("Final AudioSpec: {have:?}");

            // Save the final spec we're actually using
            inner.lock().unwrap().spec = have;

            AudioPlayer(inner)
        }
    }

    pub fn play(&self) {
        unsafe {
            SDL_PauseAudio(0);
        }
    }

    pub fn pause(&self) {
        unsafe {
            SDL_PauseAudio(1);
        }
    }

    pub fn update_waveform(&self, update: impl FnOnce(&mut SquareWaveform)) {
        let mut inner = self.0.lock().unwrap();
        update(&mut inner.waveform);
    }
}

impl AudioPlayer {
    fn audio_callback(&self, out_samples: &mut [u16]) {
        let mut inner = self.0.lock().unwrap();
        inner.waveform.next_samples(out_samples);
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        // Our audio_callback "owns" one of these ref counts.
        // We're tearing all of that down now, so reclaim it.
        unsafe {
            // Things don't work correctly without `Arc::as_ptr`
            Arc::decrement_strong_count(Arc::as_ptr(&self.0));
        }

        // At this point, the object being dropped is the only expected owner
        assert_eq!(
            Arc::strong_count(&self.0),
            1,
            "Unexpected third+ owner of AudioPlayer's inner Arc"
        );
    }
}

// Fermium doesn't supply Debug impls for some audio types, so we do that here

// SDL_AudioFormat has a Debug impl, but it prints the backing u16 verbatim.
#[derive(Copy, Clone)]
#[repr(transparent)]
struct AudioFormat(SDL_AudioFormat);

impl fmt::Debug for AudioFormat {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let format = self.0;
        fmt.debug_struct("AudioFormat")
            .field("is_signed", &SDL_AUDIO_ISSIGNED(format))
            .field("is_bigendian", &SDL_AUDIO_ISBIGENDIAN(format))
            .field("is_float", &SDL_AUDIO_ISFLOAT(format))
            .field("sample_bit_size", &SDL_AUDIO_BITSIZE(format))
            .finish()
    }
}

#[repr(transparent)]
struct AudioSpec(SDL_AudioSpec);

impl AudioSpec {
    pub fn new() -> Self {
        unsafe { core::mem::zeroed() }
    }
}

impl core::ops::Deref for AudioSpec {
    type Target = SDL_AudioSpec;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for AudioSpec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Clone for AudioSpec {
    fn clone(&self) -> Self {
        unsafe {
            // We know from the C API that these are buckets of bits, so just memcpy
            core::ptr::read(self)
        }
    }
}

impl Default for AudioSpec {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for AudioSpec {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let spec = &self.0;
        fmt.debug_struct("SDL_AudioSpec")
            .field("freq", &spec.freq)
            .field("format", &AudioFormat(spec.format))
            .field("channels", &spec.channels)
            .field("silence", &spec.silence)
            .field("samples", &spec.samples)
            .field("padding", &spec.padding)
            .field("size", &spec.size)
            .field(
                "callback",
                &spec
                    .callback
                    .map(|p| p as *const ())
                    .unwrap_or(core::ptr::null()),
            )
            .field("userdata", &spec.userdata)
            .finish()
    }
}
