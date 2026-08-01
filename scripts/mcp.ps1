# mcp.ps1 - drive the running game's in-client dev MCP surface from PowerShell.
#
# WHY THIS EXISTS. CLAUDE.md S Agent walks says Claude launches the game, teleports,
# measures and screenshots - "the user does NOT launch the game and does NOT teleport
# themselves". That assumes the dev surface is reachable. It is a streamable-HTTP MCP
# server at http://127.0.0.1:7777/mcp, and when it is NOT wired in as a session MCP
# server there is otherwise no way to drive it, which blocks the whole walk protocol.
# Written 2026-08-01 during the two-body-plan walk (journal/0130) after exactly that.
#
# USAGE
#   . .\scripts\mcp.ps1
#   Mcp-Init                                   # handshake; prints the session id
#   Mcp-Tools                                  # list tool names
#   Mcp-Call 'client_player_pose_get' @{}
#   Mcp-Call 'character_spawn_character' @{ name='a'; pos=@{x=0;y=40;z=0}; body_plan='dc:body/biped' }
#   Mcp-Call 'client_screenshot' @{ name='0130-two-plans-idle' }
#
# TWO TRAPS PAID FOR ALREADY, both costing a debugging cycle:
#   1. Do NOT name a parameter $args - it is a PowerShell automatic variable and it
#      silently corrupts the payload (the symptom is a bare -32601 "method not found"
#      on tools/call while tools/list works fine).
#   2. Invoke-WebRequest may hand back .Content as byte[] rather than string; decode
#      before parsing or .Trim() throws. Responses are SSE frames - parse "data:" lines.
#
# Units and conventions the tools use (CLAUDE.md S Agent walks): pose_* speaks METRES,
# world_fill/scan_region/get_contents speak VOXELS; yaw/pitch are RADIANS; check
# eye_in_solid in every pose response before trusting a screenshot; screenshot names
# are bare lowercase slugs and land in journal/assets/.
#
# NOTE: there is no character teleport verb on this surface. Two characters walked in
# the same direction collide and superimpose - spawn them in place instead (see
# journal/0131's walk section).

$script:McpUri = 'http://127.0.0.1:7777/mcp'
$script:McpSid = $null
$script:McpId = 100

function Mcp-Post([string]$body) {
    $h = @{ Accept = 'application/json, text/event-stream' }
    if ($script:McpSid) { $h['Mcp-Session-Id'] = $script:McpSid }
    $r = Invoke-WebRequest -Uri $script:McpUri -Method POST -TimeoutSec 60 -ContentType 'application/json' `
        -Headers $h -Body $body -ErrorAction Stop
    if (-not $script:McpSid -and $r.Headers['Mcp-Session-Id']) { $script:McpSid = [string]$r.Headers['Mcp-Session-Id'] }
    $body = $r.Content
    if ($body -is [byte[]]) { $body = [System.Text.Encoding]::UTF8.GetString($body) }
    $body = [string]$body
    $script:McpRaw = $body
    # SSE frames: pull the last "data: {json}" line that parses
    $out = $null
    foreach ($line in ($body -split "`n")) {
        $t = $line.Trim()
        if ($t.StartsWith('data:')) {
            $j = $t.Substring(5).Trim()
            if ($j -and $j.StartsWith('{')) { try { $out = $j | ConvertFrom-Json } catch {} }
        }
    }
    if (-not $out -and $body.Trim().StartsWith('{')) { try { $out = $body | ConvertFrom-Json } catch {} }
    return $out
}

function Mcp-Init {
    $script:McpSid = $null
    $init = Mcp-Post '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"design-thread","version":"1"}}}'
    Mcp-Post '{"jsonrpc":"2.0","method":"notifications/initialized"}' | Out-Null
    "session $script:McpSid"
}

function Mcp-Tools {
    $r = Mcp-Post '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
    $r.result.tools | ForEach-Object { $_.name }
}

function Mcp-Call([string]$name, [hashtable]$toolArgs = @{}) {
    $script:McpId++
    $payload = @{ jsonrpc = '2.0'; id = $script:McpId; method = 'tools/call'
                  params = @{ name = $name; arguments = $toolArgs } } | ConvertTo-Json -Depth 12 -Compress
    $script:McpSent = $payload
    $r = Mcp-Post $payload
    if ($r.error) { return "ERROR: $($r.error.message)" }
    $txt = ($r.result.content | Where-Object { $_.type -eq 'text' } | ForEach-Object { $_.text }) -join "`n"
    if ($r.result.isError) { return "TOOL-ERROR: $txt" }
    return $txt
}
