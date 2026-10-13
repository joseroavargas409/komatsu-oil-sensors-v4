$ErrorActionPreference = 'Stop'
$v4Root = $PSScriptRoot
$v4Url = 'http://127.0.0.1:8771'
try {
  $c = Invoke-RestMethod -Uri ($v4Url + '/api/config') -TimeoutSec 2
  if ($c.fixed.duration_s -eq 2.4 -and $c.fixed.mechanism -like '*two serial*') {
    Write-Host ('V4 ya está funcionando: ' + $v4Url)
    exit 0
  }
  throw 'El puerto 8771 pertenece a otro programa.'
} catch {
  if ($_.Exception.Message -like '*pertenece*') { throw }
}
$nodeCommand = Get-Command node -ErrorAction SilentlyContinue
$v4Node = if ($nodeCommand) { $nodeCommand.Source } else { Join-Path $env:USERPROFILE '.cache\codex-runtimes\codex-primary-runtime\dependencies\node\bin\node.exe' }
if (-not (Test-Path -LiteralPath $v4Node)) { throw 'Instala Node.js 22 o posterior desde nodejs.org y vuelve a ejecutar este archivo.' }
Start-Process -FilePath $v4Node -ArgumentList @('server.mjs') -WorkingDirectory $v4Root -WindowStyle Hidden -RedirectStandardOutput (Join-Path $v4Root 'server_stdout.log') -RedirectStandardError (Join-Path $v4Root 'server_stderr.log') | Out-Null
Write-Host ('V4 iniciada. Abre ' + $v4Url + '/?oil=95&sensors=6')
