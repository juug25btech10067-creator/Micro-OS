@echo off
REM ==============================================================================
REM 1-Click Push Micro-OS to GitHub
REM ==============================================================================

cd /d "%~dp0"
echo [*] Current Directory: %CD%
echo [*] Remote URL: https://github.com/juug25btech10067-creator/Micro-OS.git
echo.
echo [*] Pushing branch 'main' to GitHub...
echo (If prompted, please sign in via the browser window that opens)
echo.

git push -u origin main

echo.
if %ERRORLEVEL% EQU 0 (
    echo [SUCCESS] Your Micro-OS repository has been pushed to GitHub!
    echo Visit: https://github.com/juug25btech10067-creator/Micro-OS
) else (
    echo [ERROR] Push failed. Please check your GitHub permissions or sign-in.
)
echo.
pause
