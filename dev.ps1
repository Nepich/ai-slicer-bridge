param([ValidateSet('test','build','run','fmt','clippy')][string]$Task='test')
$env:CARGO_HOME='E:\codex\tools\Rust\cargo'
$env:RUSTUP_HOME='E:\codex\tools\Rust\rustup'
$env:PATH="E:\codex\tools\llvm-mingw\llvm-mingw-20261104-msvcrt-x86_64\bin;$env:CARGO_HOME\bin;$env:PATH"
Push-Location $PSScriptRoot
try {
 if ($Task -eq 'build') { & "$env:CARGO_HOME\bin\cargo.exe" build --release }
 elseif ($Task -eq 'fmt') { & "$env:CARGO_HOME\bin\cargo.exe" fmt --check }
 else { & "$env:CARGO_HOME\bin\cargo.exe" $Task }
 exit $LASTEXITCODE
} finally { Pop-Location }


