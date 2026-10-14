fn main() {
    println!("cargo:rustc-link-arg=-l:libSDL2-2.0.so.0");
    println!("cargo:rustc-link-arg=-l:libGL.so.1");
}
