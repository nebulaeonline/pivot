# Read-only, temporary WMI observer. Passed as command text; no policy changes.
$ErrorActionPreference='Stop'
$root=$env:PIVOT_COLLECTOR_REPORT
$work=$env:PIVOT_COLLECTOR_WORK
$writer=$null
$watcher=$null
$subscription=$null
$status='Blocked'
$failure=$null
function Row($process) {
    $created=$process.CreationDate
    if($created -isnot [datetime]) { $created=[System.Management.ManagementDateTimeConverter]::ToDateTime([string]$created) }
    [ordered]@{pid=[int]$process.ProcessId;parent_pid=[int]$process.ParentProcessId;
        start_ticks=$created.ToUniversalTime().Ticks.ToString();start_utc=$created.ToUniversalTime().ToString('o');
        command=[string]$process.CommandLine}
}
function Emit($kind,$rows) {
    $phase=if(Test-Path -LiteralPath "$root\phase.txt") { Get-Content -LiteralPath "$root\phase.txt" -Raw } else { 'before-build' }
    $entry=[ordered]@{kind=$kind;observed_utc=[datetime]::UtcNow.ToString('o');phase=$phase.Trim();processes=@($rows)}
    $writer.WriteLine(($entry | ConvertTo-Json -Depth 5 -Compress));$writer.Flush()
    if($kind -eq 'snapshot') { [IO.File]::WriteAllText("$root\snapshot-phase.txt",$phase.Trim()) }
}
try {
    $writer=[IO.StreamWriter]::new("$root\processes.ndjson",$false,[Text.UTF8Encoding]::new($false))
    # Only Avalonia collectors in Pivot-owned historical/current native work roots.
    $filter="Name='dotnet.exe' AND CommandLine LIKE '%Avalonia.BuildServices.Collector.dll%' AND (CommandLine LIKE '%C:\\PivotWork\\attended-%' OR CommandLine LIKE '%C:\\PivotWork\\collector-build-%')"
    $eventFilter=$filter.Replace('Name=', 'TargetInstance.Name=').Replace('CommandLine LIKE', 'TargetInstance.CommandLine LIKE')
    $watcher=[System.Management.ManagementEventWatcher]::new("SELECT * FROM __InstanceCreationEvent WITHIN 1 WHERE TargetInstance ISA 'Win32_Process' AND $eventFilter")
    $eventId='PivotCollector-'+[guid]::NewGuid().ToString('N')
    $subscription=Register-ObjectEvent -InputObject $watcher -EventName EventArrived -SourceIdentifier $eventId
    $watcher.Start()
    $initial=@(Get-CimInstance Win32_Process -Filter $filter -OperationTimeoutSec 5 | ForEach-Object { Row $_ })
    Emit 'snapshot' $initial
    [IO.File]::WriteAllText("$root\ready.tmp",'{"status":"Ready","creation_observer":"WMI intrinsic WITHIN 1 second; not lossless kernel tracing"}')
    [IO.File]::Move("$root\ready.tmp","$root\ready.json")
    $deadline=[datetime]::UtcNow.AddMinutes(15)
    while(-not (Test-Path -LiteralPath "$root\stop") -and [datetime]::UtcNow -lt $deadline) {
        foreach($event in @(Get-Event -SourceIdentifier $eventId -ErrorAction SilentlyContinue)) {
            Emit 'creation-observed' @(Row $event.SourceEventArgs.NewEvent.TargetInstance)
            Remove-Event -EventIdentifier $event.EventIdentifier
        }
        Emit 'snapshot' @(Get-CimInstance Win32_Process -Filter $filter -OperationTimeoutSec 5 | ForEach-Object { Row $_ })
        Start-Sleep -Milliseconds 500
    }
    # One last snapshot; fixed observation budget is controlled by the caller.
    Emit 'snapshot' @(Get-CimInstance Win32_Process -Filter $filter -OperationTimeoutSec 5 | ForEach-Object { Row $_ })
    $status=if(Test-Path -LiteralPath "$root\stop") { 'Completed' } else { 'Expired' }
} catch {
    $failure=$_.Exception.Message
    if(-not (Test-Path -LiteralPath "$root\ready.json")) {
        [IO.File]::WriteAllText("$root\ready.json",(@{status='Blocked';reason=$failure}|ConvertTo-Json -Compress))
    }
} finally {
    if($watcher) { try {$watcher.Stop()} catch {}; $watcher.Dispose() }
    if($subscription) { Unregister-Event -SourceIdentifier $eventId -ErrorAction SilentlyContinue }
    if($writer) { $writer.Dispose() }
    [IO.File]::WriteAllText("$root\monitor-result.json",(@{status=$status;reason=$failure}|ConvertTo-Json -Compress))
}
