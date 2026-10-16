//! Windows exports only GL 1.1 directly; resolve GL 2.1 with the current SDL context.
#![allow(non_snake_case)]
use super::{c_char, c_void, SDL_GL_GetProcAddress};
use std::cell::Cell;
macro_rules! entry_points {
    ($(fn $name:ident($($arg:ident: $ty:ty),* $(,)?) $(-> $ret:ty)?; )*) => {
        #[derive(Clone, Copy)]
        struct Functions { $($name: unsafe extern "system" fn($($ty),*) $(-> $ret)?,)* }
        thread_local! { static FUNCTIONS: Cell<Option<Functions>> = const { Cell::new(None) }; }
        pub(super) unsafe fn load() -> Result<(), String> {
            let gl = Functions {
                $($name: {
                    let name = concat!(stringify!($name), "\0");
                    let p = SDL_GL_GetProcAddress(name.as_ptr().cast());
                    if p.is_null() || matches!(p as usize, 1 | 2 | 3 | usize::MAX) {
                        return Err(format!("OpenGL 2.1 function {} is unavailable. Install the GPU manufacturer's Windows driver.", stringify!($name)));
                    }
                    std::mem::transmute::<*mut c_void, unsafe extern "system" fn($($ty),*) $(-> $ret)?>(p)
                },)*
            };
            FUNCTIONS.with(|slot| slot.set(Some(gl)));
            Ok(())
        }
        $(#[allow(clippy::too_many_arguments)] // Match the OpenGL ABI exactly.
        pub(super) unsafe fn $name($($arg: $ty),*) $(-> $ret)? {
            FUNCTIONS.with(|slot| match slot.get() {
                Some(gl) => (gl.$name)($($arg),*),
                // Only reached by an internal renderer ordering bug, never a failed load.
                None => unreachable!("Renderer must load OpenGL before issuing commands"),
            })
        })*
    };
}
entry_points! {
    fn glColor4f(r: f32, g: f32, b: f32, a: f32);
    fn glBlendFunc(src: u32, dst: u32);
    fn glDepthMask(enabled: u8);
    fn glGenBuffers(n: i32, id: *mut u32);
    fn glDeleteBuffers(n: i32, id: *const u32);
    fn glBindBuffer(target: u32, id: u32);
    fn glBufferData(target: u32, size: isize, data: *const c_void, usage: u32);
    fn glEnableClientState(s: u32);
    fn glDisableClientState(s: u32);
    fn glVertexPointer(size: i32, kind: u32, stride: i32, p: *const c_void);
    fn glColorPointer(size: i32, kind: u32, stride: i32, p: *const c_void);
    fn glTexCoordPointer(size: i32, kind: u32, stride: i32, p: *const c_void);
    fn glDrawArrays(mode: u32, first: i32, count: i32);
    fn glTexCoord2f(x: f32, y: f32);
    fn glGenTextures(n: i32, id: *mut u32);
    fn glDeleteTextures(n: i32, id: *const u32);
    fn glBindTexture(target: u32, id: u32);
    fn glTexImage2D(
        target: u32,
        level: i32,
        internal: i32,
        w: i32,
        h: i32,
        border: i32,
        format: u32,
        kind: u32,
        data: *const c_void,
    );
    fn glTexParameteri(target: u32, p: u32, v: i32);
    fn glCreateShader(kind: u32) -> u32;
    fn glShaderSource(shader: u32, n: i32, source: *const *const c_char, len: *const i32);
    fn glCompileShader(shader: u32);
    fn glGetShaderiv(shader: u32, p: u32, v: *mut i32);
    fn glGetShaderInfoLog(shader: u32, max: i32, n: *mut i32, log: *mut c_char);
    fn glCreateProgram() -> u32;
    fn glAttachShader(program: u32, shader: u32);
    fn glLinkProgram(program: u32);
    fn glGetProgramiv(program: u32, p: u32, v: *mut i32);
    fn glGetProgramInfoLog(program: u32, max: i32, n: *mut i32, log: *mut c_char);
    fn glDeleteShader(id: u32);
    fn glDeleteProgram(id: u32);
    fn glUseProgram(id: u32);
    fn glGetUniformLocation(program: u32, name: *const c_char) -> i32;
    fn glUniform1f(location: i32, v: f32);
    fn glUniform1i(location: i32, v: i32);
    fn glUniform3f(location: i32, x: f32, y: f32, z: f32);
    fn glClearColor(r: f32, g: f32, b: f32, a: f32);
    fn glClear(mask: u32);
    fn glEnable(a: u32);
    fn glDisable(a: u32);
    fn glViewport(x: i32, y: i32, w: i32, h: i32);
    fn glMatrixMode(m: u32);
    fn glLoadIdentity();
    fn glFrustum(l: f64, r: f64, b: f64, t: f64, n: f64, f: f64);
    fn glOrtho(l: f64, r: f64, b: f64, t: f64, n: f64, f: f64);
    fn glRotatef(a: f32, x: f32, y: f32, z: f32);
    fn glTranslatef(x: f32, y: f32, z: f32);
    fn glScalef(x: f32, y: f32, z: f32);
    fn glPushMatrix();
    fn glPopMatrix();
    fn glBegin(m: u32);
    fn glEnd();
    fn glColor3f(r: f32, g: f32, b: f32);
    fn glVertex3f(x: f32, y: f32, z: f32);
    fn glGetString(p: u32) -> *const u8;
    fn glGetError() -> u32;
    fn glReadPixels(x: i32, y: i32, w: i32, h: i32, format: u32, kind: u32, data: *mut c_void);
    fn glPixelStorei(p: u32, v: i32);
}
