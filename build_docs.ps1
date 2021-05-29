$prev_dir = Convert-Path .
$dir = Split-Path -Parent $MyInvocation.MyCommand.Path

Set-Location $dir/mltg/book
mdbook build -d ../../docs

Set-Location $dir/mltg
cargo doc --no-deps

Set-Location $dir
Copy-Item -Recurse .\target\doc .\docs\api

Set-Location $prev_dir
