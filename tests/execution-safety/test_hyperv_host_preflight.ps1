# Synthetic provider test: these functions shadow Hyper-V; no VM is accessed.
[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$OutputRoot)
$ErrorActionPreference = 'Stop'
$global:PivotPreflightTestCalls = [System.Collections.Generic.List[string]]::new()
function Get-VMHost {
    [pscustomobject]@{ComputerName='SyntheticHost';EnableEnhancedSessionMode=$true}
}
function Get-VM {
    param([string]$Name)
    $global:PivotPreflightTestCalls.Add("vm:$Name")
    if ($Name -notin @('FirstGuest','SecondGuest')) { throw "Wrong VM argument: $Name" }
    [pscustomobject]@{Name=$Name;Id='synthetic';State='Off';Generation=2;Version='synthetic'}
}
function Get-VMIntegrationService {
    param([string]$VMName)
    $global:PivotPreflightTestCalls.Add("integration:$VMName")
    if ($VMName -notin @('FirstGuest','SecondGuest')) { throw "Wrong integration argument: $VMName" }
    [pscustomobject]@{VMName=$VMName;Name='Synthetic service';Enabled=$true}
}
$collector = Join-Path $PSScriptRoot '../../scripts/qualification/hyperv_host_preflight.ps1'
$output = & $collector -VMName 'FirstGuest','SecondGuest' -OutputRoot $OutputRoot
if ($LASTEXITCODE -ne 0) { throw "Synthetic collector exited $LASTEXITCODE" }
$report = Get-Content -Raw -LiteralPath (Join-Path $output 'host.json') | ConvertFrom-Json
$expected = @('vm:FirstGuest','integration:FirstGuest','vm:SecondGuest','integration:SecondGuest')
if (($global:PivotPreflightTestCalls -join '|') -ne ($expected -join '|')) { throw "Wrong calls: $global:PivotPreflightTestCalls" }
if ($report.checks.Count -ne 5 -or @($report.checks | Where-Object status -ne 'Verified').Count) {
    throw 'Synthetic query did not complete'
}
if ($report.checks[1].data.Name -ne 'FirstGuest' -or $report.checks[3].data.Name -ne 'SecondGuest') {
    throw 'VM output did not retain exact identities'
}
if ($report.headless_qualified -ne 'NotConfigured' -or $report.gui_isolation_approved -ne 'NotConfigured') {
    throw 'Metadata query incorrectly granted qualification/isolation'
}
$manifest = Get-Content -Raw -LiteralPath (Join-Path $output 'origin-hashes.json') | ConvertFrom-Json
$actual = (Get-FileHash -LiteralPath (Join-Path $output 'host.json') -Algorithm SHA256).Hash.ToLowerInvariant()
if ($manifest.files[0].sha256 -ne $actual) { throw 'Origin hash mismatch' }
Write-Output 'PASS: exact guest arguments, result identities, no approval and origin hash (synthetic providers only)'
