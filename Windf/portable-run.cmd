@echo off
setlocal
pushd "%~dp0"
if not exist "jbrs.exe" (
    echo jbrs.exe is missing. Extract the entire Windows archive.
    popd
    pause
    exit /b 1
)
if not exist "SDL2.dll" (
    echo SDL2.dll is missing. Extract the entire Windows archive.
    popd
    pause
    exit /b 1
)
"%~dp0jbrs.exe" %*
set "GAME_RESULT=%ERRORLEVEL%"
popd
if not "%GAME_RESULT%"=="0" (
    echo Game exited with code %GAME_RESULT%. See the error above.
    pause
)
exit /b %GAME_RESULT%
