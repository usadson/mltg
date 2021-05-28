$prev_dir = Convert-Path .
$dir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $dir

fxc.exe /nologo /T vs_5_0 /E vs_main /Fo tex.vs tex.hlsl
fxc.exe /nologo /T ps_5_0 /E ps_main /Fo tex.ps tex.hlsl

Set-Location $prev_dir
