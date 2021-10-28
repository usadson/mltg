$prev_dir = Convert-Path .
$dir = Split-Path -Parent $MyInvocation.MyCommand.Path

Set-Location $dir/../book
mdbook build -d ../docs

Set-Location $prev_dir
