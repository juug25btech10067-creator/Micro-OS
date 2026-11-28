@echo off
REM ==============================================================================
REM Micro-OS 1-Click Direct Runner
REM Boots micro_os.iso directly, runs hardware emulation, and displays live OS screen
REM ==============================================================================

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\run_emulator.ps1" -IsoPath "%~dp0micro_os.iso"
pause
