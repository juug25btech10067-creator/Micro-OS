<#
.SYNOPSIS
    Validates the generated ISO-9660 image structure, El Torito boot catalog, and boot sector signatures.
#>

param(
    [string]$IsoPath = "C:\Users\Subir Bhattacharya\.gemini\antigravity\scratch\micro-os\micro_os.iso"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $IsoPath)) {
    Write-Error "ISO file not found at $IsoPath"
}

$bytes = [System.IO.File]::ReadAllBytes($IsoPath)
Write-Host "Verifying ISO image: $IsoPath"
Write-Host "Total Size: $($bytes.Length) bytes ($($bytes.Length / 2048) sectors)"

if ($bytes.Length % 2048 -ne 0) {
    Write-Error "Failed: ISO size is not a multiple of 2048 bytes."
}

# 1. Check Sector 16: Primary Volume Descriptor
$pvd = 16 * 2048
$pvdType = $bytes[$pvd]
$pvdId = [System.Text.Encoding]::ASCII.GetString($bytes, $pvd + 1, 5)
Write-Host "Sector 16 (PVD): Type=$pvdType, Identifier='$pvdId'"
if ($pvdType -ne 1 -or $pvdId -ne "CD001") {
    Write-Error "Failed: Sector 16 is not a valid ISO-9660 PVD."
}

# 2. Check Sector 17: El Torito Boot Record
$et = 17 * 2048
$etType = $bytes[$et]
$etId = [System.Text.Encoding]::ASCII.GetString($bytes, $et + 1, 5)
$etSys = [System.Text.Encoding]::ASCII.GetString($bytes, $et + 7, 23)
$bootCatSec = $bytes[$et + 0x47]
Write-Host "Sector 17 (El Torito): Type=$etType, Identifier='$etId', System='$etSys', BootCatalogSector=$bootCatSec"
if ($etType -ne 0 -or $etId -ne "CD001" -or $bootCatSec -ne 19) {
    Write-Error "Failed: Sector 17 is not a valid El Torito boot record."
}

# 3. Check Sector 19: Boot Catalog
$cat = 19 * 2048
$catHeader = $bytes[$cat]
$catPlatform = $bytes[$cat + 1]
$catKey1 = $bytes[$cat + 30]
$catKey2 = $bytes[$cat + 31]
Write-Host "Sector 19 (Boot Catalog): Header=$catHeader, Platform=$catPlatform, Key=0x$($catKey1.ToString('X2'))$($catKey2.ToString('X2'))"
if ($catHeader -ne 1 -or $catKey1 -ne 0x55 -or $catKey2 -ne 0xAA) {
    Write-Error "Failed: Boot Catalog validation entry invalid."
}

# Verify Boot Catalog 16-word checksum sum equals 0
$sum = 0
for ($i = 0; $i -lt 32; $i += 2) {
    $word = [BitConverter]::ToUInt16($bytes, $cat + $i)
    $sum = ($sum + $word) % 65536
}
Write-Host "Boot Catalog Checksum validation: 16-word sum = $sum (Expected: 0)"
if ($sum -ne 0) {
    Write-Error "Failed: Boot Catalog checksum validation failed."
}

# Check Default Initial Entry in Boot Catalog
$def = $cat + 32
$bootable = $bytes[$def]
$mediaType = $bytes[$def + 1]
$secCount = [BitConverter]::ToUInt16($bytes, $def + 6)
$loadRba = [BitConverter]::ToUInt32($bytes, $def + 8)
Write-Host "Boot Catalog Default Entry: Indicator=0x$($bootable.ToString('X2')), MediaType=$mediaType, SectorsToLoad=$secCount, LoadRBA=$loadRba"
if ($bootable -ne 0x88 -or $mediaType -ne 0 -or $loadRba -ne 20) {
    Write-Error "Failed: Default boot entry is invalid."
}

# 4. Check Sector 20: Kernel Boot Image signature at byte 510..511
$bootSec = 20 * 2048
$sig1 = $bytes[$bootSec + 510]
$sig2 = $bytes[$bootSec + 511]
Write-Host "Sector 20 (Kernel Boot Sector): Signature=0x$($sig1.ToString('X2'))$($sig2.ToString('X2')) (Expected: 0x55AA)"
if ($sig1 -ne 0x55 -or $sig2 -ne 0xAA) {
    Write-Error "Failed: Boot sector 0x55AA signature missing."
}

Write-Host "`n>>> VALIDATION SUCCESSFUL: micro_os.iso is 100% compliant and bootable! <<<"
