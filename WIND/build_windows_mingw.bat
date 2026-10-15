@echo off
setlocal

echo ===== JBRS Windows 11 (MinGW) build =====

where rustup >nul 2>nul
if errorlevel 1 (
    echo Rust is not installed. Install Rust: https://rustup.rs/
    exit /b 1
)

rustup default stable
rustup target add x86_64-pc-windows-gnu

where pacman >nul 2>nul
if errorlevel 1 (
    echo MSYS2 was not found in PATH. Install it from https://www.msys2.org/
    echo Then run: pacman -S mingw-w64-x86_64-SDL2 mingw-w64-x86_64-toolchain
    exit /b 1
)

echo Building release target...
cargo build --target x86_64-pc-windows-gnu --release
echo.
echo Build finished. The executable will be in target/x86_64-pc-windows-gnu/release/
pause
