//! ES2 renderer support: explicit attributes, CPU transform stacks, streamed dynamic batches.
//! No desktop fixed-function calls are sent to the driver.
#![allow(non_snake_case, clippy::too_many_arguments)]
use super::{c_char, c_void};
use std::cell::RefCell;
mod api {
    use super::*;
    #[cfg_attr(target_os = "android", link(name = "GLESv2"))]
    #[cfg_attr(
        target_os = "linux",
        link(name = "libGLESv2.so.2", kind = "dylib", modifiers = "+verbatim")
    )]
    extern "C" {
        pub fn glGenFramebuffers(n: i32, ids: *mut u32);
        pub fn glDeleteFramebuffers(n: i32, ids: *const u32);
        pub fn glBindFramebuffer(target: u32, id: u32);
        pub fn glFramebufferTexture2D(
            target: u32,
            attachment: u32,
            textarget: u32,
            texture: u32,
            level: i32,
        );
        pub fn glCheckFramebufferStatus(target: u32) -> u32;
        pub fn glGenRenderbuffers(n: i32, ids: *mut u32);
        pub fn glDeleteRenderbuffers(n: i32, ids: *const u32);
        pub fn glBindRenderbuffer(target: u32, id: u32);
        pub fn glRenderbufferStorage(target: u32, format: u32, w: i32, h: i32);
        pub fn glFramebufferRenderbuffer(
            target: u32,
            attachment: u32,
            renderbuffer_target: u32,
            renderbuffer: u32,
        );

        pub fn glBlendFunc(src: u32, dst: u32);
        pub fn glDepthMask(enabled: u8);
        pub fn glGenBuffers(n: i32, id: *mut u32);
        pub fn glDeleteBuffers(n: i32, id: *const u32);
        pub fn glBindBuffer(target: u32, id: u32);
        pub fn glBufferData(target: u32, size: isize, data: *const c_void, usage: u32);
        pub fn glDrawArrays(mode: u32, first: i32, count: i32);
        pub fn glGenTextures(n: i32, id: *mut u32);
        pub fn glDeleteTextures(n: i32, id: *const u32);
        pub fn glBindTexture(target: u32, id: u32);
        pub fn glTexImage2D(
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
        pub fn glTexParameteri(target: u32, p: u32, v: i32);
        pub fn glCreateShader(kind: u32) -> u32;
        pub fn glShaderSource(shader: u32, n: i32, source: *const *const c_char, len: *const i32);
        pub fn glCompileShader(shader: u32);
        pub fn glGetShaderiv(shader: u32, p: u32, v: *mut i32);
        pub fn glGetShaderInfoLog(shader: u32, max: i32, n: *mut i32, log: *mut c_char);
        pub fn glCreateProgram() -> u32;
        pub fn glAttachShader(program: u32, shader: u32);
        pub fn glLinkProgram(program: u32);
        pub fn glGetProgramiv(program: u32, p: u32, v: *mut i32);
        pub fn glGetProgramInfoLog(program: u32, max: i32, n: *mut i32, log: *mut c_char);
        pub fn glDeleteShader(id: u32);
        pub fn glDeleteProgram(id: u32);
        pub fn glUseProgram(id: u32);
        pub fn glGetUniformLocation(program: u32, name: *const c_char) -> i32;
        pub fn glUniform1f(location: i32, v: f32);
        pub fn glUniform1i(location: i32, v: i32);
        pub fn glUniform3f(location: i32, x: f32, y: f32, z: f32);
        pub fn glClearColor(r: f32, g: f32, b: f32, a: f32);
        pub fn glClear(mask: u32);
        pub fn glEnable(a: u32);
        pub fn glDisable(a: u32);
        pub fn glViewport(x: i32, y: i32, w: i32, h: i32);
        pub fn glGetString(p: u32) -> *const u8;
        pub fn glGetError() -> u32;
        pub fn glReadPixels(
            x: i32,
            y: i32,
            w: i32,
            h: i32,
            format: u32,
            kind: u32,
            data: *mut c_void,
        );
        pub fn glPixelStorei(p: u32, v: i32);
        pub fn glVertexAttribPointer(
            id: u32,
            size: i32,
            kind: u32,
            normalized: u8,
            stride: i32,
            p: *const c_void,
        );
        pub fn glEnableVertexAttribArray(id: u32);
        pub fn glDisableVertexAttribArray(id: u32);
        pub fn glUniformMatrix4fv(id: i32, n: i32, transpose: u8, p: *const f32);
        pub fn glBindAttribLocation(program: u32, id: u32, name: *const c_char);
    }
}
pub(super) use api::{
    glAttachShader, glBindAttribLocation, glBindBuffer, glBindTexture, glBlendFunc, glBufferData,
    glClear, glClearColor, glCompileShader, glCreateProgram, glCreateShader, glDeleteBuffers,
    glDeleteProgram, glDeleteShader, glDeleteTextures, glDepthMask, glGenBuffers, glGenTextures,
    glGetError, glGetProgramInfoLog, glGetProgramiv, glGetShaderInfoLog, glGetShaderiv,
    glGetString, glGetUniformLocation, glLinkProgram, glPixelStorei, glReadPixels, glShaderSource,
    glTexImage2D, glTexParameteri, glUniform1f, glUniform1i, glUniform3f, glViewport,
};
pub(super) use api::{
    glBindFramebuffer, glBindRenderbuffer, glCheckFramebufferStatus, glDeleteFramebuffers,
    glDeleteRenderbuffers, glFramebufferRenderbuffer, glFramebufferTexture2D, glGenFramebuffers,
    glGenRenderbuffers, glRenderbufferStorage,
};
type Mat = [f32; 16];
fn identity() -> Mat {
    [
        1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
    ]
}
fn multiply(a: Mat, b: Mat) -> Mat {
    let mut out = [0.; 16];
    for col in 0..4 {
        for row in 0..4 {
            for k in 0..4 {
                out[col * 4 + row] += a[k * 4 + row] * b[col * 4 + k];
            }
        }
    }
    out
}
#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    pos: [f32; 3],
    color: [f32; 4],
    uv: [f32; 2],
}
struct State {
    projection: Vec<Mat>,
    model: Vec<Mat>,
    projection_mode: bool,
    color: [f32; 4],
    uv: [f32; 2],
    vertices: Vec<Vertex>,
    triangles: Vec<Vertex>,
    primitive: u32,
    stream: u32,
    world: u32,
    ui: u32,
    current: u32,
    textured: bool,
    world_m: i32,
    world_p: i32,
    ui_m: i32,
    ui_p: i32,
    ui_texture: i32,
}
impl Default for State {
    fn default() -> Self {
        Self {
            projection: vec![identity()],
            model: vec![identity()],
            projection_mode: false,
            color: [1.; 4],
            uv: [0.; 2],
            vertices: Vec::with_capacity(4096),
            triangles: Vec::with_capacity(6144),
            primitive: 4,
            stream: 0,
            world: 0,
            ui: 0,
            current: 0,
            textured: false,
            world_m: -1,
            world_p: -1,
            ui_m: -1,
            ui_p: -1,
            ui_texture: -1,
        }
    }
}
impl State {
    fn stack(&mut self) -> &mut Vec<Mat> {
        if self.projection_mode {
            &mut self.projection
        } else {
            &mut self.model
        }
    }
    fn transform(&mut self, m: Mat) {
        let top = self.stack().last_mut().unwrap();
        *top = multiply(*top, m);
    }
    unsafe fn uniforms(&self) {
        let (m, p) = if self.current == self.ui {
            (self.ui_m, self.ui_p)
        } else {
            (self.world_m, self.world_p)
        };
        api::glUniformMatrix4fv(m, 1, 0, self.model.last().unwrap().as_ptr());
        api::glUniformMatrix4fv(p, 1, 0, self.projection.last().unwrap().as_ptr());
        if self.current == self.ui {
            api::glUniform1i(self.ui_texture, i32::from(self.textured));
        }
    }
}
thread_local! {static STATE:RefCell<State>=RefCell::new(State::default());}
pub(super) unsafe fn initialize(world: u32) -> Result<(), String> {
    let vs = super::compile(0x8b31, include_str!("../shaders/ui.vert"))?;
    let fs = match super::compile(0x8b30, include_str!("../shaders/ui.frag")) {
        Ok(v) => v,
        Err(e) => {
            api::glDeleteShader(vs);
            return Err(e);
        }
    };
    let ui = api::glCreateProgram();
    api::glAttachShader(ui, vs);
    api::glAttachShader(ui, fs);
    for (i, n) in [c"position", c"color", c"texcoord"].iter().enumerate() {
        api::glBindAttribLocation(ui, i as u32, n.as_ptr());
    }
    api::glLinkProgram(ui);
    api::glDeleteShader(vs);
    api::glDeleteShader(fs);
    let mut ok = 0;
    api::glGetProgramiv(ui, 0x8b82, &mut ok);
    if ok == 0 {
        let mut log = [0u8; 4096];
        api::glGetProgramInfoLog(ui, 4096, std::ptr::null_mut(), log.as_mut_ptr().cast());
        api::glDeleteProgram(ui);
        return Err(format!(
            "GLES2 UI shader link: {}",
            String::from_utf8_lossy(&log)
        ));
    }
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        *s = State::default();
        s.world = world;
        s.ui = ui;
        s.world_m = api::glGetUniformLocation(world, c"model_view".as_ptr());
        s.world_p = api::glGetUniformLocation(world, c"projection".as_ptr());
        s.ui_m = api::glGetUniformLocation(ui, c"model_view".as_ptr());
        s.ui_p = api::glGetUniformLocation(ui, c"projection".as_ptr());
        s.ui_texture = api::glGetUniformLocation(ui, c"textured".as_ptr());
        api::glGenBuffers(1, &mut s.stream);
    });
    Ok(())
}
pub(super) unsafe fn shutdown() {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        api::glDeleteBuffers(1, &s.stream);
        api::glDeleteProgram(s.ui);
        *s = State::default();
    });
}
pub(super) unsafe fn set_world_program(world: u32) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.world = world;
        s.world_m = api::glGetUniformLocation(world, c"model_view".as_ptr());
        s.world_p = api::glGetUniformLocation(world, c"projection".as_ptr());
    });
}
pub(super) unsafe fn glUseProgram(id: u32) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.current = if id == 0 { s.ui } else { id };
        api::glUseProgram(s.current);
    });
}
pub(super) unsafe fn glEnable(kind: u32) {
    if kind == 0xde1 {
        STATE.with(|s| s.borrow_mut().textured = true);
    } else {
        api::glEnable(kind);
    }
}
pub(super) unsafe fn glDisable(kind: u32) {
    if kind == 0xde1 {
        STATE.with(|s| s.borrow_mut().textured = false);
    } else {
        api::glDisable(kind);
    }
}
pub(super) unsafe fn glMatrixMode(mode: u32) {
    STATE.with(|s| s.borrow_mut().projection_mode = mode == 0x1701);
}
pub(super) unsafe fn glLoadIdentity() {
    STATE.with(|s| *s.borrow_mut().stack().last_mut().unwrap() = identity());
}
pub(super) unsafe fn glPushMatrix() {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let m = *s.stack().last().unwrap();
        s.stack().push(m);
    });
}
pub(super) unsafe fn glPopMatrix() {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        if s.stack().len() > 1 {
            s.stack().pop();
        }
    });
}
pub(super) unsafe fn glTranslatef(x: f32, y: f32, z: f32) {
    let mut m = identity();
    m[12] = x;
    m[13] = y;
    m[14] = z;
    STATE.with(|s| s.borrow_mut().transform(m));
}
pub(super) unsafe fn glScalef(x: f32, y: f32, z: f32) {
    let mut m = identity();
    m[0] = x;
    m[5] = y;
    m[10] = z;
    STATE.with(|s| s.borrow_mut().transform(m));
}
pub(super) unsafe fn glRotatef(angle: f32, x: f32, y: f32, z: f32) {
    let len = (x * x + y * y + z * z).sqrt();
    if len < 1e-6 {
        return;
    }
    let (x, y, z) = (x / len, y / len, z / len);
    let (s, c) = angle.to_radians().sin_cos();
    let t = 1. - c;
    let m = [
        t * x * x + c,
        t * x * y + s * z,
        t * x * z - s * y,
        0.,
        t * x * y - s * z,
        t * y * y + c,
        t * y * z + s * x,
        0.,
        t * x * z + s * y,
        t * y * z - s * x,
        t * z * z + c,
        0.,
        0.,
        0.,
        0.,
        1.,
    ];
    STATE.with(|v| v.borrow_mut().transform(m));
}
pub(super) unsafe fn glFrustum(l: f64, r: f64, b: f64, t: f64, n: f64, f: f64) {
    let m = [
        2. * n / (r - l),
        0.,
        0.,
        0.,
        0.,
        2. * n / (t - b),
        0.,
        0.,
        (r + l) / (r - l),
        (t + b) / (t - b),
        -(f + n) / (f - n),
        -1.,
        0.,
        0.,
        -2. * f * n / (f - n),
        0.,
    ]
    .map(|v| v as f32);
    STATE.with(|s| s.borrow_mut().transform(m));
}
pub(super) unsafe fn glOrtho(l: f64, r: f64, b: f64, t: f64, n: f64, f: f64) {
    let m = [
        2. / (r - l),
        0.,
        0.,
        0.,
        0.,
        2. / (t - b),
        0.,
        0.,
        0.,
        0.,
        -2. / (f - n),
        0.,
        -(r + l) / (r - l),
        -(t + b) / (t - b),
        -(f + n) / (f - n),
        1.,
    ]
    .map(|v| v as f32);
    STATE.with(|s| s.borrow_mut().transform(m));
}
pub(super) unsafe fn glColor4f(r: f32, g: f32, b: f32, a: f32) {
    STATE.with(|s| s.borrow_mut().color = [r, g, b, a]);
}
pub(super) unsafe fn glColor3f(r: f32, g: f32, b: f32) {
    glColor4f(r, g, b, 1.);
}
pub(super) unsafe fn glTexCoord2f(u: f32, v: f32) {
    STATE.with(|s| s.borrow_mut().uv = [u, v]);
}
pub(super) unsafe fn glBegin(mode: u32) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.primitive = mode;
        s.vertices.clear();
    });
}
pub(super) unsafe fn glVertex3f(x: f32, y: f32, z: f32) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let v = Vertex {
            pos: [x, y, z],
            color: s.color,
            uv: s.uv,
        };
        s.vertices.push(v);
    });
}
pub(super) unsafe fn glEnd() {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        if s.vertices.is_empty() {
            return;
        }
        s.uniforms();
        s.triangles.clear();
        let mode = if s.primitive == 7 {
            for i in (0..s.vertices.len() / 4 * 4).step_by(4) {
                for j in [0, 1, 2, 0, 2, 3] {
                    let v = s.vertices[i + j];
                    s.triangles.push(v);
                }
            }
            4
        } else {
            s.primitive
        };
        let verts = if s.primitive == 7 {
            &s.triangles
        } else {
            &s.vertices
        };
        api::glBindBuffer(0x8892, s.stream);
        api::glBufferData(
            0x8892,
            std::mem::size_of_val(verts.as_slice()) as isize,
            verts.as_ptr().cast(),
            0x88e0,
        );
        for (id, size, offset) in [(0, 3, 0), (1, 4, 12), (2, 2, 28)] {
            api::glEnableVertexAttribArray(id);
            api::glVertexAttribPointer(id, size, 0x1406, 0, 36, offset as *const c_void);
        }
        api::glDrawArrays(mode, 0, verts.len() as i32);
        for id in 0..3 {
            api::glDisableVertexAttribArray(id);
        }
        api::glBindBuffer(0x8892, 0);
    });
}
fn attribute(kind: u32) -> u32 {
    match kind {
        0x8074 => 0,
        0x8076 => 1,
        _ => 2,
    }
}
pub(super) unsafe fn glEnableClientState(kind: u32) {
    api::glEnableVertexAttribArray(attribute(kind));
}
pub(super) unsafe fn glDisableClientState(kind: u32) {
    api::glDisableVertexAttribArray(attribute(kind));
}
pub(super) unsafe fn glVertexPointer(n: i32, ty: u32, stride: i32, p: *const c_void) {
    api::glVertexAttribPointer(0, n, ty, 0, stride, p);
}
pub(super) unsafe fn glColorPointer(n: i32, ty: u32, stride: i32, p: *const c_void) {
    api::glVertexAttribPointer(1, n, ty, 0, stride, p);
}
pub(super) unsafe fn glTexCoordPointer(n: i32, ty: u32, stride: i32, p: *const c_void) {
    api::glVertexAttribPointer(2, n, ty, 0, stride, p);
}
pub(super) unsafe fn glDrawArrays(mode: u32, first: i32, count: i32) {
    STATE.with(|s| s.borrow().uniforms());
    api::glDrawArrays(mode, first, count);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transform_order_matches_column_vector_gl() {
        let mut s = State::default();
        let mut t = identity();
        t[12] = 3.;
        s.transform(t);
        let mut scale = identity();
        scale[0] = 2.;
        s.transform(scale);
        assert_eq!(s.model[0][12], 3.);
        assert_eq!(s.model[0][0], 2.);
    }
    #[test]
    fn projection_does_not_change_model_stack() {
        let mut s = State {
            projection_mode: true,
            ..Default::default()
        };
        let mut m = identity();
        m[0] = 2.;
        s.transform(m);
        assert_eq!(s.model[0], identity());
        assert_eq!(s.projection[0][0], 2.);
    }
}
