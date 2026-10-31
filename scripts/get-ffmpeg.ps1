# Fetch a private ffmpeg into .tools/ - no admin rights needed, so any
# agent can run it. record_video --format mp4|webm --ffmpeg-path <path>
# uses it; the path is printed at the end.
$tools = Join-Path $PSScriptRoot "..\.tools\ffmpeg"
New-Item -ItemType Directory -Force $tools | Out-Null
Push-Location $tools
if (!(Test-Path package.json)) { npm init -y | Out-Null }
npm install ffmpeg-static --no-fund --no-audit | Out-Null
$ff = Join-Path $tools "node_modules\ffmpeg-static\ffmpeg.exe"
Pop-Location
if (!(Test-Path $ff)) { Write-Error "ffmpeg install failed - is node/npm on PATH?"; exit 1 }
Write-Output "FFMPEG:$ff"
