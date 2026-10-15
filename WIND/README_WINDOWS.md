# JBRS — Windows quick start (WIND)

Ця папка містить прості скрипти, щоб швидко клонувати поточний репозиторій у папку WIND і почати працювати під Windows (MinGW або MSVC).

Швидко (1-2 хв):
1. Встанови Git (https://git-scm.com/) і Rust (https://rustup.rs/).
2. Відкрий PowerShell або CMD у корені цього репо і запусти `clone_wind.bat` або `.\clone_wind.ps1`.
3. Перейди в папку `WIND` і запусти `build_mingw.bat` або `build_msvc.bat` залежно від toolchain.

Примітки про toolchain:
- Для MinGW (GNU) рекомендую встановити MSYS2 і пакети mingw-w64 (https://www.msys2.org/). Після інсталяції в консолі MSYS2 виконай: `pacman -Syu` і `pacman -S mingw-w64-x86_64-toolchain`.
- Для MSVC встанови "Build Tools for Visual Studio" (C++ workload).

Детальніші кроки в скриптах нижче.
