using System.Diagnostics;
using System.Globalization;
using System.Text;
using System.Text.Json;
using Pivot.Client;
using Pivot.Protocol.Application.V1;

internal static class CapacityChecks
{
    static Position P(uint row, uint column=1) => new() { Row=row, Column=column };
    static void Check(bool b,string message) { if(!b) throw new Exception("Capacity: "+message); }
    public static async Task Run(string binary,string output)
    {
        var root=Path.Combine(Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetTempPath(),"capacity-"+Guid.NewGuid());
        Directory.CreateDirectory(root);Directory.CreateDirectory(output);
        var measurements=new List<object>();
        await using var s=await WorkbookSession.StartAsync(binary);
        Check(s.Worksheets.Single().Rows==WorkbookCapacity.MaximumRows && s.Worksheets.Single().Columns==WorkbookCapacity.MaximumColumns,"Rust/C# coordinate agreement");
        foreach(var shape in new[]{(Rows:2000,Columns:10),(Rows:4096,Columns:16)}) {
            var source=new StringBuilder();
            int cells=shape.Rows*shape.Columns;
            for(int i=0;i<cells;i++) source.Append((i+1).ToString(CultureInfo.InvariantCulture)).Append((i+1)%shape.Columns==0?'\n':',');
            var csv=Path.Combine(root,$"input-{cells}.csv");var exported=Path.Combine(root,$"output-{cells}.csv");var native=Path.Combine(root,$"workbook-{cells}.pivot");
            await File.WriteAllTextAsync(csv,source.ToString(),new UTF8Encoding(false));
            var clock=Stopwatch.StartNew();var prepared=await s.PrepareCsvImportAsync(csv);
            await s.ImportCsvAsync(s.Revision,prepared.CsvTicket,typed:true,discard:true);double importMs=clock.Elapsed.TotalMilliseconds;
            var first=(await s.ReadAsync(P(1),P(1))).Cells.Single();var last=P((uint)shape.Rows,(uint)shape.Columns);
            Check(WorkbookSession.Decode((await s.ReadAsync(last,last)).Cells.Single().Result.Decimal)==cells,"last imported numeric value");
            var captured=await s.CaptureAsync();
            Check(WorkbookSession.Decode((await s.CalculateAsync(captured.Capture,[P(1),last])).Cells.Last().Result.Decimal)==cells,"calculated table oracle");
            await s.ReleaseAsync(captured.Capture);
            clock.Restart();await s.ExportCsvAsync(s.Revision,exported,P(1),last);double exportMs=clock.Elapsed.TotalMilliseconds;
            Check((await File.ReadAllTextAsync(exported)).Replace("\r\n","\n")==source.ToString(),"complete independent row-major CSV oracle");
            var captures=new List<double>();var edits=new List<double>();var viewports=new List<double>();
            for(int i=0;i<31;i++) {
                clock.Restart();var capture=await s.CaptureAsync();captures.Add(clock.Elapsed.TotalMilliseconds);await s.ReleaseAsync(capture.Capture);
                clock.Restart();await s.ApplyAsync(s.Revision,[new Edit { At=P(1),Number=WorkbookSession.Encode(i+2) }]);edits.Add(clock.Elapsed.TotalMilliseconds);
                clock.Restart();await s.ReadAsync(P(1),P(20,10));viewports.Add(clock.Elapsed.TotalMilliseconds);
            }
            await s.ApplyAsync(s.Revision,[new Edit { At=P(1),Number=WorkbookSession.Encode(1) }]);
            Check((await s.ReadAsync(P(1),P(1))).Cells.Single().CellId==first.CellId,"replacements reuse identity");
            var formatTimes=new List<double>();
            for(int i=0;i<31;i++){clock.Restart();await s.FormatAsync(s.Revision,P(1),last,new CellStyle(Bold:i%2==0),32);formatTimes.Add(clock.Elapsed.TotalMilliseconds);}
            formatTimes.Sort();Check(formatTimes[29]<=100,"uniform numeric table format p95 objective");
            var formatSummary=await s.ReadStyleSummaryAsync(P(1),last);Check(formatSummary.MixedStyleFields==0&&CellStyle.Decode(formatSummary.SummaryStyle).Bold&&formatSummary.FormatRegions==1,"table format is one region with all values unchanged");
            var structureTimes=new List<double>();var structureRustTimes=new List<double>();
            foreach(var action in new[]{(false,false),(false,true),(true,false),(true,true)}) {
                clock.Restart();var result=await s.StructureAsync(s.Revision,action.Item1,action.Item2,2,1);structureTimes.Add(clock.Elapsed.TotalMilliseconds);
                structureRustTimes.Add(result.ExecutionNs/1_000_000.0);
                Check(result.StructureChanged&&result.StructureRetired==0,"insert/delete blank band moves content without replacing identities");
            }
            await File.WriteAllTextAsync(Path.Combine(output,$"structural-timings-{cells}.json"),JsonSerializer.Serialize(new{cells,shape.Rows,shape.Columns,order=new[]{"insert rows","delete rows","insert columns","delete columns"},wall_ms=structureTimes,rust_ms=structureRustTimes}));
            Check(structureTimes.All(t=>t<=1000),$"compact structural-command objective ({cells} cells; wall ms: {string.Join(", ",structureTimes)}; Rust ms: {string.Join(", ",structureRustTimes)})");
            clock.Restart();await s.SaveAsync(s.Revision,native);double saveMs=clock.Elapsed.TotalMilliseconds;
            clock.Restart();await s.OpenAsync(s.Revision,native);double reopenMs=clock.Elapsed.TotalMilliseconds;
            Check((await s.ReadAsync(P(1),P(1))).Cells.Single().CellId==first.CellId,"native reopen identity");
            await s.ExportCsvAsync(s.Revision,exported,P(1),last);
            Check((await File.ReadAllTextAsync(exported)).Replace("\r\n","\n")==source.ToString(),"complete save/reopen value oracle");
            captures.Sort();edits.Sort();viewports.Sort();
            var metrics=await s.ReadAsync(P(1),P(1));
            using var child=Process.GetProcessById(s.ProcessId);child.Refresh();
            long? workingSet=null,peakWorkingSet=null;
            try {workingSet=child.WorkingSet64;peakWorkingSet=child.PeakWorkingSet64;} catch(PlatformNotSupportedException) { }
            measurements.Add(new{cells,import_ms=importMs,export_ms=exportMs,save_ms=saveMs,reopen_ms=reopenMs,capture_p95_ms=captures[29],edit_p95_ms=edits[29],viewport_p95_ms=viewports[29],format_p95_ms=formatTimes[29],structure_ms=structureTimes,metrics.SelectionStats.DomainCharge,metrics.SelectionStats.DomainPeak,metrics.SelectionStats.AllocationCount,metrics.SelectionStats.AllocationBytes,working_set_bytes=workingSet,os_peak_working_set_bytes=peakWorkingSet});
            Check(s.PositionedCells==cells&&s.AllocatedCellIdentities==cells&&s.PositionedCellLimit==WorkbookCapacity.PositionedCells&&s.AllocatedCellIdentityLimit==WorkbookCapacity.AllocatedIdentities,"capacity diagnostics distinguish positioned and allocated identities");
            Check(importMs<=10000&&exportMs<=10000&&saveMs<=10000&&reopenMs<=10000,"table lifecycle latency objective");
            Check(edits[29]<=100&&viewports[29]<=100,"bounded interactive latency objective");
            Check(metrics.SelectionStats.DomainCharge<=256UL*1024*1024,"compact formatted-table steady charged-memory objective");
            // Client round-trip capture latency is reported separately from Rust-only capture time.
            Check(metrics.SelectionStats.DomainPeak<=512UL*1024*1024,"peak charged-memory objective");
        }
        await s.NewAsync(s.Revision,discard:true);
        var edge=P(65536);await s.ApplyAsync(s.Revision,[new Edit{At=edge,Entry="001234"}]);
        Check((await s.ReadAsync(edge,edge)).Cells.Single().Result.KindCase==TypedValue.KindOneofCase.Decimal,"existing entry numeric semantics unchanged");
        await s.ApplyAsync(s.Revision,[new Edit{At=edge,Text="001234"}]);
        var edgeCsv=Path.Combine(root,"edge.csv");await s.ExportCsvAsync(s.Revision,edgeCsv,edge,edge);
        Check(await File.ReadAllTextAsync(edgeCsv)=="\"001234\"\r\n","maximum-row typed export");
        var before=s.Revision;
        try {await s.ApplyAsync(before,[new Edit{At=P(65537),Entry="2"}]);throw new Exception("out-of-bounds row accepted");}catch(WorkbookSessionException){}
        Check(s.Revision==before,"row-boundary rejection atomic");
        await s.DisposeAsync();Check(s.FinalCharge==0,"zero remaining charge");
        await File.WriteAllTextAsync(Path.Combine(output,"capacity.json"),JsonSerializer.Serialize(new{status="passed",measurements,remaining_charge=s.FinalCharge}));
    }
}
