$prev_dir = Convert-Path .
$dir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $dir

dxc.exe -nologo -T vs_6_0 -E vs_main -Fo tex.vs tex.hlsl
dxc.exe -nologo -T ps_6_0 -E ps_main -Fo tex.ps tex.hlsl

Set-Location $prev_dir
