use std::ffi::{c_char, c_void, CStr, CString};
#[repr(C, align(8))]
pub struct Event {
    pub data: [u8; 56],
}
impl Event {
    pub fn kind(&self) -> u32 {
        u32::from_ne_bytes(self.data[0..4].try_into().unwrap())
    }
    pub fn key(&self) -> i32 {
        let scan = i32::from_ne_bytes(self.data[16..20].try_into().unwrap());
        match scan {
            4..=29 => 97 + scan - 4,
            30..=38 => 49 + scan - 30,
            39 => 48,
            40 => 13,
            41 => 27,
            44 => 32,
            _ => {
                if scan >= 58 {
                    1073741824 + scan
                } else {
                    i32::from_ne_bytes(self.data[20..24].try_into().unwrap())
                }
            }
        }
    }
    pub fn synthetic_key(scan: i32, symbol: i32) -> Self {
        let mut e = Self { data: [0; 56] };
        e.data[0..4].copy_from_slice(&0x300u32.to_ne_bytes());
        e.data[16..20].copy_from_slice(&scan.to_ne_bytes());
        e.data[20..24].copy_from_slice(&symbol.to_ne_bytes());
        e
    }
    pub fn click(&self) -> (i32, i32) {
        (
            i32::from_ne_bytes(self.data[20..24].try_into().unwrap()),
            i32::from_ne_bytes(self.data[24..28].try_into().unwrap()),
        )
    }
    pub fn repeat(&self) -> bool {
        self.data[13] != 0
    }
    pub fn finger(&self) -> (i64, f32, f32) {
        (
            i64::from_ne_bytes(self.data[16..24].try_into().unwrap()),
            f32::from_ne_bytes(self.data[24..28].try_into().unwrap()),
            f32::from_ne_bytes(self.data[28..32].try_into().unwrap()),
        )
    }
    pub fn touch_mouse(&self) -> bool {
        u32::from_ne_bytes(self.data[12..16].try_into().unwrap()) == u32::MAX
    }
    pub fn motion(&self) -> (i32, i32) {
        (
            i32::from_ne_bytes(self.data[28..32].try_into().unwrap()),
            i32::from_ne_bytes(self.data[32..36].try_into().unwrap()),
        )
    }
}
#[cfg_attr(
    target_os = "linux",
    link(name = "libSDL2-2.0.so.0", kind = "dylib", modifiers = "+verbatim")
)]
#[cfg_attr(target_os = "windows", link(name = "SDL2"))]
extern "C" {
    fn SDL_Init(flags: u32) -> i32;
    fn SDL_SetMainReady();
    #[cfg(windows)]
    fn SDL_GL_GetProcAddress(name: *const c_char) -> *mut c_void;
    fn SDL_Quit();
    fn SDL_GetError() -> *const c_char;
    fn SDL_GL_SetAttribute(a: i32, v: i32) -> i32;
    fn SDL_CreateWindow(
        t: *const c_char,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        flags: u32,
    ) -> *mut c_void;
    fn SDL_DestroyWindow(w: *mut c_void);
    fn SDL_GL_CreateContext(w: *mut c_void) -> *mut c_void;
    fn SDL_GL_DeleteContext(c: *mut c_void);
    fn SDL_GL_SwapWindow(w: *mut c_void);
    fn SDL_GL_SetSwapInterval(v: i32) -> i32;
    fn SDL_PollEvent(e: *mut Event) -> i32;
    fn SDL_GetKeyboardState(n: *mut i32) -> *const u8;
    fn SDL_SetRelativeMouseMode(v: i32) -> i32;
    fn SDL_GetWindowSize(w: *mut c_void, x: *mut i32, y: *mut i32);
}
#[cfg(not(windows))]
#[link(name = "libGL.so.1", kind = "dylib", modifiers = "+verbatim")]
extern "C" {
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
#[cfg(windows)]
mod windows_gl;
#[cfg(windows)]
use windows_gl::*;
#[derive(Clone, Copy)]
pub struct GpuMesh {
    pub id: u32,
    pub vertices: i32,
    pub bytes: usize,
}
#[derive(Clone, Copy)]
pub struct Texture {
    pub id: u32,
}
pub struct Renderer {
    window: *mut c_void,
    context: *mut c_void,
    pub width: i32,
    pub height: i32,
    pub driver: String,
    pub draws: usize,
    pub triangles: usize,
    ui_open: bool,
    buffers: std::collections::HashSet<u32>,
    solid_models: std::collections::HashMap<usize, GpuMesh>,
    textures: Vec<u32>,
    program: u32,
    atlas: u32,
    font: u32,
    day_uniform: i32,
    mesh_uniform: i32,
    tint_uniform: i32,
    fog_uniform: i32,
    quality_uniform: i32,
    opacity_uniform: i32,
}
impl Renderer {
    pub fn new() -> Result<Self, String> {
        unsafe {
            SDL_SetMainReady();
            if SDL_Init(0x20) != 0 {
                return Err(error());
            }
            SDL_GL_SetAttribute(17, 2);
            SDL_GL_SetAttribute(18, 1);
            SDL_GL_SetAttribute(5, 1);
            SDL_GL_SetAttribute(6, 24);
            let title = CString::new("JBRS | Original open-world prototype").unwrap();
            let w = SDL_CreateWindow(title.as_ptr(), 0x2fff0000, 0x2fff0000, 1024, 576, 2 | 32);
            if w.is_null() {
                return Err(error());
            }
            let c = SDL_GL_CreateContext(w);
            if c.is_null() {
                SDL_DestroyWindow(w);
                return Err(error());
            }
            #[cfg(windows)]
            if let Err(e) = windows_gl::load() {
                SDL_GL_DeleteContext(c);
                SDL_DestroyWindow(w);
                SDL_Quit();
                return Err(e);
            }
            SDL_GL_SetSwapInterval(1);
            let raw = glGetString(0x1f01);
            let driver = if raw.is_null() {
                "unknown".into()
            } else {
                CStr::from_ptr(raw.cast()).to_string_lossy().into_owned()
            };
            // SDL/Mesa context probing can leave GL_INVALID_ENUM on a 2.1 context.
            // Clear startup errors before validating calls made by our renderer.
            while glGetError() != 0 {}
            let mut renderer = Self {
                window: w,
                context: c,
                width: 1024,
                height: 576,
                driver,
                draws: 0,
                triangles: 0,
                ui_open: false,
                buffers: Default::default(),
                solid_models: Default::default(),
                textures: Vec::new(),
                program: 0,
                atlas: 0,
                font: 0,
                day_uniform: -1,
                mesh_uniform: -1,
                tint_uniform: -1,
                fog_uniform: -1,
                quality_uniform: -1,
                opacity_uniform: -1,
            };
            renderer.setup()?;
            Ok(renderer)
        }
    }
    fn setup(&mut self) -> Result<(), String> {
        unsafe {
            let vertex = compile(0x8b31, include_str!("../shaders/world.vert"))?;
            let fragment = compile(0x8b30, include_str!("../shaders/world.frag"))?;
            let program = glCreateProgram();
            glAttachShader(program, vertex);
            glAttachShader(program, fragment);
            glLinkProgram(program);
            glDeleteShader(vertex);
            glDeleteShader(fragment);
            let mut ok = 0;
            glGetProgramiv(program, 0x8b82, &mut ok);
            if ok == 0 {
                let mut log = vec![0u8; 2048];
                glGetProgramInfoLog(program, 2048, std::ptr::null_mut(), log.as_mut_ptr().cast());
                glDeleteProgram(program);
                return Err(String::from_utf8_lossy(&log).into_owned());
            }
            self.program = program;
            let uniform = |name: &str| {
                let c = CString::new(name).unwrap();
                glGetUniformLocation(program, c.as_ptr())
            };
            self.day_uniform = uniform("daylight");
            self.mesh_uniform = uniform("mesh_mode");
            self.tint_uniform = uniform("model_tint");
            self.fog_uniform = uniform("fog_end");
            self.quality_uniform = uniform("detail_amount");
            self.opacity_uniform = uniform("opacity");
            glUseProgram(program);
            glUniform1f(self.opacity_uniform, 1.);
            glUniform1i(uniform("atlas"), 0);
            glUniform1f(self.fog_uniform, 450.);
            let pixels = atlas_pixels(64);
            self.atlas = self.texture(256, 256, &pixels).id;
            glUseProgram(0);
            glGenTextures(1, &mut self.font);
            glBindTexture(0x0de1, self.font);
            glTexImage2D(
                0x0de1,
                0,
                0x1908,
                512,
                640,
                0,
                0x1908,
                0x1401,
                include_bytes!("../assets/ui-font.rgba").as_ptr().cast(),
            );
            glTexParameteri(0x0de1, 0x2801, 0x2601);
            glTexParameteri(0x0de1, 0x2800, 0x2601);
            self.textures.push(self.font);
            Ok(())
        }
    }
    pub fn fog(&self, end: f32) {
        unsafe {
            glUniform1f(self.fog_uniform, end);
        }
    }
    pub fn detail(&mut self, amount: f32) {
        unsafe {
            glUseProgram(self.program);
            glUniform1f(self.quality_uniform, amount);
        }
    }
    /// Rounded superellipsoid: exponent 1 is an ellipsoid, 0.3 is a beveled box.
    pub fn rounded(
        &mut self,
        p: [f32; 3],
        size: [f32; 3],
        color: [f32; 3],
        angle: f32,
        n: usize,
        exponent: f32,
    ) {
        let n = n.clamp(6, 24);
        let rings = n / 2;
        let signed = |x: f32| x.signum() * x.abs().powf(exponent);
        unsafe {
            glUniform1i(self.mesh_uniform, 0);
            glPushMatrix();
            glTranslatef(p[0], p[1], p[2]);
            glRotatef(angle.to_degrees(), 0., 1., 0.);
            glBegin(7);
            for ring in 0..rings {
                for slice in 0..n {
                    for (i, j) in [
                        (ring, slice),
                        (ring, slice + 1),
                        (ring + 1, slice + 1),
                        (ring + 1, slice),
                    ] {
                        let lat = -std::f32::consts::FRAC_PI_2
                            + std::f32::consts::PI * i as f32 / rings as f32;
                        let lon = std::f32::consts::TAU * j as f32 / n as f32;
                        let x = signed(lat.cos()) * signed(lon.cos());
                        let y = signed(lat.sin());
                        let z = signed(lat.cos()) * signed(lon.sin());
                        let shade = (0.76 + 0.16 * y + 0.09 * x - 0.06 * z).clamp(0.4, 1.);
                        glColor3f(color[0] * shade, color[1] * shade, color[2] * shade);
                        glVertex3f(x * size[0] * 0.5, y * size[1] * 0.5, z * size[2] * 0.5);
                    }
                }
            }
            glEnd();
            glPopMatrix();
        }
        self.draws += 1;
        self.triangles += n * rings * 2;
    }
    pub fn material_resolution(&mut self, level: usize) {
        let tile = [64, 128, 256, 512][level.min(3)];
        let pixels = atlas_pixels(tile);
        unsafe {
            glBindTexture(0x0de1, self.atlas);
            glTexImage2D(
                0x0de1,
                0,
                0x1907,
                (tile * 4) as i32,
                (tile * 4) as i32,
                0,
                0x1907,
                0x1401,
                pixels.as_ptr().cast(),
            );
        }
    }
    pub fn texture(&mut self, w: i32, h: i32, pixels: &[u8]) -> Texture {
        assert_eq!(pixels.len(), (w * h * 3) as usize);
        unsafe {
            let mut id = 0;
            glGenTextures(1, &mut id);
            glBindTexture(0x0de1, id);
            glPixelStorei(0x0cf5, 1);
            glTexImage2D(
                0x0de1,
                0,
                0x1907,
                w,
                h,
                0,
                0x1907,
                0x1401,
                pixels.as_ptr().cast(),
            );
            glTexParameteri(0x0de1, 0x2801, 0x2601);
            glTexParameteri(0x0de1, 0x2800, 0x2601);
            glTexParameteri(0x0de1, 0x2802, 0x812f);
            glTexParameteri(0x0de1, 0x2803, 0x812f);
            self.textures.push(id);
            Texture { id }
        }
    }
    pub fn texture_rect(&mut self, texture: Texture, rect: [f32; 4]) {
        let [x, y, w, h] = rect;
        let resume = self.ui_open;
        self.finish_ui();
        unsafe {
            glEnable(0x0de1);
            glBindTexture(0x0de1, texture.id);
            glColor3f(1., 1., 1.);
            glBegin(7);
            for (px, py, u, v) in [
                (x, y, 0., 0.),
                (x + w, y, 1., 0.),
                (x + w, y + h, 1., 1.),
                (x, y + h, 0., 1.),
            ] {
                glTexCoord2f(u, v);
                glVertex3f(px, py, 0.);
            }
            glEnd();
            glDisable(0x0de1);
            if resume {
                glBegin(7);
                self.ui_open = true;
            }
        }
        self.draws += if resume { 2 } else { 1 };
    }
    pub fn solid_model(
        &mut self,
        mesh: &engine_core::Mesh,
        p: [f32; 3],
        angle: f32,
        tint: [f32; 3],
    ) {
        unsafe {
            glUniform1i(self.mesh_uniform, 0);
            glPushMatrix();
            glTranslatef(p[0], p[1], p[2]);
            glRotatef(angle.to_degrees(), 0., 1., 0.);
            glBegin(4);
            for v in &mesh.vertices {
                glColor3f(
                    v.color[0] * tint[0],
                    v.color[1] * tint[1],
                    v.color[2] * tint[2],
                );
                glVertex3f(v.position[0], v.position[1], v.position[2]);
            }
            glEnd();
            glPopMatrix();
        }
        self.triangles += mesh.vertices.len() / 3;
        self.draws += 1;
    }
    pub fn cached_model_usage(&self) -> (usize, usize) {
        (
            self.solid_models.len(),
            self.solid_models.values().map(|m| m.bytes).sum(),
        )
    }
    /// Upload immutable vehicle geometry once per renderer/context, then draw its VBO.
    /// Keys identify an immutable catalog part and LOD, never an individual car.
    pub fn cached_solid_model(
        &mut self,
        key: usize,
        cpu_mesh: &engine_core::Mesh,
        p: [f32; 3],
        angle: f32,
        tint: [f32; 3],
    ) {
        self.cached_solid_model_tilted(key, cpu_mesh, p, angle, tint, [0.; 2]);
    }
    pub fn cached_solid_model_tilted(
        &mut self,
        key: usize,
        cpu_mesh: &engine_core::Mesh,
        p: [f32; 3],
        angle: f32,
        tint: [f32; 3],
        tilt: [f32; 2],
    ) {
        if cpu_mesh.vertices.is_empty() {
            return;
        }
        let mesh = if let Some(mesh) = self.solid_models.get(&key) {
            *mesh
        } else {
            let mesh = self.upload(cpu_mesh);
            self.solid_models.insert(key, mesh);
            mesh
        };
        unsafe {
            glPushMatrix();
            glTranslatef(p[0], p[1], p[2]);
            glRotatef(angle.to_degrees(), 0., 1., 0.);
            glRotatef(tilt[0].to_degrees(), 1., 0., 0.);
            glRotatef(tilt[1].to_degrees(), 0., 0., 1.);
            glUniform3f(self.tint_uniform, tint[0], tint[1], tint[2]);
        }
        self.draw_buffer(mesh, 2);
        unsafe {
            glPopMatrix();
        }
    }
    pub fn upload(&mut self, mesh: &engine_core::Mesh) -> GpuMesh {
        unsafe {
            let mut id = 0;
            glGenBuffers(1, &mut id);
            glBindBuffer(0x8892, id);
            let bytes = mesh.vertices.len() * std::mem::size_of::<engine_core::Vertex>();
            glBufferData(
                0x8892,
                bytes as isize,
                mesh.vertices.as_ptr().cast(),
                0x88e4,
            );
            glBindBuffer(0x8892, 0);
            self.buffers.insert(id);
            GpuMesh {
                id,
                vertices: mesh.vertices.len() as i32,
                bytes,
            }
        }
    }
    pub fn release(&mut self, mesh: GpuMesh) {
        if self.buffers.remove(&mesh.id) {
            unsafe {
                glDeleteBuffers(1, &mesh.id);
            }
        }
    }
    pub fn mesh(&mut self, mesh: GpuMesh) {
        self.draw_buffer(mesh, 1);
    }
    fn draw_buffer(&mut self, mesh: GpuMesh, mode: i32) {
        unsafe {
            glUniform1i(self.mesh_uniform, mode);
            glBindTexture(0x0de1, self.atlas);
            glBindBuffer(0x8892, mesh.id);
            let stride = std::mem::size_of::<engine_core::Vertex>() as i32;
            glEnableClientState(0x8074);
            glEnableClientState(0x8076);
            glEnableClientState(0x8078);
            glVertexPointer(3, 0x1406, stride, std::ptr::null());
            glColorPointer(4, 0x1406, stride, 12usize as *const c_void);
            glTexCoordPointer(2, 0x1406, stride, 28usize as *const c_void);
            glDrawArrays(4, 0, mesh.vertices);
            glDisableClientState(0x8074);
            glDisableClientState(0x8076);
            glDisableClientState(0x8078);
            glBindBuffer(0x8892, 0);
            glUniform1i(self.mesh_uniform, 0);
        }
        self.draws += 1;
        self.triangles += mesh.vertices as usize / 3;
    }
    pub fn version(&self) -> String {
        unsafe {
            CStr::from_ptr(glGetString(0x1f02).cast())
                .to_string_lossy()
                .into_owned()
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        let code = unsafe { glGetError() };
        if code == 0 {
            Ok(())
        } else {
            Err(format!("OpenGL error 0x{code:x}"))
        }
    }
    pub fn poll(&self) -> Option<Event> {
        let mut e = Event { data: [0; 56] };
        unsafe {
            if SDL_PollEvent(&mut e) != 0 {
                Some(e)
            } else {
                None
            }
        }
    }
    pub fn key(&self, scan: usize) -> bool {
        unsafe {
            let mut n = 0;
            let p = SDL_GetKeyboardState(&mut n);
            scan < n as usize && *p.add(scan) != 0
        }
    }
    pub fn capture(&self, on: bool) {
        unsafe {
            SDL_SetRelativeMouseMode(on as i32);
        }
    }
    pub fn begin(&mut self, eye: [f32; 3], yaw: f32, pitch: f32, day: f32) {
        unsafe {
            SDL_GetWindowSize(self.window, &mut self.width, &mut self.height);
            self.draws = 0;
            self.triangles = 0;
            glUseProgram(self.program);
            glUniform1f(self.day_uniform, day);
            glUniform1i(self.mesh_uniform, 0);
            glViewport(0, 0, self.width, self.height);
            glClearColor(0.08 + day * 0.35, 0.11 + day * 0.49, 0.2 + day * 0.52, 1.);
            glClear(0x4100);
            glEnable(0xb71);
            glMatrixMode(0x1701);
            glLoadIdentity();
            let aspect = self.width as f64 / self.height.max(1) as f64;
            glFrustum(-aspect * 0.07, aspect * 0.07, -0.07, 0.07, 0.1, 650.);
            glMatrixMode(0x1700);
            glLoadIdentity();
            glRotatef(pitch.to_degrees(), 1., 0., 0.);
            glRotatef(yaw.to_degrees(), 0., 1., 0.);
            glTranslatef(-eye[0], -eye[1], -eye[2]);
        }
    }
    pub fn cube(&mut self, p: [f32; 3], s: [f32; 3], c: [f32; 3], angle: f32) {
        unsafe {
            glUniform1i(self.mesh_uniform, 0);
            glPushMatrix();
            glTranslatef(p[0], p[1], p[2]);
            glRotatef(angle.to_degrees(), 0., 1., 0.);
            glScalef(s[0], s[1], s[2]);
            glBegin(7);
            let verts = [
                [-0.5, -0.5, -0.5],
                [0.5, -0.5, -0.5],
                [0.5, 0.5, -0.5],
                [-0.5, 0.5, -0.5],
                [-0.5, -0.5, 0.5],
                [0.5, -0.5, 0.5],
                [0.5, 0.5, 0.5],
                [-0.5, 0.5, 0.5],
            ];
            for (face, shade) in [
                ([0, 1, 2, 3], 0.7),
                ([5, 4, 7, 6], 0.85),
                ([4, 0, 3, 7], 0.75),
                ([1, 5, 6, 2], 0.9),
                ([3, 2, 6, 7], 1.),
                ([4, 5, 1, 0], 0.6),
            ] {
                glColor3f(c[0] * shade, c[1] * shade, c[2] * shade);
                for i in face {
                    let v = verts[i];
                    glVertex3f(v[0], v[1], v[2]);
                }
            }
            glEnd();
            glPopMatrix();
        }
        self.draws += 1;
    }
    pub fn glass(&mut self, p: [f32; 3], size: [f32; 3], color: [f32; 3], angle: f32) {
        unsafe {
            glUniform1f(self.opacity_uniform, 0.28);
            glEnable(0xbe2);
            glBlendFunc(0x302, 0x303);
            glDepthMask(0);
        }
        self.cube(p, size, color, angle);
        unsafe {
            glDepthMask(1);
            glDisable(0xbe2);
            glUniform1f(self.opacity_uniform, 1.);
        }
    }
    /// A rigid cuboid aligned between two joints, for articulated limbs.
    pub fn bone(
        &mut self,
        a: [f32; 3],
        b: [f32; 3],
        width: f32,
        depth: f32,
        yaw: f32,
        c: [f32; 3],
    ) {
        let side = [yaw.cos(), 0., -yaw.sin()];
        let dy = b[1] - a[1];
        let horizontal = ((b[0] - a[0]).powi(2) + (b[2] - a[2]).powi(2)).sqrt();
        let length = (horizontal * horizontal + dy * dy).sqrt().max(0.001);
        let axis = [(b[0] - a[0]) / length, dy / length, (b[2] - a[2]) / length];
        let front = [
            side[1] * axis[2] - side[2] * axis[1],
            side[2] * axis[0] - side[0] * axis[2],
            side[0] * axis[1] - side[1] * axis[0],
        ];
        let mut v = [[0.; 3]; 8];
        for (i, signs) in [
            (-1., -1., false),
            (1., -1., false),
            (1., 1., false),
            (-1., 1., false),
            (-1., -1., true),
            (1., -1., true),
            (1., 1., true),
            (-1., 1., true),
        ]
        .iter()
        .enumerate()
        {
            let center = if signs.2 { b } else { a };
            for j in 0..3 {
                v[i][j] =
                    center[j] + side[j] * signs.0 * width * 0.5 + front[j] * signs.1 * depth * 0.5;
            }
        }
        unsafe {
            glBegin(7);
            for (face, shade) in [
                ([0, 1, 2, 3], 0.65),
                ([4, 7, 6, 5], 0.9),
                ([0, 4, 5, 1], 0.75),
                ([3, 2, 6, 7], 1.),
                ([0, 3, 7, 4], 0.8),
                ([1, 5, 6, 2], 0.88),
            ] {
                glColor3f(c[0] * shade, c[1] * shade, c[2] * shade);
                for i in face {
                    glVertex3f(v[i][0], v[i][1], v[i][2]);
                }
            }
            glEnd();
        }
        self.draws += 1;
        self.triangles += 12;
    }
    pub fn object_shadow(&mut self, p: [f32; 3], size: [f32; 3], ground: f32) {
        let h = (p[1] + size[1] * 0.5 - ground).max(0.);
        let x = p[0] - size[0] * 0.5;
        let z = p[2] - size[2] * 0.5;
        let sx = size[0];
        let sz = size[2];
        let dx = h * 0.55;
        let dz = h * 0.3;
        unsafe {
            glBegin(7);
            glColor3f(0.18, 0.21, 0.18);
            for v in [
                [x, ground + 0.035, z],
                [x + sx + dx, ground + 0.035, z + dz],
                [x + sx + dx, ground + 0.035, z + sz + dz],
                [x, ground + 0.035, z + sz],
            ] {
                glVertex3f(v[0], v[1], v[2]);
            }
            glEnd();
        }
        self.draws += 1;
        self.triangles += 2;
    }
    pub fn ui(&mut self) {
        unsafe {
            glUseProgram(0);
            glDisable(0x0de1);
            glDisable(0xb71);
            glMatrixMode(0x1701);
            glLoadIdentity();
            glOrtho(0., 1024., 576., 0., -1., 1.);
            glMatrixMode(0x1700);
            glLoadIdentity();
            glBegin(7);
            self.ui_open = true;
            self.draws += 1;
        }
    }
    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, c: [f32; 3]) {
        unsafe {
            glColor3f(c[0], c[1], c[2]);
            if !self.ui_open {
                glBegin(7);
            }
            for (px, py) in [(x, y), (x + w, y), (x + w, y + h), (x, y + h)] {
                glVertex3f(px, py, 0.);
            }
            if !self.ui_open {
                glEnd();
                self.draws += 1;
            }
        }
    }
    pub fn label(&mut self, x: f32, y: f32, size: f32, t: &str, c: [f32; 3]) {
        let resume = self.ui_open;
        self.finish_ui();
        unsafe {
            glEnable(0x0de1);
            glBindTexture(0x0de1, self.font);
            glEnable(0xbe2);
            glBlendFunc(0x302, 0x303);
            glColor4f(c[0], c[1], c[2], 1.);
            glBegin(7);
            let scale = size * 7. / 25.;
            let mut cursor = x;
            for ch in t.chars() {
                let code = ch as usize;
                let id = if (32..127).contains(&code) {
                    code - 32
                } else if (0x400..0x460).contains(&code) {
                    95 + code - 0x400
                } else {
                    31
                };
                let u = (id % 16) as f32 / 16.;
                let v = (id / 16) as f32 / 16.;
                for (px, py, tu, tv) in [
                    (cursor, y, u, v),
                    (cursor + 32. * scale, y, u + 1. / 16., v),
                    (
                        cursor + 32. * scale,
                        y + 40. * scale,
                        u + 1. / 16.,
                        v + 1. / 16.,
                    ),
                    (cursor, y + 40. * scale, u, v + 1. / 16.),
                ] {
                    glTexCoord2f(tu, tv);
                    glVertex3f(px, py, 0.);
                }
                cursor += include_bytes!("../assets/ui-font-widths.bin")[id] as f32 * scale;
            }
            glEnd();
            glDisable(0x0de1);
            glDisable(0xbe2);
            if resume {
                glBegin(7);
                self.ui_open = true;
            }
        }
        self.draws += 2;
    }
    pub fn panel(&mut self, rect: [f32; 4], color: [f32; 3], alpha: f32, radius: f32) {
        let resume = self.ui_open;
        self.finish_ui();
        let [x, y, w, h] = rect;
        let radius = radius.min(w / 2.).min(h / 2.);
        unsafe {
            glEnable(0xbe2);
            glBlendFunc(0x302, 0x303);
            glColor4f(color[0], color[1], color[2], alpha);
            glBegin(6);
            glVertex3f(x + w / 2., y + h / 2., 0.);
            for (cx, cy, start) in [
                (x + w - radius, y + radius, -90.),
                (x + w - radius, y + h - radius, 0.),
                (x + radius, y + h - radius, 90.),
                (x + radius, y + radius, 180.),
            ] {
                for i in 0..=8 {
                    let a = (start + i as f32 * 90. / 8.).to_radians();
                    glVertex3f(cx + radius * a.cos(), cy + radius * a.sin(), 0.);
                }
            }
            glVertex3f(x + w - radius, y, 0.);
            glEnd();
            glDisable(0xbe2);
            if resume {
                glBegin(7);
                self.ui_open = true;
            }
        }
        self.draws += 2;
    }
    fn finish_ui(&mut self) {
        if self.ui_open {
            unsafe {
                glEnd();
            }
            self.ui_open = false;
        }
    }
    pub fn capture_ppm(&mut self, path: &str) -> std::io::Result<()> {
        self.finish_ui();
        let mut data = vec![0u8; (self.width * self.height * 3) as usize];
        unsafe {
            glPixelStorei(0x0d05, 1);
            glReadPixels(
                0,
                0,
                self.width,
                self.height,
                0x1907,
                0x1401,
                data.as_mut_ptr().cast(),
            );
        }
        use std::io::Write;
        let mut file = std::fs::File::create(path)?;
        write!(file, "P6\n{} {}\n255\n", self.width, self.height)?;
        for row in data.chunks_exact(self.width as usize * 3).rev() {
            file.write_all(row)?;
        }
        Ok(())
    }
    pub fn end(&mut self) {
        self.finish_ui();
        unsafe { SDL_GL_SwapWindow(self.window) }
    }
}
impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            for id in &self.buffers {
                glDeleteBuffers(1, id);
            }
            for id in &self.textures {
                glDeleteTextures(1, id);
            }
            glDeleteProgram(self.program);
            SDL_GL_DeleteContext(self.context);
            SDL_DestroyWindow(self.window);
            SDL_Quit();
        }
    }
}
unsafe fn error() -> String {
    CStr::from_ptr(SDL_GetError())
        .to_string_lossy()
        .into_owned()
}

unsafe fn compile(kind: u32, source: &str) -> Result<u32, String> {
    let id = glCreateShader(kind);
    let source = CString::new(source).unwrap();
    let p = source.as_ptr();
    glShaderSource(id, 1, &p, std::ptr::null());
    glCompileShader(id);
    let mut ok = 0;
    glGetShaderiv(id, 0x8b81, &mut ok);
    if ok == 0 {
        let mut log = vec![0u8; 2048];
        glGetShaderInfoLog(id, 2048, std::ptr::null_mut(), log.as_mut_ptr().cast());
        glDeleteShader(id);
        return Err(String::from_utf8_lossy(&log).into_owned());
    }
    Ok(id)
}
fn atlas_pixels(tile: usize) -> Vec<u8> {
    let mut pixels = vec![0u8; (tile * 4) * (tile * 4) * 3];
    for y in 0..tile * 4 {
        for x in 0..tile * 4 {
            let mat = (y / tile) * 4 + x / tile;
            let u = (x % tile) * 64 / tile;
            let v = (y % tile) * 64 / tile;
            let grain = ((x * 73 + y * 151 + x * y * 3) % 23) as u8;
            let value = match mat {
                0 => 160 + grain,
                1 => 205 + grain,
                2 => {
                    if v % 16 < 2 || (u + (v / 16) % 2 * 16) % 32 < 2 {
                        140
                    } else {
                        220 + grain / 2
                    }
                }
                3 => {
                    if u % 16 < 2 || v % 16 < 2 {
                        120
                    } else {
                        210 + grain / 2
                    }
                }
                4 => {
                    if v.is_multiple_of(8) {
                        155
                    } else {
                        220
                    }
                }
                5 => 210 + grain,
                6 => 185 + grain,
                7 => 160 + grain * 2,
                8 => 190 + grain,
                9 => 255,
                10 => {
                    if u.is_multiple_of(9) {
                        145
                    } else {
                        210 + grain
                    }
                }
                11 => {
                    if v % 12 < 2 {
                        130
                    } else {
                        200 + grain
                    }
                }
                _ => 220,
            };
            let i = (y * tile * 4 + x) * 3;
            let micro = if tile > 64 {
                ((x * 17 + y * 31) % 7) as u8
            } else {
                0
            };
            pixels[i..i + 3].fill(value.saturating_sub(micro));
        }
    }
    pixels
}

#[cfg(test)]
mod material_tests {
    use super::*;
    #[test]
    fn atlas_resolutions_keep_tiles_and_add_real_texels() {
        for tile in [64, 128, 256, 512] {
            let p = atlas_pixels(tile);
            assert_eq!(p.len(), (tile * 4).pow(2) * 3);
            assert!(p.iter().any(|v| *v < 200));
        }
        assert_ne!(atlas_pixels(64)[0], atlas_pixels(512)[3]);
    }
}

#[cfg(test)]
mod keyboard_tests {
    use super::*;
    #[test]
    fn physical_controls_ignore_layout_and_capslock() {
        for (scan, key) in [(8, 101), (9, 102), (16, 109), (10, 103), (13, 106)] {
            for symbol in [key, key - 32, 0x444] {
                assert_eq!(Event::synthetic_key(scan, symbol).key(), key);
            }
        }
    }
}
