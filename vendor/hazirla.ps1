# Taze klon icin gelistirici hazirlik betigi:
# 1) On kosullari denetler (Rust, LLVM, MSVC)  2) Bagimliliklari indirir  3) Derler.
# Kullanim: powershell -ExecutionPolicy Bypass -File vendor\hazirla.ps1

$ErrorActionPreference = "Stop"
$kok = Split-Path $PSScriptRoot -Parent

Write-Host "=== 1/3 On kosul denetimi ==="
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "HATA: cargo bulunamadi. Rust kurun: https://rustup.rs" -ForegroundColor Red
    exit 1
}
Write-Host ("Rust OK: " + (cargo --version))

$llvmBin = "C:\Program Files\LLVM\bin"
if (-not (Test-Path "$llvmBin\clang-cl.exe")) {
    Write-Host "HATA: clang-cl bulunamadi. LLVM kurun: https://github.com/llvm/llvm-project/releases" -ForegroundColor Red
    Write-Host "Beklenen yol: $llvmBin (farkli yerdeyse vendor\derle.ps1 icindeki yolu guncelleyin)"
    exit 1
}
Write-Host "LLVM OK: $llvmBin"

if (-not (Test-Path "C:\Program Files (x86)\Microsoft Visual Studio")) {
    Write-Host "UYARI: Visual Studio C++ Build Tools bulunamadi gibi gorunuyor." -ForegroundColor Yellow
    Write-Host "Kurulum: https://visualstudio.microsoft.com/downloads/ (Desktop development with C++)"
} else {
    Write-Host "MSVC Build Tools OK"
}

Write-Host "=== 2/3 Bagimliliklarin indirilmesi (internet gerekir) ==="
cargo fetch --manifest-path "$kok\Cargo.toml"
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host "=== 3/3 Derleme (ilk derlemede MuPDF C cekirdegi derlenir, uzun surebilir) ==="
& "$PSScriptRoot\derle.ps1"
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host ""
Write-Host "HAZIR: $kok\release\pdf_goruntuleyici.exe"
Write-Host "Gelistirmeye devam: kok dizinde 'cargo check' / 'cargo build' / 'cargo run'"
