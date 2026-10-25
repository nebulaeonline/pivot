# Read-only, built-in Windows PowerShell 5.1; no admin or Python required.
$ErrorActionPreference = 'Stop'
$repo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$name = $env:COMPUTERNAME + '-' + [guid]::NewGuid().ToString('N').Substring(0,8)
$report = Join-Path $env:LOCALAPPDATA ('PivotValidation\setup-reports\' + $name)
New-Item -ItemType Directory -Path $report -Force | Out-Null
$data = [ordered]@{
    hostname = $env:COMPUTERNAME
    user = $env:USERNAME
    home = $env:USERPROFILE
    source_root = $repo
    machine = (Get-CimInstance Win32_OperatingSystem).OSArchitecture
    addresses = @(Get-NetIPAddress | Where-Object { $_.IPAddress -ne '127.0.0.1' -and $_.IPAddress -ne '::1' } | Select-Object IPAddress,InterfaceAlias)
    sshd = @(Get-Service sshd -ErrorAction SilentlyContinue | Select-Object Name,Status,StartType)
    listeners = @(Get-NetTCPConnection -LocalPort 22 -State Listen -ErrorAction SilentlyContinue | Select-Object LocalAddress,LocalPort)
    changes_made = $false
}
$data | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $report 'endpoint.json') -Encoding UTF8
try {
    Get-NetConnectionProfile | Select-Object InterfaceAlias,NetworkCategory,IPv4Connectivity |
        ConvertTo-Json | Set-Content -LiteralPath (Join-Path $report 'network-profiles.json') -Encoding UTF8
    $rules = @(Get-NetFirewallRule -Name '*OpenSSH*' -ErrorAction Stop)
    $rules | Select-Object Name,Enabled,Profile,Direction,Action |
        ConvertTo-Json | Set-Content -LiteralPath (Join-Path $report 'ssh-firewall-rules.json') -Encoding UTF8
    $rules | Get-NetFirewallAddressFilter | Select-Object InstanceID,LocalAddress,RemoteAddress |
        ConvertTo-Json | Set-Content -LiteralPath (Join-Path $report 'ssh-firewall-addresses.json') -Encoding UTF8
} catch {
    $_.Exception.Message | Set-Content -LiteralPath (Join-Path $report 'network-inspection-error.txt') -Encoding UTF8
}
# Read service failure evidence without starting the service or changing settings.
Get-CimInstance Win32_Service -Filter "Name='sshd'" |
    Select-Object Name,State,StartMode,StartName,PathName,ExitCode,ServiceSpecificExitCode |
    ConvertTo-Json | Set-Content -LiteralPath (Join-Path $report 'service.json') -Encoding UTF8
foreach ($logName in @('OpenSSH/Admin', 'OpenSSH/Operational', 'System')) {
    $fileName = $logName.Replace('/', '-') + '.txt'
    try {
        $filter = @{LogName=$logName; StartTime=(Get-Date).AddHours(-6)}
        if ($logName -eq 'System') { $filter.ProviderName = 'Service Control Manager' }
        Get-WinEvent -FilterHashtable $filter -MaxEvents 40 -ErrorAction Stop |
            Select-Object TimeCreated,Id,LevelDisplayName,Message |
            Format-List | Out-String -Width 240 |
            Set-Content -LiteralPath (Join-Path $report $fileName) -Encoding UTF8
    } catch {
        $_.Exception.Message | Set-Content -LiteralPath (Join-Path $report $fileName) -Encoding UTF8
    }
}
$destination = Join-Path $repo ('artifacts\validation-endpoints\' + $name)
New-Item -ItemType Directory -Path (Split-Path $destination) -Force | Out-Null
Copy-Item -LiteralPath $report -Destination $destination -Recurse
Write-Host ('Read-only SSH report returned: ' + $destination)
