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
$script:maxManagedPids = 0

function Get-DescendantProcessIdsFromSnapshot {
    param(
        [int]$RootProcessId,
        [object[]]$Snapshot
    )

    $ids = New-Object 'System.Collections.Generic.HashSet[int]'
    [void]$ids.Add($RootProcessId)
    $changed = $true
    while ($changed) {
        $changed = $false
        foreach ($candidate in $Snapshot) {
            $candidateId = [int]$candidate.ProcessId
            $parentId = [int]$candidate.ParentProcessId
            if ($ids.Contains($parentId) -and -not $ids.Contains($candidateId)) {
                [void]$ids.Add($candidateId)
                $changed = $true
            }
        }
    }
    return @($ids | Sort-Object)
}

function Assert-ObserverSelfTest {
    $snapshot = @(
        [pscustomobject]@{ ProcessId = 200; ParentProcessId = 100 },
        [pscustomobject]@{ ProcessId = 300; ParentProcessId = 200 },
        [pscustomobject]@{ ProcessId = 400; ParentProcessId = 999 },
        [pscustomobject]@{ ProcessId = 500; ParentProcessId = 300 }
    )
    $observed = @(Get-DescendantProcessIdsFromSnapshot -RootProcessId 100 -Snapshot $snapshot)
    $expected = @(100, 200, 300, 500)
    if (($observed -join ',') -ne ($expected -join ',')) {
        throw "process-tree observer self-test failed: expected $($expected -join ','), got $($observed -join ',')"
    }
}

function Get-ManagedProcessIds {
    if ($null -eq $process) {
        return @()
    }
    $snapshot = @(Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId)
    return @(Get-DescendantProcessIdsFromSnapshot -RootProcessId $process.Id -Snapshot $snapshot)
}

function Assert-NoInternetSockets {
    if ($process.HasExited) {
        throw "golamd exited before locality observation completed"
    }

    $managedIds = @(Get-ManagedProcessIds)
    if ($managedIds.Count -gt $script:maxManagedPids) {
        $script:maxManagedPids = $managedIds.Count
    }
    if ($managedIds.Count -lt 1) {
        throw "managed process-tree observer did not capture golamd"
    }

    $netstat = @(& netstat -ano)
    foreach ($managedId in $managedIds) {
        $pattern = "\s$managedId\s*$"
        $matches = @($netstat | Select-String -Pattern $pattern)
        if ($matches.Count -gt 0) {
            $rendered = ($matches | ForEach-Object { $_.Line }) -join [Environment]::NewLine
            throw "Golam managed process tree owns an unexpected Internet socket (pid=$managedId):`n$rendered"
        }
    }
}

Assert-ObserverSelfTest

try {
    $env:GOLAM_ROOT = $root
    $process = Start-Process -FilePath $Binary -ArgumentList "--foreground" -PassThru `
        -RedirectStandardError $stderrLog -RedirectStandardOutput $stdoutLog
    $env:GOLAM_ROOT = $previousRoot

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

    if ($script:maxManagedPids -lt 1) {
        throw "managed process-tree observer did not capture golamd"
    }

    Write-Output "PROCESS_TREE_TRAVERSAL_SELF_TEST=PASS"
    Write-Output "MANAGED_PROCESS_TREE_OBSERVER=ENABLED"
    Write-Output "MAX_MANAGED_PIDS_OBSERVED=$script:maxManagedPids"
    Write-Output "STRICT_LOCAL_INET_SOCKETS=0"
    Write-Output "LOCAL_IPC_LISTENER=OBSERVED"
}
finally {
    $env:GOLAM_ROOT = $previousRoot
    if ($null -ne $process -and -not $process.HasExited) {
        $managedIds = @(Get-ManagedProcessIds | Sort-Object -Descending)
        foreach ($managedId in $managedIds) {
            Stop-Process -Id $managedId -Force -ErrorAction SilentlyContinue
        }
        $process.WaitForExit()
    }
    Remove-Item -Recurse -Force $root -ErrorAction SilentlyContinue
}
