@echo off
REM ==============================================================================
REM Run Micro-OS with Oracle VM VirtualBox (Intel e1000 Live Internet Networking)
REM ==============================================================================

set VM_NAME=MicroOS_Rust
set ISO_PATH=%~dp0..\micro_os.iso
set VBOX_MANAGE="C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"

if not exist %VBOX_MANAGE% (
    echo [!] VirtualBox not found at default installation path (%VBOX_MANAGE%).
    pause
    exit /b 1
)

echo [*] Cleaning up any previous VM named '%VM_NAME%'...
%VBOX_MANAGE% controlvm "%VM_NAME%" poweroff >nul 2>nul
%VBOX_MANAGE% unregistervm "%VM_NAME%" --delete >nul 2>nul

echo [*] Creating VirtualBox VM '%VM_NAME%'...
%VBOX_MANAGE% createvm --name "%VM_NAME%" --ostype "Other_64" --register
%VBOX_MANAGE% modifyvm "%VM_NAME%" --memory 128 --vram 16 --boot1 dvd --boot2 none --boot3 none --boot4 none
%VBOX_MANAGE% modifyvm "%VM_NAME%" --nic1 nat --nictype1 82540EM
%VBOX_MANAGE% storagectl "%VM_NAME%" --name "IDE Controller" --add ide
%VBOX_MANAGE% storageattach "%VM_NAME%" --storagectl "IDE Controller" --port 0 --device 0 --type dvddrive --medium "%ISO_PATH%"

echo.
echo [*] Launching Micro-OS in VirtualBox...
%VBOX_MANAGE% startvm "%VM_NAME%"

echo.
echo [OK] Micro-OS VM started successfully!
pause
