$ErrorActionPreference='Stop'
$exe=Join-Path $PSScriptRoot 'target\release\ai-slicer-bridge.exe'
if (!(Test-Path -LiteralPath $exe)) {throw 'Build the release executable first: cargo build --release'}
if (!(Get-Command codex -ErrorAction SilentlyContinue)) {throw 'Codex client CLI not found. Run from the desktop client terminal or add the client CLI to PATH.'}
codex mcp add ai_slicer_bridge --env "AI_SLICER_SETTINGS_PATH=$(Join-Path $PSScriptRoot 'local\preferences.json')" -- $exe
if ($LASTEXITCODE -ne 0) {throw 'Registration failed'}
codex mcp get ai_slicer_bridge
Write-Output 'Registration saved. Restart the desktop app if tools are not visible in the current chat. Then ask the AI to verify the connection.'

