<# Read-only host metadata. No guest commands, lifecycle changes or isolation approval. #>
[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string[]]$VMName,
    [Parameter(Mandatory=$true)][string]$OutputRoot
)
$ErrorActionPreference = 'Stop'
if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
    throw 'Unsupported: run this read-only host collector on the Windows Hyper-V host.'
}
$stamp = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffffffZ')
$root = [IO.Path]::GetFullPath($OutputRoot)
$drive = [IO.DriveInfo]::new([IO.Path]::GetPathRoot($root))
if ($drive.DriveType -ne [IO.DriveType]::Fixed -or $drive.DriveFormat -ne 'NTFS') {
    throw 'Use an explicitly chosen local NTFS evidence root, not a network share.'
}
$report = Join-Path $root "hyperv-preflight-$stamp"
New-Item -ItemType Directory -Path $report | Out-Null
$checks = @()
function Read-Check([string]$CheckId, [scriptblock]$Query) {
    try {
        $value = & $Query
        return [pscustomobject]@{ check=$CheckId; status='Verified'; meaning='Metadata query completed; not isolation approval'; data=$value }
    } catch {
        $state = if ($_.Exception -is [System.Management.Automation.CommandNotFoundException] -or
                     $_.CategoryInfo.Category -eq [System.Management.Automation.ErrorCategory]::ObjectNotFound) {
            'NotConfigured'
        } else { 'Blocked' }
        return [pscustomobject]@{ check=$CheckId; status=$state; reason=$_.Exception.Message }
    }
}
$checks += Read-Check 'host-enhanced-session-capability' {
    Get-VMHost | Select-Object ComputerName, EnableEnhancedSessionMode
}
foreach ($guestName in $VMName) {
    $checks += Read-Check "vm:$guestName" {
        Get-VM -Name $guestName | Select-Object Name, Id, State, Generation, Version,
            ProcessorCount, MemoryAssigned, EnhancedSessionTransportType
    }
    $checks += Read-Check "integration-services:$guestName" {
        Get-VMIntegrationService -VMName $guestName |
            Select-Object VMName, Name, Enabled, PrimaryStatusDescription
    }
}
$result = [ordered]@{
    schema='pivot-guest-qualification-preflight-v1'; class='headless'; created_utc=[DateTime]::UtcNow.ToString('o')
    collector_sha256=(Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash.ToLowerInvariant()
    host=$env:COMPUTERNAME; requested_guests=$VMName; checks=$checks
    installed='See each VM query; owner report alone is not verification'
    headless_qualified='NotConfigured'; gui_isolation_approved='NotConfigured'
    limitations=@('Enhanced Session capability is not current redirection state',
        'Integration Services list does not prove clipboard/drive separation',
        'Inspect exact connection settings and record guest session separately',
        'No clipboard, personal profiles, guest commands or VM configuration accessed')
}
$path = Join-Path $report 'host.json'
$result | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $path -Encoding utf8
$hash = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
@{algorithm='SHA-256'; files=@(@{path='host.json'; sha256=$hash})} |
    ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $report 'origin-hashes.json') -Encoding utf8
Write-Output $report
if (@($checks | Where-Object status -ne 'Verified').Count) { exit 2 }
exit 0
