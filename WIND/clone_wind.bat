@echo off
REM Клонує цей репозиторій у папку WORKING_COPY всередині WIND
REM Запустіть цей файл з папки WIND або з кореня репозиторію.
setlocal
:: Визначаємо директорію скрипта (папка WIND)
set SCRIPT_DIR=%~dp0
set WORKDIR=%SCRIPT_DIR%WORKING_COPY

if exist "%WORKDIR%" (
  echo WORKING_COPY вже існує за адресою %WORKDIR%
  echo Щоб оновити: cd "%WORKDIR%" && git pull
  exit /b 0
)

echo Клонуємо репозиторій у %WORKDIR%...
git clone https://github.com/MrShadowkill/JBRS "%WORKDIR%"
echo Готово.
