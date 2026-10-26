#[cfg(any(target_os = "android", feature = "gles"))]
extern crate renderer_gles2 as renderer_gl21;
mod chaos;
mod doors;
mod escape;
mod fleet;
mod game;
mod glitches;
mod heists;
mod mobile;
mod npcs;
mod quality;
mod startup_warning;
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

pub fn export_editor_map(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    game::export_editor_map(path)
}

#[cfg(test)]
mod allocation_probe {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::cell::Cell;
    thread_local! {
        static TRACK: Cell<bool> = const { Cell::new(false) };
        static COUNT: Cell<usize> = const { Cell::new(0) };
    }
    struct Allocator;
    #[global_allocator]
    static ALLOCATOR: Allocator = Allocator;
    fn record() {
        let _ = TRACK.try_with(|track| {
            if track.get() {
                let _ = COUNT.try_with(|count| count.set(count.get() + 1));
            }
        });
    }
    unsafe impl GlobalAlloc for Allocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            record();
            System.alloc(layout)
        }
        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            record();
            System.alloc_zeroed(layout)
        }
        unsafe fn realloc(&self, p: *mut u8, layout: Layout, n: usize) -> *mut u8 {
            record();
            System.realloc(p, layout, n)
        }
        unsafe fn dealloc(&self, p: *mut u8, layout: Layout) {
            System.dealloc(p, layout);
        }
    }
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            TRACK.with(|track| track.set(false));
        }
    }
    pub fn count(f: impl FnOnce()) -> usize {
        COUNT.with(|count| count.set(0));
        TRACK.with(|track| track.set(true));
        let guard = Guard;
        f();
        drop(guard);
        COUNT.with(Cell::get)
    }
}
