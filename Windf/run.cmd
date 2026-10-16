@echo off
if exist "%~dp0dist\Start.cmd" (
    call "%~dp0dist\Start.cmd" %*
    exit /b
)
echo No Windows build found. Run build.cmd first.
pause
exit /b 1
