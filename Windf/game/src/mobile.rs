pub fn enabled() -> bool {
    cfg!(target_os = "android") || std::env::var("JBRS_TOUCH").as_deref() == Ok("1")
}
#[cfg(target_os = "android")]
pub fn configure_android() -> Result<(), String> {
    use std::ffi::{c_char, CStr};
    #[link(name = "SDL2")]
    extern "C" {
        fn SDL_AndroidGetInternalStoragePath() -> *const c_char;
        fn SDL_SetHint(name: *const c_char, value: *const c_char) -> i32;
    }
    unsafe {
        let p = SDL_AndroidGetInternalStoragePath();
        if p.is_null() {
            return Err("Android private storage is unavailable".into());
        }
        if !p.is_null() {
            let path = std::path::PathBuf::from(CStr::from_ptr(p).to_string_lossy().as_ref())
                .join("save.json");
            std::env::set_var("JBRS_SAVE", path);
        }
        SDL_SetHint(c"SDL_ANDROID_TRAP_BACK_BUTTON".as_ptr(), c"1".as_ptr());
        SDL_SetHint(c"SDL_TOUCH_MOUSE_EVENTS".as_ptr(), c"0".as_ptr());
        SDL_SetHint(c"SDL_MOUSE_TOUCH_EVENTS".as_ptr(), c"0".as_ptr());
    }
    // Mobile defaults are applied only on the first launch. Saved settings remain editable.
    if !save_system::path().exists() {
        std::env::set_var("JBRS_QUALITY", "0");
        std::env::set_var("JBRS_TEXTURE_LEVEL", "0");
        std::env::set_var("JBRS_SHADOWS", "0");
    }
    Ok(())
}
#[cfg(target_os = "android")]
pub fn android_error(message: &str) {
    use std::ffi::{c_char, CString};
    #[link(name = "log")]
    extern "C" {
        fn __android_log_write(priority: i32, tag: *const c_char, text: *const c_char) -> i32;
    }
    #[link(name = "SDL2")]
    extern "C" {
        fn SDL_ShowSimpleMessageBox(
            flags: u32,
            title: *const c_char,
            message: *const c_char,
            window: *mut std::ffi::c_void,
        ) -> i32;
    }
    if let Ok(text) = CString::new(message) {
        unsafe {
            __android_log_write(6, c"JBRS".as_ptr(), text.as_ptr());
            SDL_ShowSimpleMessageBox(
                0x10,
                c"JBRS Mobile".as_ptr(),
                text.as_ptr(),
                std::ptr::null_mut(),
            );
        }
    }
}
pub fn quiz_key(x: f32, y: f32) -> Option<i32> {
    for (i, key) in [49, 50, 51, 52, 53, 54, 55, 56, 57, 8, 48, 13]
        .into_iter()
        .enumerate()
    {
        let bx = 350. + (i % 3) as f32 * 105.;
        let by = 215. + (i / 3) as f32 * 70.;
        if (bx..=bx + 95.).contains(&x) && (by..=by + 60.).contains(&y) {
            return Some(key);
        }
    }
    None
}
