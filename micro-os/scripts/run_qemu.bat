@echo off
REM ==============================================================================
REM Run Micro-OS with QEMU and Intel e1000 Live Internet Networking
REM ==============================================================================

set ISO_PATH=%~dp0..\micro_os.iso

echo [Micro-OS] Booting %ISO_PATH% with Intel e1000 NIC and User Internet Gateway...

where qemu-system-x86_64 >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo [!] qemu-system-x86_64 not found in PATH.
    echo Please install QEMU or add it to PATH.
    echo Alternatively, open Oracle VM VirtualBox or VMware and attach micro_os.iso as a virtual optical disc.
    pause
    exit /b 1
)

qemu-system-x86_64 -m 128M -cdrom "%ISO_PATH%" -net nic,model=e1000,macaddr=52:54:00:12:34:56 -net user -boot d
