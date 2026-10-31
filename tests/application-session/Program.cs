using System.Diagnostics;
using System.Globalization;
using System.Text.Json;
using Pivot.Client;
using Pivot.Protocol.Application.V1;

await ClipboardReadChecks.Run();
if(args.Length==3&&args[0]=="--filtering-checks") { await FilteringChecks.Run(args[1],args[2]);Console.WriteLine("Filtering session checks: PASS");return; }
if(args.Length==1&&args[0]=="--clipboard-read-checks") {
    Console.WriteLine("Clipboard read checks: PASS (delayed text/native, exact Strings, empty text, obsolete/type changes, bounded exhaustion and stale interactions)");
    return;
}

static Position P(uint row, uint col) => new() { Row = row, Column = col };
static Edit N(uint row, uint col, decimal value) => new() { At = P(row,col), Number = WorkbookSession.Encode(value) };
static Edit F(uint row, uint col, string source) => new() { At = P(row,col), Formula = source };
static void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
static Cell At(Response frame, uint row, uint col) => frame.Cells.Single(c => c.At.Row == row && c.At.Column == col);
static async Task Fails(Func<Task> action, bool conflict = false)
{
    try { await action(); throw new Exception("Expected rejection."); }
    catch (WorkbookSessionException e) { Check(!conflict || e.IsConflict, "Expected exact-state conflict."); }
}
static async Task Cancelled(Task action)
{
    try { await action; throw new Exception("Expected cancellation."); }
    catch (OperationCanceledException) { }
}
static async Task CancelledAfterDispatch(WorkbookSession session, Func<CancellationToken,Task> action)
{
    using var cancel = new CancellationTokenSource();
    session.RequestDispatchedForTesting = () => {
        session.RequestDispatchedForTesting = null;
        cancel.Cancel();
    };
    try { await Cancelled(action(cancel.Token)); }
    finally { session.RequestDispatchedForTesting = null; }
}
await FilteringChecks.Run(args[0],args.Length>1 ? Path.GetDirectoryName(Path.GetFullPath(args[1]))! : Path.GetTempPath());
await CapacityChecks.Run(args[0],args.Length>1 ? Path.GetDirectoryName(Path.GetFullPath(args[1]))! : Path.GetTempPath());
await CsvChecks.Run(args[0],args.Length>1 ? Path.GetDirectoryName(Path.GetFullPath(args[1]))! : Path.GetTempPath());
await RoundingChecks.Run(args[0],args.Length>1 ? Path.GetDirectoryName(Path.GetFullPath(args[1]))! : Path.GetTempPath());
await FormattingChecks.Run(args[0],args.Length>1 ? Path.GetDirectoryName(Path.GetFullPath(args[1]))! : Path.GetTempPath());
await RegionChecks.Run(args[0],args.Length>1 ? Path.GetDirectoryName(Path.GetFullPath(args[1]))! : Path.GetTempPath());
await StructureChecks.Run(args[0],args.Length>1 ? Path.GetDirectoryName(Path.GetFullPath(args[1]))! : Path.GetTempPath());
await SortingChecks.Run(args[0],args.Length>1 ? Path.GetDirectoryName(Path.GetFullPath(args[1]))! : Path.GetTempPath());
await TypedChecks.Run(args[0],args.Length>1 ? Path.GetDirectoryName(Path.GetFullPath(args[1]))! : Path.GetTempPath());
await WorksheetChecks.Run(args[0],args.Length>1 ? Path.GetDirectoryName(Path.GetFullPath(args[1]))! : Path.GetTempPath());
await FillChecks.Run(args[0],args.Length>1 ? Path.GetDirectoryName(Path.GetFullPath(args[1]))! : Path.GetTempPath());
await AggregateChecks.Run(args[0],args.Length>1 ? Path.GetDirectoryName(Path.GetFullPath(args[1]))! : Path.GetTempPath());
await StructuredBindingChecks.Run(args[0]);
await ProjectionChecks.Run(args[0]);
await ClipboardChecks.Run(args[0],args.Length>1 ? Path.GetDirectoryName(Path.GetFullPath(args[1]))! : Path.GetTempPath());
await TextChecks.Run(args[0],args.Length>1 ? Path.GetDirectoryName(Path.GetFullPath(args[1]))! : Path.GetTempPath());
await RangeChecks.Run(args[0],args.Length>1 ? Path.GetDirectoryName(Path.GetFullPath(args[1]))! : Path.GetTempPath());
await PersistenceChecks.Run(args[0], args.Length>1 ? Path.GetDirectoryName(Path.GetFullPath(args[1]))! : Path.GetTempPath());
var clock = Stopwatch.StartNew();
var session = await WorkbookSession.StartAsync(args[0]);
double startup = clock.Elapsed.TotalMilliseconds;
try { await session.ApplyAsync("", [N(1,1,1m)]); throw new Exception("Expected explicit revision requirement."); }
catch (ArgumentException) { }
await Fails(()=>session.ApplyAsync(session.Revision,[F(1,1,new string('x',WorkbookSession.MaximumFrame))]));
await session.ApplyAsync(session.Revision,[N(8,8,new decimal(0,0,0,true,3)),N(8,9,decimal.MaxValue),F(8,10,"=I8+1")]);
var exact=await session.ReadAsync(P(8,8),P(8,10));
Check(At(exact,8,8).Number.Negative && At(exact,8,8).Number.Scale==3,"Signed zero metadata survives boundary");
Check(WorkbookSession.Decode(At(exact,8,9).Number)==decimal.MaxValue,"Full Decimal96 coefficient survives boundary");
Check(At(exact,8,10).Error.Length>0,"Rust arithmetic overflow is reported");
await session.ApplyAsync(session.Revision, [N(1,1,1.2300m), N(1,2,2m), F(1,3,"=A1+B1")]);
var frame = await session.ReadAsync(P(1,1),P(1,3));
Check(WorkbookSession.Decode(At(frame,1,3).Value) == 3.23m,"Rust formula arithmetic");
Check(At(frame,1,1).Number.Scale == 4,"Submitted scale retained");
string stable = At(frame,1,1).CellId;
var s = await session.CaptureAsync();
await session.ApplyAsync(session.Revision, [N(1,1,5m)]);
Check(WorkbookSession.Decode(At(await session.ReadAsync(P(1,1),P(1,3)),1,3).Value)==7m,"Dependent edit recalculation");
await session.ApplyAsync(session.Revision, [new Edit { At=P(1,1),MoveTo=P(2,1) },N(1,1,99m)]);
frame = await session.ReadAsync(P(1,1),P(2,3));
Check(At(frame,2,1).CellId==stable && At(frame,1,1).CellId!=stable,"Move/reuse stable identities");
Check(WorkbookSession.Decode(At(frame,1,3).Value)==7m,"Formula stays bound to moved identity");
await session.ApplyAsync(session.Revision, [F(1,3,"=A1+B1"),new Edit { At=P(1,2),Clear=true }]);
Check(WorkbookSession.Decode(At(await session.ReadAsync(P(1,1),P(1,3)),1,3).Value)==99m,"Explicit rebind and clear-zero");
string current = session.Revision;
var old = await session.CalculateAsync(s.Capture,[P(1,3)]);
Check(old.Revision==s.Revision && old.Revision!=current && session.Revision==current,"Old result cannot rewind current revision");
Check(old.Generation.Length>0 && WorkbookSession.Decode(old.Cells[0].Value)==3.23m,"Immutable S survives edits/moves");
await session.ApplyAsync(session.Revision,[F(1,4,"=(")]);
frame=await session.ReadAsync(P(1,1),P(1,4));
Check(At(frame,1,4).Error.Length>0 && At(frame,1,3).Error.Length==0,"Invalid draft isolated in viewport");
await session.ApplyAsync(session.Revision,[F(1,4,"=C1*2")]);
Check(WorkbookSession.Decode(At(await session.ReadAsync(P(1,4),P(1,4)),1,4).Value)==198m,"Draft repair");
current=session.Revision;
await Fails(()=>session.ApplyAsync(current,[N(1,5,9m),new Edit {At=P(2,1),MoveTo=P(1,1)}]));
Check(session.Revision==current,"Failed batch retains revision");
Check((await session.ReadAsync(P(1,5),P(1,5))).Cells.Count==0,"Failed batch rolls back earlier edit");
await Fails(()=>session.ApplyAsync(s.Revision,[N(1,5,8m)]),conflict:true);
await Fails(()=>session.ReadAsync(P(1,1),P(17,17)));
await Fails(()=>session.ReadAsync(P(2,2),P(1,1)));
var captures = new List<string>();
for(int i=0;i<3;i++) captures.Add((await session.CaptureAsync()).Capture);
await Fails(()=>session.CaptureAsync());
foreach(var id in captures) await session.ReleaseAsync(id);
using(var cancelled=new CancellationTokenSource()) {
    cancelled.Cancel();
    await Cancelled(session.ApplyAsync(session.Revision,[N(1,5,88m)],cancelled.Token));
}
Check((await session.ReadAsync(P(1,5),P(1,5))).Cells.Count==0,"Queued cancellation prevents mutation");
{
    ulong before=session.LastCompletedRequestId;
    await CancelledAfterDispatch(session,token=>session.ApplyAsync(session.Revision,[N(4,1,81m)],token));
    Check(session.LastCompletedRequestId==before+1,"Dispatched cancelled edit has a known response");
}
Check(WorkbookSession.Decode((await session.ReadAsync(P(4,1),P(4,1))).Cells[0].Value)==81m,"Cancellation does not roll back an accepted edit");
await CancelledAfterDispatch(session,token=>session.CaptureAsync(token));
captures.Clear();
for(int i=0;i<3;i++) captures.Add((await session.CaptureAsync()).Capture);
foreach(var id in captures) await session.ReleaseAsync(id);

// A real dispatched request is drained despite caller cancellation; the next frame stays aligned.
await session.ApplyAsync(session.Revision,Enumerable.Range(1,256).Select(i=>F((uint)i,6,i==1?"=A1+1":$"=F{i-1}+1")));
var heavy = await session.CaptureAsync();
{
    ulong before=session.LastCompletedRequestId;
    await CancelledAfterDispatch(session,token=>session.CalculateAsync(heavy.Capture,Enumerable.Range(1,256).Select(i=>P((uint)i,6)),token));
    Check(session.LastCompletedRequestId==before+1,"Cancelled dispatched response was drained");
}
frame=await session.ReadAsync(P(256,6),P(256,6));
Check(WorkbookSession.Decode(frame.Cells[0].Value)==355m,"Transport aligned after cancellation");

// Representative boundary timings, not UI or kernel-only transport measurements.
var metrics = new List<object>();
foreach(string operation in new[]{"edit","viewport","calculate"}) {
    var times=new List<double>();var rust=new List<double>();var encode=new List<double>();var residual=new List<double>();
    for(int i=0;i<34;i++) {
        clock.Restart();
        if(operation=="edit") await session.ApplyAsync(session.Revision,[N(3,1,i+1)]);
        else if(operation=="viewport") await session.ReadAsync(P(1,1),P(20,10));
        else await session.CalculateAsync(heavy.Capture,[P(256,6)]);
        if(i>=3) {
            times.Add(clock.Elapsed.TotalMilliseconds);rust.Add(session.LastRustExecutionNs/1e6);
            encode.Add(session.LastClientSerialization.TotalMilliseconds+session.LastClientDeserialization.TotalMilliseconds+session.LastRustSerializationNs/1e6);
            residual.Add(Math.Max(0,session.LastRoundTrip.TotalMilliseconds-session.LastRustExecutionNs/1e6-session.LastClientDeserialization.TotalMilliseconds-session.LastRustSerializationNs/1e6));
        }
    }
    times.Sort();rust.Sort();encode.Sort();residual.Sort();
    metrics.Add(new {operation,samples=31,median_ms=times[15],p95_ms=times[29],max_ms=times[30],rust_median_ms=rust[15],serialization_probe_median_ms=encode[15],framing_transport_scheduling_residual_median_ms=residual[15]});
}
await PersistenceChecks.Measure(session,args[0],args.Length>1 ? Path.GetDirectoryName(Path.GetFullPath(args[1]))! : Path.GetTempPath());
// Bounded larger diagnostic through the same production session, with caps unchanged.
for(int column=9;column<=10;column++) await session.ApplyAsync(session.Revision,Enumerable.Range(400,256).Select(row=>N((uint)row,(uint)column,row)));
await PersistenceChecks.Measure(session,args[0],args.Length>1 ? Path.GetDirectoryName(Path.GetFullPath(args[1]))! : Path.GetTempPath(),larger:true);
clock.Restart();await session.DisposeAsync();double shutdown=clock.Elapsed.TotalMilliseconds;
Check(session.FinalCharge==0,"Explicit close joins runtime and releases all retained captures");
await session.DisposeAsync();

// Broken child is terminal. No mutation is implicitly retried or replayed.
var broken=await WorkbookSession.StartAsync(args[0]);
using(var child=Process.GetProcessById(broken.ProcessId)) {child.Kill(entireProcessTree:true);await child.WaitForExitAsync();}
try {await broken.ReadAsync(P(1,1),P(1,1));throw new Exception("Expected transport failure");}
catch(IOException) { }
await broken.DisposeAsync();
var report=JsonSerializer.Serialize(new {status="passed",startup_ms=startup,shutdown_ms=shutdown,remaining_charge=session.FinalCharge,metrics},new JsonSerializerOptions {WriteIndented=true});
if(args.Length>1) await File.WriteAllTextAsync(args[1],report);
Console.WriteLine(report);
