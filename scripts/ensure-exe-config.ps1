$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$candidateDirs = @(
  (Join-Path $repoRoot "src-tauri\target\debug"),
  (Join-Path $repoRoot "src-tauri\target\release")
)

$defaultConfig = @{
  instanceName = "Sidecar"
  mcp = @{
    host = "127.0.0.1"
    port = 39333
  }
  latex = @{
    engine = "xelatex"
    compileTimeoutSeconds = 60
  }
  gallery = @{
    defaultSidebarMode = "groups"
  }
  paths = @{
    dataDir = "./data"
  }
}

$found = @()

foreach ($dir in $candidateDirs) {
  if (!(Test-Path -LiteralPath $dir)) {
    continue
  }

  $exe = Get-ChildItem -LiteralPath $dir -Filter "*.exe" -File -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -notmatch "build-script|deps" } |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1

  if (!$exe) {
    continue
  }

  $configPath = Join-Path $exe.DirectoryName "sidecar.config.json"
  $dataDir = Join-Path $exe.DirectoryName "data"

  if (!(Test-Path -LiteralPath $configPath)) {
    $json = $defaultConfig | ConvertTo-Json -Depth 8
    [System.IO.File]::WriteAllText($configPath, $json, [System.Text.UTF8Encoding]::new($false))
  }

  New-Item -ItemType Directory -Force -Path $dataDir | Out-Null
  New-Item -ItemType Directory -Force -Path (Join-Path $dataDir "artifacts") | Out-Null
  New-Item -ItemType Directory -Force -Path (Join-Path $dataDir "logs") | Out-Null
  New-Item -ItemType Directory -Force -Path (Join-Path $dataDir "debug-artifacts") | Out-Null

  $found += [pscustomobject]@{
    Exe = $exe.FullName
    Config = $configPath
    Data = $dataDir
  }
}

if ($found.Count -eq 0) {
  $searched = $candidateDirs -join [Environment]::NewLine
  throw "No built exe found. Searched:$([Environment]::NewLine)$searched$([Environment]::NewLine)Run pnpm tauri build or cargo build first."
}

$found | Format-List
