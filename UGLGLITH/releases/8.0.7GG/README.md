# Глітчед версія 8.0.7GG

- [Windows x64 ZIP](UGLGLITH-8.0.7GG-Windows-x64.zip): розпакуйте весь архів, включно з UGLGLITH.exe і SDL2.dll, запустіть Start.cmd.
- [Linux x64 tar.gz](UGLGLITH-8.0.7GG-Linux-x64.tar.gz): розпакуйте, запустіть ./Start.sh. Потрібні glibc >= 2.35, SDL2 та OpenGL 2.1; Rust не потрібен.
- [Android ARM64 APK](UGLGLITH-8.0.7GG-Android-arm64.apk): Android 10+, GLES2; окремий пакет dev.jbrs.uglglith. Підписано локальним preview-сертифікатом.

При кожному відкритті: статичне жирне червоне попередження про фоточутливу епілепсію. ОК активується через 10 секунд видимого екрана. Підтвердження не зберігається. Escape/Back закриває гру.

Сцена перемикається після глітчу або через 15 секунд активної гри. F6 перемикає вручну.

[SHA256SUMS](SHA256SUMS). Linux перевірено через Mesa/Xvfb; APK: підпис, маніфест, DEX та ARM64 ELF перевірено. Фізичні Android/Windows пристрої не перевірялися.
