#[cfg(any(target_os = "android", feature = "gles"))]
extern crate renderer_gles2 as renderer_gl21;
mod apartment_raid;
mod doors;
mod escape;
mod fleet;
mod game;
mod heists;
mod mobile;
mod npcs;
mod quality;
mod traffic;
mod world_view;
#[allow(dead_code)]
pub fn desktop_main() {
    if let Err(e) = game::run() {
        eprintln!("JBRS: {e}");
        std::process::exit(1);
    }
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn SDL_main(_argc: i32, _argv: *mut *mut std::ffi::c_char) -> i32 {
    match mobile::configure_android().and_then(|()| game::run()) {
        Ok(()) => 0,
        Err(e) => {
            mobile::android_error(&e);
            1
        }
    }
}
