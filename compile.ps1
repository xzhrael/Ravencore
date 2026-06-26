# Luca Pro Hybrid Engine - Rust Compilation Script
# Required: Android NDK and cargo-ndk (cargo install cargo-ndk)
# Targets: aarch64-linux-android (arm64-v8a), armv7-linux-androideabi (armeabi-v7a)

Write-Host "Starting Rust cross-compilation for LucaPro..." -ForegroundColor Cyan

# Ensure output directories exist
New-Item -ItemType Directory -Force -Path "libs/arm64-v8a" | Out-Null
New-Item -ItemType Directory -Force -Path "libs/armeabi-v7a" | Out-Null

# Configure environment variables
if (-not $env:ANDROID_NDK_HOME) {
    $env:ANDROID_NDK_HOME = "C:\android-ndk-r27d"
}

# Compile using direct cargo and toolchain linkers
Write-Host "Compiling Rust binaries via direct Cargo..." -ForegroundColor Gray
Push-Location "compile/lucapro_helper"
& "$env:USERPROFILE\.cargo\bin\cargo" build --target aarch64-linux-android --release
$res1 = $LASTEXITCODE
& "$env:USERPROFILE\.cargo\bin\cargo" build --target armv7-linux-androideabi --release
$res2 = $LASTEXITCODE
Pop-Location

if ($res1 -eq 0 -and $res2 -eq 0) {
    # Copy binaries
    Copy-Item -Path "compile/lucapro_helper/target/aarch64-linux-android/release/lucapro_helper" -Destination "libs/arm64-v8a/lucapro_helper" -Force
    Copy-Item -Path "compile/lucapro_helper/target/armv7-linux-androideabi/release/lucapro_helper" -Destination "libs/armeabi-v7a/lucapro_helper" -Force
    Write-Host "Rust Compilation SUCCESS!" -ForegroundColor Green
    Write-Host "Binaries saved to libs/arm64-v8a/ and libs/armeabi-v7a/" -ForegroundColor Gray

    # Packaging Section
    Write-Host "Packaging Magisk/KernelSU module..." -ForegroundColor Cyan
    $version = (Get-Content module.prop | Select-String "^version=").Line.Split("=")[1].Trim()
    $rand4 = Get-Random -Minimum 1000 -Maximum 9999
    $rand6 = -join (1..6 | ForEach-Object { "0123456789abcdef"[(Get-Random -Maximum 16)] })
    $ZIP_NAME = "LucaPro-$version-$rand4-$rand6-release.zip"
    $OUT_PATH = Join-Path (Get-Item .).Parent.FullName $ZIP_NAME

    Compress-Archive -Path "META-INF", "libs", "system", "webroot", "scripts", "module.prop", "system.prop", "post-fs-data.sh", "service.sh", "customize.sh", "uninstall.sh", "toast.apk", "system_monitor.apk", "gamelist.txt" -DestinationPath $OUT_PATH -Force
    Write-Host "Module zipped successfully: $OUT_PATH" -ForegroundColor Green
} else {
    Write-Host "Rust Compilation FAILED!" -ForegroundColor Red
}
