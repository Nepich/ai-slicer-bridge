$ErrorActionPreference='Stop'
$exe=Join-Path $PSScriptRoot 'target\release\ai-slicer-bridge.exe'
$requests=@(
 '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"smoke","version":"1"}}}',
 '{"jsonrpc":"2.0","method":"notifications/initialized"}',
 '{"jsonrpc":"2.0","id":2,"method":"tools/list"}',
 '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"slicer_ui_action","arguments":{"digest":"fake","purpose":"must refuse without observation","action":{"kind":"key","key":"enter"}}}}',
 '{"jsonrpc":"2.0","id":4,"method":"no_such_method"}',
 'bad-json',
 '{"jsonrpc":"2.0","id":5,"method":"ping"}'
)
. (Join-Path $PSScriptRoot 'Invoke-Mcp.ps1')
$lines=@(Invoke-SlicerMcp -Exe $exe -Requests $requests)

$r=@($lines | ForEach-Object {$_ | ConvertFrom-Json})
if ($r.Count -ne 6) {throw "Expected six responses, got $($r.Count)"}
if ($r[0].result.serverInfo.name -ne 'ai-slicer-bridge') {throw 'Wrong server'}
if ($r[1].result.tools.Count -ne 5) {throw 'Wrong tool count'}
if (!$r[2].result.isError -or $r[2].result.content[0].text -notlike '*Inspect first*') {throw 'Action without observation accepted'}
if ($r[3].error.code -ne -32601) {throw 'Method error missing'}
if ($r[4].error.code -ne -32700) {throw 'Parse error missing'}
if ($r[5].id -ne 5) {throw 'Server did not recover from parse error'}
Write-Output 'PASS: MCP handshake, tool schemas, mutation refusal, unknown method, malformed JSON recovery, ping. No UI input performed.'






