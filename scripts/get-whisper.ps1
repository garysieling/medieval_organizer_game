# Fetch a private whisper-cli + the tiny model - no admin rights, no C++
# toolchain, so any agent can run it. dev-voice (push-to-talk feedback,
# #1083) transcribes with .tools/whisper/whisper-cli.exe against
# assets/models/ggml-tiny.en.bin; both paths print at the end.
$ErrorActionPreference = "Stop"
$tools = Join-Path $PSScriptRoot "..\.tools\whisper"
$models = Join-Path $PSScriptRoot "..\assets\models"
New-Item -ItemType Directory -Force $tools | Out-Null
New-Item -ItemType Directory -Force $models | Out-Null

# The last release that shipped Windows binaries (1.9.x publishes source
# only): pin it so the fetch is reproducible
# The zip drops its payload under Release/ (the top-level whisper-cli.exe
# is a deprecation stub; the real one keeps its DLLs beside it)
$exe = Join-Path $tools "Release\whisper-cli.exe"
if (!(Test-Path $exe)) {
    $zip = Join-Path $tools "whisper-bin-x64.zip"
    Invoke-WebRequest -Uri "https://github.com/ggerganov/whisper.cpp/releases/download/v1.8.2/whisper-bin-x64.zip" -OutFile $zip
    Expand-Archive -Path $zip -DestinationPath $tools -Force
    Remove-Item $zip
    # the zip's exe has been both "whisper-cli.exe" and "main.exe"
    if (!(Test-Path $exe)) {
        $exe = Join-Path $tools "Release\main.exe"
    }
}
if (!(Test-Path $exe)) { Write-Error "whisper-cli.exe not found after extract - check the release layout"; exit 1 }

$model = Join-Path $models "ggml-tiny.en.bin"
if (!(Test-Path $model)) {
    Invoke-WebRequest -Uri "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin" -OutFile $model
}
if (!(Test-Path $model)) { Write-Error "model download failed"; exit 1 }

Write-Output "WHISPER:$exe"
Write-Output "MODEL:$model"
