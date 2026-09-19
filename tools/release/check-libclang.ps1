# Check LIBCLANG_PATH for ethean-storage/rocksdb builds (Windows)

$ErrorActionPreference = "Stop"
$candidates = @(
  $env:LIBCLANG_PATH,
  "C:\Program Files\LLVM\bin",
  "C:\Program Files (x86)\LLVM\bin"
) | Where-Object { $_ }

foreach ($dir in $candidates) {
  $dll = Join-Path $dir "libclang.dll"
  if (Test-Path $dll) {
    Write-Host "FOUND $dll"
    Write-Host "Set: `$env:LIBCLANG_PATH = '$dir'"
    exit 0
  }
}

Write-Host "libclang.dll not found."
Write-Host "Install LLVM (https://releases.llvm.org/) or set LIBCLANG_PATH to the bin dir that contains libclang.dll."
Write-Host "Then: cargo test -p ethean-storage --features rocksdb"
exit 1
