using System.Diagnostics;
using System.Text.Json;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
static class RangeChecks
{
 static Position P(uint r,uint c)=>new(){Row=r,Column=c};
 static Edit N(uint r,uint c,decimal v)=>new(){At=P(r,c),Number=WorkbookSession.Encode(v)};
 static Edit F(uint r,uint c,string v)=>new(){At=P(r,c),Formula=v};
 static void Check(bool yes,string message){if(!yes)throw new Exception(message);}
 static Cell At(Response response,uint r,uint c)=>response.Cells.Single(x=>x.At.Row==r&&x.At.Column==c);
 static async Task<decimal> Value(WorkbookSession session,uint r,uint c)=>WorkbookSession.Decode(At(await session.ReadAsync(P(r,c),P(r,c)),r,c).Value);
 public static async Task Run(string binary,string output)
 {
  string root=Path.Combine(Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetTempPath(),"pivot-sum-"+Guid.NewGuid());Directory.CreateDirectory(root);string file=Path.Combine(root,"sum.pivot");
  var s=await WorkbookSession.StartAsync(binary);
  await s.ApplyAsync(s.Revision,[N(1,1,10),N(2,1,20),N(3,1,30),F(1,4," =A1+sum(A3:A1) ")]);Check(await Value(s,1,4)==70,"SUM and scalar terms");
  var old=await s.CaptureAsync();await s.ApplyAsync(s.Revision,[new Edit{At=P(1,1),MoveTo=P(5,1)}]);Check(await Value(s,1,4)==60,"Scalar follows identity, range follows area");
  await s.ApplyAsync(s.Revision,[N(1,1,7)]);Check(await Value(s,1,4)==67,"Coordinate reuse enters range");
  var prior=await s.CalculateAsync(old.Capture,[P(1,4)]);Check(WorkbookSession.Decode(prior.Cells[0].Value)==70,"Captured range membership is immutable");await s.ReleaseAsync(old.Capture);
  await s.ApplyAsync(s.Revision,[new Edit{At=P(2,1),Clear=true},new Edit{At=P(1,4),MoveTo=P(2,4)}]);Check(await Value(s,2,4)==47,"Clear and formula movement retain bound rectangle");
  await s.ApplyAsync(s.Revision,[N(1,5,2),F(2,2,"=E1*3"),F(3,4,"=SUM(B2:B100)"),F(6,9,"=SUM(A10:H1024)")]);Check(await Value(s,3,4)==6,"Computed member");
  await s.ApplyAsync(s.Revision,[N(1,5,4)]);Check(await Value(s,3,4)==12,"Outside input invalidates computed member aggregate");
  await s.ApplyAsync(s.Revision,[F(2,2,"=D3+1")]);var cycle=await s.ReadAsync(P(3,4),P(3,4));Check(cycle.Cells[0].Error.Length>0,"Mixed scalar/selection cycle");Check(await Value(s,2,4)==47,"Unrelated cycle preserves selected local closure");
  await s.ApplyAsync(s.Revision,[F(2,2,"=E1*3"),F(7,9,"=SUM(A1)"),F(8,9,"=SUM(A1:Z99999)")]);Check((await s.ReadAsync(P(7,9),P(8,9))).Cells.All(c=>c.Error.Length>0),"Invalid range drafts");
  var empty=await s.ReadAsync(P(6,9),P(6,9));Check(WorkbookSession.Decode(empty.Cells[0].Value)==0&&empty.SelectionStats.Points<20,"Mostly-empty large rectangle has no per-blank nodes");
  await s.ApplyAsync(s.Revision,[N(1024,6,4)]);Check(await Value(s,6,9)==4,"Sparse member insertion");
  var fileClock=Stopwatch.StartNew();var prepared=await s.PrepareSaveAsync(s.Revision,file);double prepareMs=fileClock.Elapsed.TotalMilliseconds;await s.ApplyAsync(s.Revision,[N(3,1,5)]);fileClock.Restart();await s.CommitSaveAsync(prepared.Revision,prepared.SaveTicket);double commitMs=fileClock.Elapsed.TotalMilliseconds;Check(s.IsDirty,"Range Save S while editing T");
  var reader=await WorkbookSession.StartAsync(binary);fileClock.Restart();await reader.OpenAsync(reader.Revision,file,discard:true);double openMs=fileClock.Elapsed.TotalMilliseconds;Check(await Value(reader,2,4)==47&&await Value(s,2,4)==22,"WB3 restores saved selection generation, not current T");
  Check(At(await reader.ReadAsync(P(2,4),P(2,4)),2,4).Formula==" =A1+sum(A3:A1) ","Original range source survives move/reopen");
  await reader.ApplyAsync(reader.Revision,[N(2,1,9)]);Check(await Value(reader,2,4)==56,"Reopened coordinate-area membership updates without rebind");
  Check((await reader.ReadAsync(P(7,9),P(8,9))).Cells.All(c=>c.Error.Length>0),"Range drafts survive WB3");
  await reader.DisposeAsync();Check(reader.FinalCharge==0,"Reopened SUM cleanup");
  var observations=new List<object>();var latest=await s.ReadAsync(P(6,9),P(6,9));
  foreach(string operation in new[]{"bind","member_edit","member_add_remove","boundary_move","selected_sum","capture"}) {
   string? capture=null;if(operation=="selected_sum")capture=(await s.CaptureAsync()).Capture;
   for(int i=0;i<12;i++){
    var before=latest.SelectionStats;var clock=Stopwatch.StartNew();Response result;
    if(operation=="bind")result=await s.ApplyAsync(s.Revision,[F(6,9,"=SUM(A10:H1024)")]);
    else if(operation=="member_edit")result=await s.ApplyAsync(s.Revision,[N(1024,6,i+1)]);
    else if(operation=="member_add_remove") {await s.ApplyAsync(s.Revision,[N(500,6,i+1)]);result=await s.ApplyAsync(s.Revision,[new Edit{At=P(500,6),Clear=true}]);}
    else if(operation=="boundary_move"){await s.ApplyAsync(s.Revision,[new Edit{At=P(1024,6),MoveTo=P(1024,9)}]);result=await s.ApplyAsync(s.Revision,[new Edit{At=P(1024,9),MoveTo=P(1024,6)}]);}
    else if(operation=="selected_sum")result=await s.CalculateAsync(capture!,[P(6,9)]);
    else {var created=await s.CaptureAsync();result=await s.ReleaseAsync(created.Capture);}
    double ms=clock.Elapsed.TotalMilliseconds;var after=result.SelectionStats;
    if(i>0)observations.Add(new{operation,request_ms=ms,index_nodes=after.VisitedIndexNodes-before.VisitedIndexNodes,candidates=after.SelectionCandidates-before.SelectionCandidates,members=after.EmittedMembers-before.EmittedMembers,allocations=after.AllocationCount-before.AllocationCount,allocation_bytes=after.AllocationBytes-before.AllocationBytes,invalidated_nodes=after.InvalidatedNodes,domain_charge=after.DomainCharge,domain_peak=after.DomainPeak});latest=result;
   }
   if(capture is not null)await s.ReleaseAsync(capture);
  }
  await s.SaveAsync(s.Revision);await s.DisposeAsync();Check(s.FinalCharge==0,"SUM session cleanup");
  await File.WriteAllTextAsync(Path.Combine(output,"ranges.json"),JsonSerializer.Serialize(new{status="passed",workload="sparse large rectangle, ordinary production session; 11 observations after one warmup per operation",remaining_charge=s.FinalCharge,range_file_single_observation=new{prepare_ms=prepareMs,commit_ms=commitMs,open_ms=openMs,bytes=new FileInfo(file).Length},observations},new JsonSerializerOptions{WriteIndented=true}));
 }
}
