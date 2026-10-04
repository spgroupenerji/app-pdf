$env:PATH = "C:\Windows\Microsoft.NET\Framework64\v4.0.30319;C:\Program Files\LLVM\bin;" + $env:PATH
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
$env:CC = "C:\Program Files\LLVM\bin\clang-cl.exe"
$env:CXX = "C:\Program Files\LLVM\bin\clang-cl.exe"
$env:UYGULAMA_SURUMU = "v" + (Get-Date -Format "yyyyMMddHHmm")
$kok = Split-Path $PSScriptRoot -Parent
Write-Host "Surum: $env:UYGULAMA_SURUMU"
cargo build --release --manifest-path "$kok\Cargo.toml"
New-Item "$kok\release" -ItemType Directory -Force | Out-Null
Copy-Item "$kok\target\release\pdf_goruntuleyici.exe" -Destination "$kok\release" -Force
