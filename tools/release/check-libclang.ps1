# Resolve LIBCLANG_PATH + MSVC/Windows SDK INCLUDE for ethean-storage/rocksdb (Windows)

# Usage:
#   . .\tools\release\check-libclang.ps1
#   cargo test -p ethean-storage --features rocksdb
#
# Exit 0 when libclang.dll is found and INCLUDE is set (or already usable).
# Exit 1 when libclang is missing. Exit 2 when stdarg.h cannot be located.

# Avoid `$ErrorActionPreference = Stop` so callers can dot-source before cargo
# (cargo writes progress to stderr and Stop would abort the parent shell).

function Find-LibClangDir {
  $candidates = @(
    $env:LIBCLANG_PATH,
    "C:\Program Files\LLVM\bin",
    "C:\Program Files (x86)\LLVM\bin"
  ) | Where-Object { $_ }
  foreach ($dir in $candidates) {
    $dll = Join-Path $dir "libclang.dll"
    if (Test-Path $dll) { return $dir }
  }
  return $null
}

function Find-MsvcInclude {
  $roots = @(
    "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2017\Community\VC\Tools\MSVC",
    "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2019\BuildTools\VC\Tools\MSVC",
    "${env:ProgramFiles}\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC",
    "${env:ProgramFiles}\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC"
  )
  foreach ($root in $roots) {
    if (-not (Test-Path $root)) { continue }
    $ver = Get-ChildItem $root | Sort-Object Name -Descending | Select-Object -First 1
    if (-not $ver) { continue }
    $inc = Join-Path $ver.FullName "include"
    if (Test-Path (Join-Path $inc "stdarg.h")) { return $inc }
  }
  return $null
}

function Find-WinSdkIncludes {
  $sdkRoot = "${env:ProgramFiles(x86)}\Windows Kits\10\Include"
  if (-not (Test-Path $sdkRoot)) { return @() }
  $ver = Get-ChildItem $sdkRoot | Sort-Object Name -Descending | Select-Object -First 1
  if (-not $ver) { return @() }
  @(
    (Join-Path $ver.FullName "ucrt"),
    (Join-Path $ver.FullName "shared"),
    (Join-Path $ver.FullName "um")
  ) | Where-Object { Test-Path $_ }
}

$clang = Find-LibClangDir
if (-not $clang) {
  Write-Host "libclang.dll not found."
  Write-Host "Install LLVM (https://releases.llvm.org/) or set LIBCLANG_PATH to the bin dir."
  exit 1
}
$env:LIBCLANG_PATH = $clang
Write-Host "FOUND libclang under $clang"
Write-Host "Set: `$env:LIBCLANG_PATH = '$clang'"

$msvcInc = Find-MsvcInclude
if (-not $msvcInc) {
  Write-Host "stdarg.h not found under known MSVC Toolsets."
  Write-Host "Install VS Build Tools (C++ workload) so bindgen can see C headers."
  Write-Host "Then re-run this script before: cargo test -p ethean-storage --features rocksdb"
  exit 2
}

$parts = @($msvcInc) + (Find-WinSdkIncludes)
$env:INCLUDE = ($parts -join ";")
Write-Host "Set: `$env:INCLUDE = '$($env:INCLUDE)'"
Write-Host "Ready for: cargo test -p ethean-storage --features rocksdb"
exit 0
