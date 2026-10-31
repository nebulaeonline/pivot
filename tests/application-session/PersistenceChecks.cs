using System.Diagnostics;
using System.Text.Json;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
static class PersistenceChecks
{
 static Position P(uint r,uint c)=>new(){Row=r,Column=c};
 static Edit N(uint r,uint c,decimal n)=>new(){At=P(r,c),Number=WorkbookSession.Encode(n)};
 static Edit F(uint r,uint c,string s)=>new(){At=P(r,c),Formula=s};
 static void Check(bool b,string m){if(!b)throw new Exception(m);}
 static async Task Fail(Func<Task> a){try{await a();}catch(WorkbookSessionException){return;}throw new Exception("Expected document rejection");}
 public static async Task Run(string binary,string output)
 {
  string root=Path.Combine(Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetTempPath(),"pivot-workbook-"+Guid.NewGuid());Directory.CreateDirectory(root);
  string path=Path.Combine(root,"roundtrip.pivot");
  var s=await WorkbookSession.StartAsync(binary);
  Check(!s.IsDirty && s.SavedCheckpoint.Length==0 && s.DocumentPath.Length==0,"Untouched initial document is clean without claiming a saved file");
  await s.NewAsync(s.Revision);Check(!s.IsDirty,"Untouched New is clean");
  await s.CloseDocumentAsync(s.Revision);await s.NewAsync(s.Revision);
  await s.ApplyAsync(s.Revision,[new Edit{At=P(1,1),Clear=true}]);Check(!s.IsDirty,"No-op blank clear stays clean");
  await s.ApplyAsync(s.Revision,[N(1,1,1.2300m),F(1,2," =A1+2 "),F(1,3,"=("),N(3,1,new decimal(0,0,0,true,4)),N(3,2,decimal.MaxValue)]);
  await s.ApplyAsync(s.Revision,[new Edit{At=P(1,1),MoveTo=P(2,1)},N(1,1,100m)]);
  var frame=await s.ReadAsync(P(1,1),P(3,3));string stable=frame.Cells.Single(c=>c.At.Row==2&&c.At.Column==1).CellId;string doc=s.DocumentId;
  await Fail(()=>s.NewAsync(s.Revision));
  var clock=Stopwatch.StartNew();var prepared=await s.PrepareSaveAsync(s.Revision,path);double prepareMs=clock.Elapsed.TotalMilliseconds;
  await s.ApplyAsync(s.Revision,[N(2,1,8m)]);
  clock.Restart();var committed=await s.CommitSaveAsync(prepared.Revision,prepared.SaveTicket);double commitMs=clock.Elapsed.TotalMilliseconds;
  Check(s.IsDirty&&s.SavedCheckpoint.Length>0,"Save S must not mark T clean");
  var second=await WorkbookSession.StartAsync(binary);
  clock.Restart();var opened=await second.OpenAsync(second.Revision,path,discard:true);double openMs=clock.Elapsed.TotalMilliseconds;
  Check(!second.IsDirty&&second.DocumentId==doc,"Restored identity and saved checkpoint");
  var restored=await second.ReadAsync(P(1,1),P(3,3));
  Check(restored.Cells.Single(c=>c.At.Row==2&&c.At.Column==1).CellId==stable,"Cell identity survives independent process");
  Check(WorkbookSession.Decode(restored.Cells.Single(c=>c.At.Row==1&&c.At.Column==2).Value)==3.23m,"Moved historical binding must not rebind to reused A1");
  var restoredFormula=restored.Cells.Single(c=>c.At.Row==1&&c.At.Column==2);
  Check(restoredFormula.Formula==" =A2+2 "&&restoredFormula.AuthoredFormula==" =A1+2 ","Effective moved address and exact authored provenance");
  Check(restored.Cells.Single(c=>c.At.Row==1&&c.At.Column==3).Error.Length>0,"Invalid draft survives");
  Check(restored.Cells.Single(c=>c.At.Row==3&&c.At.Column==1).Number is {Negative:true,Scale:4},"Exact signed zero");
  Check(WorkbookSession.Decode(restored.Cells.Single(c=>c.At.Row==3&&c.At.Column==2).Number)==decimal.MaxValue,"96-bit coefficient");
  await second.ApplyAsync(second.Revision,[F(1,3,"=B1*2")]);
  Check(WorkbookSession.Decode((await second.ReadAsync(P(1,3),P(1,3))).Cells[0].Value)==6.46m,"Reopened draft repair");
  // External writer advances the file; both Save and Save As to the same association conflict.
  await second.SaveAsync(second.Revision);
  await Fail(()=>s.SaveAsync(s.Revision));await Fail(()=>s.SaveAsync(s.Revision,path));Check(s.IsDirty,"Conflict retains dirty state");
  var good=await File.ReadAllBytesAsync(path);
  string bad=Path.Combine(root,"bad.pivot");await File.WriteAllBytesAsync(bad,good[..^1]);
  string revision=s.Revision;await Fail(()=>s.OpenAsync(s.Revision,bad,discard:true));Check(s.Revision==revision&&s.DocumentId==doc,"Failed open keeps current document");
  await File.WriteAllBytesAsync(bad,new byte[4*1024*1024+100]);await Fail(()=>s.OpenAsync(s.Revision,bad,discard:true));
  await Fail(()=>s.SaveAsync(s.Revision,Path.Combine(root,"missing","x.pivot")));Check((await File.ReadAllBytesAsync(path)).SequenceEqual(good),"Failed Save As preserves valid destination");
  string other=Path.Combine(root,"other.pivot");await s.SaveAsync(s.Revision,other);Check(!s.IsDirty&&Path.GetFileName(s.DocumentPath)==Path.GetFileName(other),"Save As new association");
  await s.CloseDocumentAsync(s.Revision);Check(s.DocumentId.Length==0,"Close document");await s.OpenAsync(s.Revision,other);Check(!s.IsDirty,"Reopen in same child");
  using(var cancelled=new CancellationTokenSource()){cancelled.Cancel();try{await s.SaveAsync(s.Revision,cancellation:cancelled.Token);}catch(OperationCanceledException){}}
  await s.SaveAsync(s.Revision); // No abandoned ticket after pre-dispatch cancellation.
  await second.SaveAsync(second.Revision,other); // Save As explicitly replaces a separately observed existing destination.
  await s.OpenAsync(s.Revision,other);
  Check(WorkbookSession.Decode((await s.ReadAsync(P(1,3),P(1,3))).Cells[0].Value)==6.46m,"Save As existing destination round trip");
  await s.ApplyAsync(s.Revision,[N(4,1,7m)]);
  var latePrepared=await s.PrepareSaveAsync(s.Revision);
  using(var cancel=new CancellationTokenSource()) {
   var dispatched=s.CommitSaveAsync(s.Revision,latePrepared.SaveTicket,cancellation:cancel.Token);cancel.Cancel();
   try{await dispatched;}catch(OperationCanceledException){}
  }
  Check(!s.IsDirty,"Drained dispatched Save acknowledges the exact committed generation despite caller cancellation");
  await s.DisposeAsync();await second.DisposeAsync();Check(s.FinalCharge==0&&second.FinalCharge==0,"Document close releases charges");
  await File.WriteAllTextAsync(Path.Combine(output,"persistence.json"),JsonSerializer.Serialize(new{status="passed",prepare_ms=prepareMs,commit_ms=commitMs,open_ms=openMs,export_ms=prepared.ExportNs/1e6,capture_ms=prepared.CaptureNs/1e6,encode_ms=prepared.EncodeNs/1e6,decode_validate_ms=opened.DecodeNs/1e6,restore_ms=opened.RestoreNs/1e6,write_install_ms=committed.FileNs/1e6,read_ms=opened.FileNs/1e6,remaining_charge=s.FinalCharge,fixture_directory=root},new JsonSerializerOptions{WriteIndented=true}));
 }
 public static async Task Measure(WorkbookSession session,string binary,string output, bool larger = false)
 {
  string root=Path.Combine(Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetTempPath(),"pivot-measure-"+Guid.NewGuid());Directory.CreateDirectory(root);
  string path=Path.Combine(root,"production.pivot");var reader=await WorkbookSession.StartAsync(binary);
  var rows=new List<object>();
  for(int i=0;i<12;i++) {
   var clock=Stopwatch.StartNew();var prepared=await session.PrepareSaveAsync(session.Revision,i==0?path:null);
   var saved=await session.CommitSaveAsync(session.Revision,prepared.SaveTicket);double save=clock.Elapsed.TotalMilliseconds;
   clock.Restart();var opened=await reader.OpenAsync(reader.Revision,path,discard:true);double open=clock.Elapsed.TotalMilliseconds;
   if(i>0)rows.Add(new{save_ms=save,open_ms=open,capture_ms=prepared.CaptureNs/1e6,encode_ms=prepared.EncodeNs/1e6,write_install_ms=saved.FileNs/1e6,read_ms=opened.FileNs/1e6,decode_validate_ms=opened.DecodeNs/1e6,restore_ms=opened.RestoreNs/1e6});
  }
  var result=await reader.ReadAsync(P(256,6),P(256,6));Check(WorkbookSession.Decode(result.Cells.Single().Value)==355m,"Production fixture reopened calculation");
  await reader.DisposeAsync();Check(reader.FinalCharge==0,"Measured restoration closes cleanly");
  await File.WriteAllTextAsync(Path.Combine(output,larger ? "persistence-larger-diagnostic.json" : "persistence-production.json"),JsonSerializer.Serialize(new{status="passed",samples=11,warmups=1,bytes=new FileInfo(path).Length,workload=larger ? "bounded larger diagnostic: baseline plus 512 numeric cells, unchanged production limits" : "existing application fixture: approximately 282 live cells, 256-formula chain",remaining_charge=reader.FinalCharge,observations=rows},new JsonSerializerOptions{WriteIndented=true}));
 }

}
