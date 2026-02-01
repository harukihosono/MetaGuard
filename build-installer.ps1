# MetaGuard Secure Installer Build Script v2.0

Write-Host "================================================" -ForegroundColor Cyan
Write-Host "     MetaGuard Installer Builder v2.0           " -ForegroundColor Cyan
Write-Host "================================================" -ForegroundColor Cyan
Write-Host ""

# 1. Create directories
Write-Host "[1/8] Preparing directories..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path "installer" | Out-Null
New-Item -ItemType Directory -Force -Path "resources" | Out-Null

# 2. Copy manifest file
Write-Host "[2/8] Preparing manifest file..." -ForegroundColor Yellow
if (Test-Path "MetaGuard.exe.manifest") {
    Copy-Item "MetaGuard.exe.manifest" "resources\" -Force
    Write-Host "  OK: Manifest file copied" -ForegroundColor Green
} else {
    Write-Host "  ! Manifest file not found" -ForegroundColor Yellow
}

# 3. Build Rust program
Write-Host "[3/8] Building Rust program..." -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "  ERROR: Build failed" -ForegroundColor Red
    exit 1
}
Write-Host "  OK: Build completed (VC++ runtime embedded)" -ForegroundColor Green

# 4. Create sample config file
Write-Host "[4/8] Creating sample config file..." -ForegroundColor Yellow

$sampleConfig = ";============================================================`r`n"
$sampleConfig += ";          MetaGuard Config (MetaGuard.ini)`r`n"
$sampleConfig += ";============================================================`r`n"
$sampleConfig += "`r`n"
$sampleConfig += "[Settings]`r`n"
$sampleConfig += "CheckInterval=30`r`n"
$sampleConfig += "AutoStart=OFF`r`n"
$sampleConfig += "`r`n"
$sampleConfig += "[MT4_MT5]`r`n"
$sampleConfig += "; MT_1=ON|XM MT4|C:\\Program Files\\XM MT4\\terminal.exe`r`n"

[System.IO.File]::WriteAllText("MetaGuard.ini.sample", $sampleConfig, [System.Text.Encoding]::UTF8)
Write-Host "  OK: Sample config file created" -ForegroundColor Green

# 5. Check icon file
Write-Host "[5/8] Checking icon file..." -ForegroundColor Yellow
if (-not (Test-Path "icon.ico")) {
    Write-Host "  ! Icon file not found" -ForegroundColor Yellow
}

# 6. Create LICENSE file
Write-Host "[6/8] Creating license file..." -ForegroundColor Yellow

$licenseText = "MIT License`r`n`r`nCopyright (c) 2024 Haruki Hosono`r`n`r`n"
$licenseText += "Permission is hereby granted, free of charge, to any person obtaining a copy "
$licenseText += "of this software and associated documentation files (the `"Software`"), to deal "
$licenseText += "in the Software without restriction, including without limitation the rights "
$licenseText += "to use, copy, modify, merge, publish, distribute, sublicense, and/or sell "
$licenseText += "copies of the Software, and to permit persons to whom the Software is "
$licenseText += "furnished to do so, subject to the following conditions:`r`n`r`n"
$licenseText += "The above copyright notice and this permission notice shall be included in all "
$licenseText += "copies or substantial portions of the Software.`r`n`r`n"
$licenseText += "THE SOFTWARE IS PROVIDED `"AS IS`", WITHOUT WARRANTY OF ANY KIND."

[System.IO.File]::WriteAllText("LICENSE", $licenseText, [System.Text.Encoding]::UTF8)
Write-Host "  OK: License file created" -ForegroundColor Green

# 7. Check build result
Write-Host "[7/8] Checking build result..." -ForegroundColor Yellow
if (Test-Path "target\release\MetaGuard.exe") {
    $fileInfo = Get-Item "target\release\MetaGuard.exe"
    $sizeMB = [math]::Round($fileInfo.Length / 1MB, 2)
    Write-Host "  OK: MetaGuard.exe: $sizeMB MB" -ForegroundColor Green
} else {
    Write-Host "  ERROR: Build file not found" -ForegroundColor Red
    exit 1
}

# 8. Create installer with Inno Setup
Write-Host "[8/8] Creating installer..." -ForegroundColor Yellow

$innoSetupPaths = @(
    "C:\Program Files (x86)\Inno Setup 6\ISCC.exe",
    "C:\Program Files\Inno Setup 6\ISCC.exe"
)

$isccPath = $null
foreach ($path in $innoSetupPaths) {
    if (Test-Path $path) {
        $isccPath = $path
        break
    }
}

if ($isccPath) {
    Write-Host "  Running Inno Setup..." -ForegroundColor Yellow
    & "$isccPath" setup.iss

    if ($LASTEXITCODE -eq 0) {
        Write-Host ""
        Write-Host "================================================" -ForegroundColor Green
        Write-Host "        Installer created successfully!         " -ForegroundColor Green
        Write-Host "================================================" -ForegroundColor Green
        Write-Host ""
        Write-Host "Installer location:" -ForegroundColor Cyan
        Write-Host "  installer\" -ForegroundColor White
    } else {
        Write-Host "  ERROR: Installer creation failed" -ForegroundColor Red
    }
} else {
    Write-Host "  ERROR: Inno Setup not found" -ForegroundColor Red
    Write-Host "  Please install from: https://jrsoftware.org/isdl.php" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Press Enter to exit..."
Read-Host
