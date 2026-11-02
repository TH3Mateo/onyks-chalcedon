@echo off
rem Uploads the built installers to the ONYKS server (the folder the Bloodstone web page offers
rem for download). One ssh connection, so the password is asked for once.
rem
rem   scripts\upload-release.bat            version from package.json
rem   scripts\upload-release.bat 1.3.2      another version
setlocal EnableDelayedExpansion

set HOST=onyks-admin@onyks-server
set REMOTE_DIR=onyks-bloodstone/chalcedon

cd /d "%~dp0.."

set VER=%~1
if "%VER%"=="" (
    for /f "tokens=2 delims=:, " %%v in ('findstr /r /c:"^ *\"version\"" package.json') do set VER=%%~v
)
if "%VER%"=="" (
    echo Cannot read the version from package.json.
    exit /b 1
)

set STAGE=%TEMP%\chalcedon-release-%VER%
rmdir /s /q "%STAGE%" 2>nul
mkdir "%STAGE%"

set MISSING=0
call :stage "src-tauri\target\release\bundle\nsis\ONYKS Chalcedon_%VER%_x64-setup.exe"
call :stage "dist-linux\ONYKS Chalcedon_%VER%_amd64.deb"
call :stage "dist-linux\ONYKS Chalcedon-%VER%-1.x86_64.rpm"
call :stage "dist-linux\onyks-chalcedon-%VER%-1-x86_64.pkg.tar.zst"
if not "%MISSING%"=="0" (
    echo Build the missing installers first. Nothing was uploaded.
    rmdir /s /q "%STAGE%"
    exit /b 1
)

echo.
echo Uploading version %VER% to %HOST%:%REMOTE_DIR%/%VER%/ ...
tar --format ustar -cf - -C "%STAGE%" . | ssh %HOST% "mkdir -p '%REMOTE_DIR%/%VER%' && tar -xf - -C '%REMOTE_DIR%/%VER%' && ls -l '%REMOTE_DIR%/%VER%'"
set RESULT=%ERRORLEVEL%

rmdir /s /q "%STAGE%"
if not "%RESULT%"=="0" (
    echo.
    echo Upload failed. "Permission denied" usually means the folder belongs to root on the server:
    echo   sudo chown -R onyks-admin:onyks-admin ~/onyks-bloodstone/chalcedon
    exit /b %RESULT%
)
echo.
echo Done.
exit /b 0

:stage
if not exist %1 (
    echo MISSING: %~1
    set MISSING=1
    goto :eof
)
copy %1 "%STAGE%\" >nul
goto :eof
