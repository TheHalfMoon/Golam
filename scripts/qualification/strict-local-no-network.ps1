param(
    [string]$Binary = "target\debug\golamd.exe"
)

$ErrorActionPreference = "Stop"
$root = Join-Path $env:RUNNER_TEMP ("golam-net-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $root | Out-Null
$stderrLog = Join-Path $root "golamd.stderr"
$stdoutLog = Join-Path $root "golamd.stdout"
$process = $null
$previousRoot = $env:GOLAM_ROOT

try {
    $env:GOLAM_ROOT = $root
    $process = Start-Process -FilePath $Binary -ArgumentList "--foreground" -PassThru `
        -RedirectStandardError $stderrLog -RedirectStandardOutput $stdoutLog
    $env:GOLAM_ROOT = $previousRoot

    function Assert-NoInternetSockets {
        if ($process.HasExited) {
            throw "golamd exited before locality observation completed"
        }
        $pattern = "\s$($process.Id)\s*$"
        $matches = @(& netstat -ano | Select-String -Pattern $pattern)
        if ($matches.Count -gt 0) {
            $rendered = ($matches | ForEach-Object { $_.Line }) -join [Environment]::NewLine
            throw "Golam process owns an unexpected Internet socket:`n$rendered"
        }
    }

    $ready = $false
    for ($i = 0; $i -lt 100; $i++) {
        Assert-NoInternetSockets
        $stderr = if (Test-Path $stderrLog) { Get-Content -Raw $stderrLog } else { "" }
        if ($stderr -match "golamd: listening on") {
            $ready = $true
            break
        }
        Start-Sleep -Milliseconds 50
    }

    if (-not $ready) {
        $stderr = if (Test-Path $stderrLog) { Get-Content -Raw $stderrLog } else { "" }
        throw "golamd did not reach local IPC readiness. stderr: $stderr"
    }

    for ($i = 0; $i -lt 40; $i++) {
        Assert-NoInternetSockets
        Start-Sleep -Milliseconds 50
    }

    Write-Output "STRICT_LOCAL_INET_SOCKETS=0"
    Write-Output "LOCAL_IPC_LISTENER=OBSERVED"
}
finally {
    $env:GOLAM_ROOT = $previousRoot
    if ($null -ne $process -and -not $process.HasExited) {
        Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
        $process.WaitForExit()
    }
    Remove-Item -Recurse -Force $root -ErrorAction SilentlyContinue
}
