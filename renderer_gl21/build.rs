fn main() {
    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-arg=-l:libSDL2-2.0.so.0");
        println!("cargo:rustc-link-arg=-l:libGL.so.1");
    }

    #[cfg(target_os = "windows")]
    {
        // MinGW/MSYS2 / Windows build: SDL2 + OpenGL are provided by the Windows toolchain.
        println!("cargo:rustc-link-lib=sdl2");
        println!("cargo:rustc-link-lib=opengl32");
    }
}
