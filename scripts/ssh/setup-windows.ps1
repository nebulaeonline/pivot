param([switch]$Elevated, [string]$Stage, [string]$LoginUser, [string]$UserHome)
$ErrorActionPreference = 'Stop'

if (-not $Elevated) {
    if ($env:COMPUTERNAME -eq 'ROBPC') { throw 'Run this inside the intended Windows guest, not on ROBPC.' }
    $repo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
    $public = Join-Path $repo 'artifacts\validation-ssh\validation.pub'
    if (-not (Test-Path -LiteralPath $public)) { throw 'Validation public key is missing from shared storage.' }
    Write-Host 'This installs/starts OpenSSH Server, adds the validation public key, requires key authentication, and permits TCP22 from the local subnet.'
    Write-Host 'Existing authorized keys are preserved. Windows will request administrator approval. PowerShell 7 is not needed.'
    if ((Read-Host 'Set up SSH in this Windows guest? [y/N]') -notmatch '^(y|yes)$') { exit 2 }
    # Stage locally before elevation: elevated sessions may not have the J: mapping.
    $stagePath = Join-Path $env:LOCALAPPDATA ('PivotValidation\ssh-setup\' + [guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $stagePath -Force | Out-Null
    Copy-Item -LiteralPath $PSCommandPath -Destination (Join-Path $stagePath 'setup.ps1')
    Copy-Item -LiteralPath $public -Destination (Join-Path $stagePath 'validation.pub')
    $arguments = '-NoProfile -ExecutionPolicy Bypass -File "' + (Join-Path $stagePath 'setup.ps1') + '" -Elevated -Stage "' + $stagePath + '" -LoginUser "' + $env:USERNAME + '" -UserHome "' + $env:USERPROFILE + '"'
    $process = Start-Process -FilePath powershell.exe -Verb RunAs -WindowStyle Hidden -ArgumentList $arguments -Wait -PassThru
    $destination = Join-Path $repo ('artifacts\validation-endpoints\' + $env:COMPUTERNAME + '-ssh-' + [guid]::NewGuid().ToString('N').Substring(0,8))
    New-Item -ItemType Directory -Path (Split-Path $destination) -Force | Out-Null
    Copy-Item -LiteralPath $stagePath -Destination $destination -Recurse
    Write-Host ('Setup report: ' + $destination)
    if (Test-Path -LiteralPath (Join-Path $stagePath 'result.json')) { Get-Content -LiteralPath (Join-Path $stagePath 'result.json') }
    exit $process.ExitCode
}

$code = 1
$configBackup = $null
$result = [ordered]@{ hostname=$env:COMPUTERNAME; user=$LoginUser; status='Failed'; changes=@() }
Start-Transcript -Path (Join-Path $Stage 'setup.log') | Out-Null
try {
    $principal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
    if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { throw 'Administrator approval required.' }
    $key = (Get-Content -LiteralPath (Join-Path $Stage 'validation.pub') -Raw).Trim()
    if ($key -notmatch '^ssh-ed25519 [A-Za-z0-9+/=]+(?: [^\r\n]+)?$') { throw 'Expected one Ed25519 public key.' }
    $capability = Get-WindowsCapability -Online -Name 'OpenSSH.Server~~~~0.0.1.0'
    if ($capability.State -ne 'Installed') {
        Add-WindowsCapability -Online -Name 'OpenSSH.Server~~~~0.0.1.0' | Out-Null
        $result.changes += 'Installed OpenSSH Server'
    }
    $openssh = Join-Path $env:SystemRoot 'System32\OpenSSH'
    $configRoot = Join-Path $env:ProgramData 'ssh'
    New-Item -ItemType Directory -Path $configRoot -Force | Out-Null
    # sshd checks directory write permissions at startup, beyond sshd -t.
    # Back up ACLs and restrict writes to SYSTEM/Administrators, not ordinary users.
    $aclBackup = @()
    foreach ($directory in @($configRoot, (Join-Path $configRoot 'logs'))) {
        if (-not (Test-Path -LiteralPath $directory)) { continue }
        $aclBackup += [ordered]@{path=$directory; sddl=(Get-Acl -LiteralPath $directory).Sddl}
        $aclBackup | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $Stage 'directory-acls-before.json') -Encoding UTF8
        $acl = New-Object Security.AccessControl.DirectorySecurity
        $acl.SetAccessRuleProtection($true, $false)
        $acl.SetOwner((New-Object Security.Principal.SecurityIdentifier('S-1-5-32-544')))
        foreach ($sid in @('S-1-5-18','S-1-5-32-544')) {
            $identity = New-Object Security.Principal.SecurityIdentifier($sid)
            $access = New-Object Security.AccessControl.FileSystemAccessRule($identity, 'FullControl', 'ContainerInherit,ObjectInherit', 'None', 'Allow')
            $acl.AddAccessRule($access)
        }
        # Directory-only read access; do not grant ordinary users access to host keys.
        $users = New-Object Security.Principal.SecurityIdentifier('S-1-5-32-545')
        $acl.AddAccessRule((New-Object Security.AccessControl.FileSystemAccessRule($users, 'ReadAndExecute', 'None', 'None', 'Allow')))
        Set-Acl -LiteralPath $directory -AclObject $acl
        $result.changes += ('Restricted SSH directory write permissions: ' + $directory)
    }
    $config = Join-Path $configRoot 'sshd_config'
    if (-not (Test-Path -LiteralPath $config)) { Copy-Item -LiteralPath (Join-Path $openssh 'sshd_config_default') -Destination $config }
    $stamp = Get-Date -Format 'yyyyMMddTHHmmssfff'
    $configBackup = $config + '.pivot-backup-' + $stamp
    Copy-Item -LiteralPath $config -Destination $configBackup
    $old = Get-Content -LiteralPath $config -Raw
    $old = [regex]::Replace($old, '(?ms)^# BEGIN PIVOT VALIDATION\r?\n.*?^# END PIVOT VALIDATION\r?\n', '')
    # Capture the actual SYSTEM/service startup, rather than only an administrator
    # foreground process. LOCAL0 writes OpenSSH logs under ProgramData\ssh\logs.
    $block = "# BEGIN PIVOT VALIDATION`r`nPubkeyAuthentication yes`r`nPasswordAuthentication no`r`nAuthenticationMethods publickey`r`nSyslogFacility LOCAL0`r`nLogLevel DEBUG3`r`n# END PIVOT VALIDATION`r`n"
    [IO.File]::WriteAllText($config, $block + $old, (New-Object Text.UTF8Encoding($false)))
    & (Join-Path $openssh 'ssh-keygen.exe') -A
    if ($LASTEXITCODE -ne 0) { throw 'Host-key generation failed.' }
    # ssh-keygen runs as the approving administrator, while sshd runs as SYSTEM.
    # Remove the creator's explicit ACE and ownership from private host keys.
    # Keep key bytes/fingerprints unchanged; save only original ACL metadata.
    $hostKeyAclBackup = @()
    foreach ($hostKey in @(Get-ChildItem -LiteralPath $configRoot -File -Filter 'ssh_host_*_key')) {
        $hostKeyAclBackup += [ordered]@{path=$hostKey.FullName; sddl=(Get-Acl -LiteralPath $hostKey.FullName).Sddl}
        $hostKeyAclBackup | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $Stage 'host-key-acls-before.json') -Encoding UTF8
        $keyAcl = New-Object Security.AccessControl.FileSecurity
        $keyAcl.SetAccessRuleProtection($true,$false)
        $keyAcl.SetOwner((New-Object Security.Principal.SecurityIdentifier('S-1-5-18')))
        foreach ($sid in @('S-1-5-18','S-1-5-32-544')) {
            $identity = New-Object Security.Principal.SecurityIdentifier($sid)
            $keyAcl.AddAccessRule((New-Object Security.AccessControl.FileSystemAccessRule($identity,'FullControl','Allow')))
        }
        Set-Acl -LiteralPath $hostKey.FullName -AclObject $keyAcl
    }
    $result.changes += 'Set private host-key owner to SYSTEM and access to SYSTEM/Administrators only; key contents retained'
    # Install for the normal account and the default Windows administrators rule.
    $userSsh = Join-Path $UserHome '.ssh'
    New-Item -ItemType Directory -Path $userSsh -Force | Out-Null
    foreach ($path in @((Join-Path $userSsh 'authorized_keys'), (Join-Path $configRoot 'administrators_authorized_keys'))) {
        [string]$existing = if (Test-Path -LiteralPath $path) { Get-Content -LiteralPath $path -Raw } else { '' }
        if (-not $existing.Contains(($key -split '\s+')[1])) {
            [IO.File]::AppendAllText($path, "`r`n" + $key + "`r`n", (New-Object Text.UTF8Encoding($false)))
        }
    }
    $adminKeys = Join-Path $configRoot 'administrators_authorized_keys'
    & icacls.exe $adminKeys /inheritance:r /grant:r '*S-1-5-32-544:F' '*S-1-5-18:F'
    if ($LASTEXITCODE -ne 0) { throw 'Administrator key ACL setup failed.' }
    & (Join-Path $openssh 'sshd.exe') -t
    if ($LASTEXITCODE -ne 0) { throw 'sshd_config failed validation; backup retained.' }
    $effective = & (Join-Path $openssh 'sshd.exe') -T -C ('user=' + $LoginUser.ToLowerInvariant() + ',host=localhost,addr=127.0.0.1')
    if ($LASTEXITCODE -ne 0 -or $effective -notcontains 'passwordauthentication no' -or $effective -notcontains 'authenticationmethods publickey') { throw 'Effective configuration is not key-only; service not started by this script.' }
    $rule = Get-NetFirewallRule -Name 'OpenSSH-Server-In-TCP' -ErrorAction SilentlyContinue
    if ($rule) { $rule | Set-NetFirewallRule -Enabled True -Profile Any -RemoteAddress LocalSubnet }
    else { New-NetFirewallRule -Name 'OpenSSH-Server-In-TCP' -DisplayName 'OpenSSH Server (Pivot validation)' -Enabled True -Profile Any -Direction Inbound -Protocol TCP -LocalPort 22 -Action Allow -RemoteAddress LocalSubnet | Out-Null }
    Set-Service sshd -StartupType Automatic
    try {
        if ((Get-Service sshd).Status -eq 'Running') { Restart-Service sshd } else { Start-Service sshd }
    } finally {
        $logDirectory = Join-Path $configRoot 'logs'
        if (Test-Path -LiteralPath $logDirectory) {
            $serviceLogs = Join-Path $Stage 'service-logs'
            New-Item -ItemType Directory -Path $serviceLogs -Force | Out-Null
            Get-ChildItem -LiteralPath $logDirectory -File -Filter '*.log' | ForEach-Object {
                Get-Content -LiteralPath $_.FullName -Tail 500 -ErrorAction Continue |
                    Set-Content -LiteralPath (Join-Path $serviceLogs $_.Name) -Encoding UTF8
            }
        }
    }
    # Keep useful file logging, but avoid ongoing verbose per-connection traces.
    $runningConfig = (Get-Content -LiteralPath $config -Raw).Replace('LogLevel DEBUG3', 'LogLevel INFO')
    [IO.File]::WriteAllText($config, $runningConfig, (New-Object Text.UTF8Encoding($false)))
    Restart-Service sshd
    $result.changes += 'Installed public key; key-only authentication; local-subnet SSH firewall rule; automatic sshd service'
    $result.status = 'Ready'
    $result.addresses = @(Get-NetIPAddress -AddressFamily IPv4 | Where-Object IPAddress -ne '127.0.0.1' | Select-Object IPAddress,InterfaceAlias)
    $result.host_fingerprint = & (Join-Path $openssh 'ssh-keygen.exe') -lf (Join-Path $configRoot 'ssh_host_ed25519_key.pub')
    $result.key_fingerprint = & (Join-Path $openssh 'ssh-keygen.exe') -lf (Join-Path $Stage 'validation.pub')
    $code = 0
} catch {
    $result.error = $_.Exception.Message
    Write-Output $_
    # Collect before restoring configuration; never include private-key contents.
    try {
        Get-CimInstance Win32_Service -Filter "Name='sshd'" |
            Select-Object Name,State,StartName,PathName,ExitCode,ServiceSpecificExitCode |
            ConvertTo-Json | Set-Content -LiteralPath (Join-Path $Stage 'service.json') -Encoding UTF8
        foreach ($logName in @('OpenSSH/Admin','OpenSSH/Operational','System','Application')) {
            try {
                $filter = @{LogName=$logName; StartTime=(Get-Date).AddMinutes(-15)}
                if ($logName -eq 'System') { $filter.ProviderName = 'Service Control Manager' }
                Get-WinEvent -FilterHashtable $filter -MaxEvents 30 -ErrorAction Stop |
                    Select-Object TimeCreated,Id,ProviderName,Message | Format-List |
                    Out-String -Width 240 | Set-Content -LiteralPath (Join-Path $Stage ($logName.Replace('/','-') + '.txt')) -Encoding UTF8
            } catch { $_.Exception.Message | Set-Content -LiteralPath (Join-Path $Stage ($logName.Replace('/','-') + '.txt')) -Encoding UTF8 }
        }
    } catch { $result.diagnostic_error = $_.Exception.Message }
    if ($configBackup -and (Test-Path -LiteralPath $configBackup)) {
        try { Copy-Item -LiteralPath $configBackup -Destination $config -Force; $result.config_restored = $true }
        catch { $result.config_restore_error = $_.Exception.Message }
    }
}
finally {
    $result | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $Stage 'result.json') -Encoding UTF8
    Stop-Transcript | Out-Null
}
exit $code
