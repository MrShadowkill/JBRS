use super::*;
use std::ffi::{c_char, c_void, CStr};
#[repr(C)]
struct Spec {
    freq: i32,
    format: u16,
    channels: u8,
    silence: u8,
    samples: u16,
    padding: u16,
    size: u32,
    callback: Option<unsafe extern "C" fn(*mut c_void, *mut u8, i32)>,
    userdata: *mut c_void,
}
#[cfg_attr(
    target_os = "linux",
    link(name = "libSDL2-2.0.so.0", kind = "dylib", modifiers = "+verbatim")
)]
#[cfg_attr(any(target_os = "windows", target_os = "android"), link(name = "SDL2"))]
extern "C" {
    fn SDL_InitSubSystem(flags: u32) -> i32;
    fn SDL_QuitSubSystem(flags: u32);
    fn SDL_GetError() -> *const c_char;
    fn SDL_OpenAudioDevice(
        device: *const c_char,
        capture: i32,
        want: *const Spec,
        got: *mut Spec,
        changes: i32,
    ) -> u32;
    fn SDL_CloseAudioDevice(device: u32);
    fn SDL_PauseAudioDevice(device: u32, pause: i32);
    fn SDL_QueueAudio(device: u32, data: *const c_void, len: u32) -> i32;
    fn SDL_GetQueuedAudioSize(device: u32) -> u32;
    fn SDL_ClearQueuedAudio(device: u32);
}
fn error() -> String {
    unsafe {
        let p = SDL_GetError();
        if p.is_null() {
            "unknown SDL audio error".into()
        } else {
            CStr::from_ptr(p).to_string_lossy().into_owned()
        }
    }
}
pub struct Audio {
    device: u32,
    synth: Synth,
    buffer: [f32; 1024],
    paused: bool,
    fade_remaining: usize,
}
impl Audio {
    pub fn open() -> Result<Self, String> {
        unsafe {
            if SDL_InitSubSystem(0x10) != 0 {
                return Err(format!("SDL audio initialization: {}", error()));
            }
            let want = Spec {
                freq: RATE as i32,
                format: if cfg!(target_endian = "little") {
                    0x8120
                } else {
                    0x9120
                },
                channels: 2,
                silence: 0,
                samples: 512,
                padding: 0,
                size: 0,
                callback: None,
                userdata: std::ptr::null_mut(),
            };
            let mut got = Spec { ..want };
            let device = SDL_OpenAudioDevice(std::ptr::null(), 0, &want, &mut got, 0);
            if device == 0 {
                SDL_QuitSubSystem(0x10);
                return Err(format!("SDL playback device: {}", error()));
            }
            if got.freq != want.freq || got.format != want.format || got.channels != 2 {
                SDL_CloseAudioDevice(device);
                SDL_QuitSubSystem(0x10);
                return Err("SDL returned an unsupported audio format".into());
            }
            Ok(Self {
                device,
                synth: Synth::default(),
                buffer: [0.; 1024],
                paused: true,
                fade_remaining: 128,
            })
        }
    }
    pub fn pump(&mut self, scene: &Scene) -> Result<(), String> {
        unsafe {
            if !scene.active {
                if !self.paused {
                    SDL_PauseAudioDevice(self.device, 1);
                    SDL_ClearQueuedAudio(self.device);
                    self.synth = Synth::default();
                    self.paused = true;
                }
                return Ok(());
            }
            if scene.steps > 0 {
                self.synth.step(scene.surface);
            }
            // Six queued blocks (~139 ms) tolerate slow frames; filling remains bounded.
            if self.paused || SDL_GetQueuedAudioSize(self.device) == 0 {
                self.fade_remaining = 128;
            }
            let bytes = std::mem::size_of_val(&self.buffer) as u32;
            for _ in 0..8 {
                if SDL_GetQueuedAudioSize(self.device) >= bytes * 6 {
                    break;
                }
                self.synth.render(scene, &mut self.buffer);
                for frame in self.buffer.chunks_exact_mut(2) {
                    if self.fade_remaining == 0 {
                        break;
                    }
                    let gain = 1. - self.fade_remaining as f32 / 128.;
                    frame[0] *= gain;
                    frame[1] *= gain;
                    self.fade_remaining -= 1;
                }
                if SDL_QueueAudio(self.device, self.buffer.as_ptr().cast(), bytes) != 0 {
                    return Err(format!("SDL audio queue: {}", error()));
                }
            }
            if self.paused {
                SDL_PauseAudioDevice(self.device, 0);
                self.paused = false;
            }
            Ok(())
        }
    }
    pub fn queued_bytes(&self) -> u32 {
        unsafe { SDL_GetQueuedAudioSize(self.device) }
    }
}
impl Drop for Audio {
    fn drop(&mut self) {
        unsafe {
            SDL_CloseAudioDevice(self.device);
            SDL_QuitSubSystem(0x10);
        }
    }
}
