[CmdletBinding(SupportsShouldProcess)]
param(
    [switch]$InstallToolchain,
    [switch]$InstallLocalAi,
    [switch]$DownloadModels
)

$ErrorActionPreference = 'Stop'
$DataRoot = Join-Path $env:LOCALAPPDATA 'com.englishaicoach.desktop'
$Directories = @('database', 'models\whisper', 'voices', 'logs', 'temporary_audio', 'tools\whisper', 'tools\piper')

function Test-Command([string]$Name) { return [bool](Get-Command $Name -ErrorAction SilentlyContinue) }
function Show-Status([string]$Name, [bool]$Ready, [string]$Detail) {
    $Mark = if ($Ready) { '[READY]' } else { '[MISSING]' }
    Write-Host "$Mark $Name - $Detail"
}
function Install-WingetPackage([string]$Id, [string[]]$ExtraArguments = @()) {
    if ($PSCmdlet.ShouldProcess($Id, 'Install package with winget')) {
        & winget install --id $Id --exact --accept-package-agreements --accept-source-agreements --silent @ExtraArguments
        if ($LASTEXITCODE -ne 0) { throw "winget could not install $Id (exit $LASTEXITCODE)." }
    }
}

Write-Host 'English AI Coach - local setup diagnostics'
Write-Host "Runtime data: $DataRoot"
foreach ($RelativePath in $Directories) { New-Item -ItemType Directory -Force -Path (Join-Path $DataRoot $RelativePath) | Out-Null }

$RustReady = Test-Command 'rustc'
$CmakeReady = Test-Command 'cmake'
$GitReady = Test-Command 'git'
$OllamaReady = Test-Command 'ollama'
$PythonReady = Test-Command 'python'
$VsWhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$CppReady = (Test-Path -LiteralPath $VsWhere) -and [bool](& $VsWhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath)

Show-Status 'Rust' $RustReady $(if ($RustReady) { (& rustc --version) } else { 'Rust stable MSVC is required' })
Show-Status 'Visual C++ Build Tools' $CppReady $(if ($CppReady) { 'C++ workload installed' } else { 'Desktop development with C++ is required' })
Show-Status 'CMake' $CmakeReady $(if ($CmakeReady) { (& cmake --version | Select-Object -First 1) } else { 'Required to build whisper.cpp' })
Show-Status 'Git' $GitReady $(if ($GitReady) { (& git --version) } else { 'Required to fetch whisper.cpp source' })
Show-Status 'Ollama' $OllamaReady $(if ($OllamaReady) { (& ollama --version) } else { 'Local LLM runtime not installed' })
Show-Status 'Python' $PythonReady $(if ($PythonReady) { (& python --version) } else { 'Required by the current Piper package' })

if ($InstallToolchain) {
    if (-not (Test-Command 'winget')) { throw 'winget is required for assisted prerequisite installation.' }
    if (-not $RustReady) { Install-WingetPackage 'Rustlang.Rustup' }
    if (-not $CmakeReady) { Install-WingetPackage 'Kitware.CMake' }
    if (-not $CppReady) {
        Install-WingetPackage 'Microsoft.VisualStudio.2022.BuildTools' @('--override', '--wait --passive --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended')
    }
    Write-Host 'Open a new terminal before building after toolchain installation.'
}

if ($InstallLocalAi) {
    if (-not $OllamaReady) { Install-WingetPackage 'Ollama.Ollama' }
    if (-not $PythonReady) { Install-WingetPackage 'Python.Python.3.12' }
    if (-not (Test-Command 'cmake') -or -not (Test-Command 'git')) { throw 'Install Git and CMake before building whisper.cpp.' }
    $SourceRoot = Join-Path $PSScriptRoot '..\local-ai\whisper.cpp'
    if (-not (Test-Path -LiteralPath $SourceRoot)) { & git clone --depth 1 --branch v1.9.1 https://github.com/ggml-org/whisper.cpp.git $SourceRoot }
    & cmake -S $SourceRoot -B (Join-Path $SourceRoot 'build') -DGGML_NATIVE=ON -DWHISPER_BUILD_TESTS=OFF
    & cmake --build (Join-Path $SourceRoot 'build') --config Release --parallel
    $WhisperExe = Get-ChildItem -LiteralPath (Join-Path $SourceRoot 'build') -Recurse -Filter whisper-cli.exe | Select-Object -First 1
    if (-not $WhisperExe) { throw 'whisper-cli.exe was not produced.' }
    Copy-Item -LiteralPath $WhisperExe.FullName -Destination (Join-Path $DataRoot 'tools\whisper\whisper-cli.exe') -Force

    & python -m pip install --user 'piper-tts==1.4.2'
    $PythonScripts = (& python -c "import sysconfig; print(sysconfig.get_path('scripts'))").Trim()
    $PiperExe = Join-Path $PythonScripts 'piper.exe'
    if (-not (Test-Path -LiteralPath $PiperExe)) { throw "Piper launcher not found at $PiperExe" }
    Copy-Item -LiteralPath $PiperExe -Destination (Join-Path $DataRoot 'tools\piper\piper.exe') -Force
}

if ($DownloadModels) {
    Write-Host 'Explicit model download selected. Approximate total: 4 GB.'
    if (-not (Test-Command 'ollama')) { throw 'Install Ollama first with -InstallLocalAi.' }
    & ollama pull qwen3.5:4b
    $WhisperDirectory = Join-Path $DataRoot 'models\whisper'
    Invoke-WebRequest 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin' -OutFile (Join-Path $WhisperDirectory 'ggml-base.en.bin')
    Invoke-WebRequest 'https://huggingface.co/ggml-org/whisper-vad/resolve/main/ggml-silero-v6.2.0.bin' -OutFile (Join-Path $WhisperDirectory 'ggml-silero-v6.2.0.bin')
    & python -m piper.download_voices --data-dir (Join-Path $DataRoot 'voices') en_US-lessac-medium
}

Write-Host 'Setup check complete. No cloud API keys are required.'
