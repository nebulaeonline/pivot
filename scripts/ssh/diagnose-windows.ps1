param([switch]$Elevated, [switch]$SystemProbe, [string]$Stage)
$ErrorActionPreference = 'Stop'
if (-not $Elevated -and -not $SystemProbe) {
    if ($env:COMPUTERNAME -eq 'ROBPC') { throw 'Run inside the Windows guest.' }
    $repo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
    $stagePath = Join-Path $env:LOCALAPPDATA ('PivotValidation\ssh-diagnostic\' + [guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $stagePath -Force | Out-Null
    Copy-Item -LiteralPath $PSCommandPath -Destination (Join-Path $stagePath 'diagnose.ps1')
    Write-Host 'UAC is required. Compares administrator/SYSTEM startup on loopback, collects service metadata and port 22 ownership. No service/configuration changes. Temporary SYSTEM task is removed afterward.'
    $arguments = '-NoProfile -ExecutionPolicy Bypass -File "' + (Join-Path $stagePath 'diagnose.ps1') + '" -Elevated -Stage "' + $stagePath + '"'
    $process = Start-Process powershell.exe -Verb RunAs -WindowStyle Hidden -ArgumentList $arguments -Wait -PassThru
    $destination = Join-Path $repo ('artifacts\validation-endpoints\' + $env:COMPUTERNAME + '-debug-' + [guid]::NewGuid().ToString('N').Substring(0,8))
    New-Item -ItemType Directory -Path (Split-Path $destination) -Force | Out-Null
    Copy-Item -LiteralPath $stagePath -Destination $destination -Recurse
    Write-Host ('Diagnostic returned: ' + $destination)
    exit $process.ExitCode
}
$result = [ordered]@{hostname=$env:COMPUTERNAME; service_changed=$false; configuration_changed=$false}
$child = $null
$taskName = $null
$code = 1
try {
    $binary = Join-Path $env:SystemRoot 'System32\OpenSSH\sshd.exe'
    $root = Join-Path $env:ProgramData 'ssh'
    $config = Join-Path $root 'sshd_config'
    $result.identity = [Security.Principal.WindowsIdentity]::GetCurrent().Name
    if ($SystemProbe -and [Security.Principal.WindowsIdentity]::GetCurrent().User.Value -ne 'S-1-5-18') { throw 'SYSTEM probe must run as SYSTEM.' }
    if (-not $SystemProbe) {
        Get-CimInstance Win32_Service -Filter "Name='sshd'" |
            Select-Object Name,State,StartName,PathName,ExitCode,ServiceSpecificExitCode |
            ConvertTo-Json | Set-Content -LiteralPath (Join-Path $Stage 'service.json')
        foreach ($query in @('qc','qsidtype','qprivs','qfailure')) {
            & sc.exe $query sshd 2>&1 | Out-File -LiteralPath (Join-Path $Stage ('service-' + $query + '.txt'))
        }
        Get-NetTCPConnection -State Listen -LocalPort 22 -ErrorAction SilentlyContinue |
            Select-Object LocalAddress,LocalPort,OwningProcess |
            ConvertTo-Json | Set-Content -LiteralPath (Join-Path $Stage 'port22.json')
    }
    (Get-Item -LiteralPath $binary).VersionInfo | Format-List FileVersion,ProductVersion |
        Out-String | Set-Content -LiteralPath (Join-Path $Stage 'version.txt')
    # ACL metadata only: never read or export private host-key contents.
    @($root, (Join-Path $root 'logs'), $config) + @(Get-ChildItem -LiteralPath $root -Filter 'ssh_host_*_key' | ForEach-Object FullName) |
        ForEach-Object {
            if (Test-Path -LiteralPath $_) {
                $acl = Get-Acl -LiteralPath $_
                [pscustomobject]@{path=$_; owner=$acl.Owner; sddl=$acl.Sddl}
            }
        } | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $Stage 'acls.json')
    # PS5 can throw on the first native stderr line with Stop, losing the rest
    # of an OpenSSH warning. Process redirection preserves the whole diagnostic.
    $configProbe = Start-Process -FilePath $binary -ArgumentList '-T' -PassThru -WindowStyle Hidden -Wait -RedirectStandardOutput (Join-Path $Stage 'effective-config.txt') -RedirectStandardError (Join-Path $Stage 'config-errors.txt')
    $result.config_probe_exit_code = $configProbe.ExitCode
    if (Get-NetTCPConnection -State Listen -LocalPort 49222 -ErrorAction SilentlyContinue) { throw 'Diagnostic loopback port already occupied; no process launched.' }
    # A separate minimal config guarantees no additional configured ListenAddress
    # exposes the diagnostic. The installed config is inspected with -T above.
    $diagnosticConfig = Join-Path $Stage 'diagnostic-config'
    @('Port 49222','ListenAddress 127.0.0.1','PasswordAuthentication no','PubkeyAuthentication yes','AuthenticationMethods publickey') |
        Set-Content -LiteralPath $diagnosticConfig -Encoding ASCII
    $arguments = '-ddd -e -f "' + $diagnosticConfig + '"'
    $child = Start-Process -FilePath $binary -ArgumentList $arguments -PassThru -WindowStyle Hidden -RedirectStandardOutput (Join-Path $Stage 'stdout.txt') -RedirectStandardError (Join-Path $Stage 'sshd-debug.txt')
    $exited = $child.WaitForExit(10000)
    if ($exited) { $child.Refresh(); $result.exit_code=$child.ExitCode; $result.outcome='Exited' }
    else {
        $result.outcome='Still running at diagnostic deadline'
        $result.listeners = @(Get-NetTCPConnection -OwningProcess $child.Id -State Listen -ErrorAction SilentlyContinue | Select-Object LocalAddress,LocalPort)
    }
    if (-not $child.HasExited) { $child.Kill(); $child.WaitForExit(); $result.owned_process_stopped=$true }
    if ($SystemProbe) {
        # Compare the installed config with the setup's key-only authentication,
        # changing only listener/port for containment. Refuse includes rather than
        # risk an included file adding a non-loopback listener.
        $installed = Get-Content -LiteralPath $config
        if ($installed | Where-Object { $_ -match '^\s*Include\s' }) {
            $result.installed_config_probe='Skipped: Include requires inspection before loopback sanitization'
        } else {
            $installedConfig = Join-Path $Stage 'installed-loopback-config'
            $header = @('Port 49222','ListenAddress 127.0.0.1','PasswordAuthentication no','PubkeyAuthentication yes','AuthenticationMethods publickey')
            $header + @($installed | Where-Object { $_ -notmatch '^\s*(Port|ListenAddress)\s' }) |
                Set-Content -LiteralPath $installedConfig -Encoding UTF8
            # Windows PowerShell UTF8 includes a BOM; OpenSSH needs UTF8 without BOM.
            $contents = Get-Content -LiteralPath $installedConfig -Raw
            [IO.File]::WriteAllText($installedConfig,$contents,(New-Object Text.UTF8Encoding($false)))
            $arguments = '-ddd -e -f "' + $installedConfig + '"'
            $child = Start-Process -FilePath $binary -ArgumentList $arguments -PassThru -WindowStyle Hidden -RedirectStandardOutput (Join-Path $Stage 'installed-stdout.txt') -RedirectStandardError (Join-Path $Stage 'installed-debug.txt')
            if ($child.WaitForExit(10000)) { $child.Refresh(); $result.installed_config_exit_code=$child.ExitCode }
            else { $result.installed_config_probe='Still running at diagnostic deadline' }
            if (-not $child.HasExited) { $child.Kill(); $child.WaitForExit(); $result.installed_process_stopped=$true }
        }
    }
    if (-not $SystemProbe) {
        # The SYSTEM task executes only from a fresh administrator/SYSTEM-owned
        # directory. Never point a privileged task at user-writable script files.
        $systemRoot = Join-Path $env:ProgramData ('PivotSshDiagnostic-' + [guid]::NewGuid().ToString('N'))
        New-Item -ItemType Directory -Path $systemRoot | Out-Null
        $systemAcl = New-Object Security.AccessControl.DirectorySecurity
        $systemAcl.SetAccessRuleProtection($true,$false)
        $systemAcl.SetOwner((New-Object Security.Principal.SecurityIdentifier('S-1-5-32-544')))
        foreach ($sid in @('S-1-5-18','S-1-5-32-544')) {
            $identity = New-Object Security.Principal.SecurityIdentifier($sid)
            $systemAcl.AddAccessRule((New-Object Security.AccessControl.FileSystemAccessRule($identity,'FullControl','ContainerInherit,ObjectInherit','None','Allow')))
        }
        Set-Acl -LiteralPath $systemRoot -AclObject $systemAcl
        $systemScript = Join-Path $systemRoot 'diagnose.ps1'
        Copy-Item -LiteralPath $PSCommandPath -Destination $systemScript
        $taskName = 'PivotSshDiagnostic-' + [guid]::NewGuid().ToString('N')
        $action = New-ScheduledTaskAction -Execute (Join-Path $env:SystemRoot 'System32\WindowsPowerShell\v1.0\powershell.exe') -Argument ('-NoProfile -ExecutionPolicy Bypass -File "' + $systemScript + '" -SystemProbe -Stage "' + $systemRoot + '"')
        $principal = New-ScheduledTaskPrincipal -UserId 'SYSTEM' -LogonType ServiceAccount -RunLevel Highest
        $settings = New-ScheduledTaskSettingsSet -ExecutionTimeLimit (New-TimeSpan -Seconds 35) -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries
        Register-ScheduledTask -TaskName $taskName -Action $action -Principal $principal -Settings $settings | Out-Null
        Start-ScheduledTask -TaskName $taskName
        $deadline = (Get-Date).AddSeconds(45)
        do {
            Start-Sleep -Milliseconds 250
            $taskState = (Get-ScheduledTask -TaskName $taskName).State
            $taskInfo = Get-ScheduledTaskInfo -TaskName $taskName
        } while ((-not (Test-Path -LiteralPath (Join-Path $systemRoot 'result.json')) -or $taskState -eq 'Running') -and (Get-Date) -lt $deadline)
        $result.system_task_result = $taskInfo.LastTaskResult
        if ($taskState -eq 'Running') { Stop-ScheduledTask -TaskName $taskName; $result.system_task_timeout=$true }
        Copy-Item -LiteralPath $systemRoot -Destination (Join-Path $Stage 'system-probe') -Recurse
        if (-not (Test-Path -LiteralPath (Join-Path $systemRoot 'result.json'))) { throw 'SYSTEM task did not return a result; task status captured.' }
        # Preserve native evidence; remove only the transient task registration.
        $result.system_native_report = $systemRoot
    }
    $code = 0
} catch { $result.error=$_.Exception.Message }
finally {
    if ($taskName) { Unregister-ScheduledTask -TaskName $taskName -Confirm:$false -ErrorAction Continue }
    if ($child) {
        $child.Refresh()
        if (-not $child.HasExited) { $child.Kill(); $child.WaitForExit(); $result.owned_process_stopped=$true }
    }
    $result | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $Stage 'result.json') -Encoding UTF8
}
exit $code
