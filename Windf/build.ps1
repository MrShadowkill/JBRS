param(
    [ValidateSet('x86_64-pc-windows-msvc', 'x86_64-pc-windows-gnu')]
    [string]$Target = 'x86_64-pc-windows-msvc'
)
$ErrorActionPreference = 'Stop'
Push-Location $PSScriptRoot
try {
    if (-not [Environment]::Is64BitOperatingSystem) { throw 'Windows x64 is required.' }
    foreach ($tool in @('cargo', 'rustup')) {
        if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) {
            throw "Install Rust from https://rustup.rs/ and reopen the terminal ($tool missing)."
        }
    }
    if ($Target.EndsWith('msvc') -and -not (Get-Command cl.exe -ErrorAction SilentlyContinue)) {
        throw 'Run build.cmd, or use the x64 Visual Studio Developer PowerShell with C++ Build Tools installed.'
    }
    if ($Target.EndsWith('gnu') -and -not (Get-Command x86_64-w64-mingw32-gcc -ErrorAction SilentlyContinue)) {
        throw 'Add the MSYS2 MinGW x64 toolchain to PATH, or use build.cmd for MSVC.'
    }
    foreach ($line in Get-Content 'vendor/SDL2/SHA256SUMS') {
        $parts = $line -split '  ', 2
        $actual = (Get-FileHash (Join-Path 'vendor/SDL2' $parts[1]) -Algorithm SHA256).Hash
        if ($actual -ne $parts[0]) { throw "SDL2 integrity check failed: $($parts[1])" }
    }
    & rustup target add $Target
    if ($LASTEXITCODE -ne 0) { throw 'Rust target installation failed.' }
    & cargo build --locked --release --target $Target --target-dir "$PSScriptRoot/target"
    if ($LASTEXITCODE -ne 0) { throw 'Game build failed.' }
    New-Item -ItemType Directory -Force 'dist' | Out-Null
    Copy-Item "target/$Target/release/jailbreak-rs.exe" 'dist/jailbreak-rs.exe' -Force
    Copy-Item 'vendor/SDL2/bin/SDL2.dll' 'dist/SDL2.dll' -Force
    Copy-Item 'portable-run.cmd' 'dist/Start.cmd' -Force
    Copy-Item 'LICENSE', 'THIRD_PARTY_NOTICES.md', 'README_WINDOWS.md' 'dist/' -Force
    Copy-Item 'vendor/SDL2/LICENSE.txt' 'dist/SDL2-LICENSE.txt' -Force
    Copy-Item 'vehicles/assets/ATTRIBUTION.md' 'dist/VEHICLE_ATTRIBUTION.md' -Force
    Compress-Archive -Path 'dist/*' -DestinationPath 'JBRS-Windows11-x64.zip' -Force
    Write-Host 'Ready: dist/Start.cmd and JBRS-Windows11-x64.zip'
} finally {
    Pop-Location
}
