using System.Text.Json;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
internal static class FillChecks {
 static Position P(uint r,uint c=1)=>new(){Row=r,Column=c};
 static void Check(bool b,string m){if(!b)throw new Exception("Fill boundary: "+m);}
 public static async Task Run(string binary,string output){
  var s=await WorkbookSession.StartAsync(binary);
  try{
   await s.ApplyAsync(s.Revision,[new Edit{At=P(1),Number=WorkbookSession.Encode(7)},new Edit{At=P(1,2),Formula="=A1+1"}]);
   await s.FillAsync(s.Revision,P(1),P(3,2),Request.Types.FillDirection.Down);var r=await s.ReadAsync(P(3),P(3,2));Check(WorkbookSession.Decode(r.Cells.Last().Value)==8,"real Rust downward formula translation");
   string invalidRevision=s.Revision;bool invalid=false;try{await s.FillAsync(s.Revision,P(1),P(3), (Request.Types.FillDirection)999);}catch(WorkbookSessionException){invalid=true;}Check(invalid&&s.Revision==invalidRevision,"unknown direction rejected without mutation");
   string revision=s.Revision;var id=r.Cells.Last().CellId;await s.FillAsync(s.Revision,P(1),P(3,2),Request.Types.FillDirection.Down);Check(s.Revision==revision&&(await s.ReadAsync(P(3,2),P(3,2))).Cells.Single().CellId==id,"repeat is no-op");
   foreach(var d in new[]{Request.Types.FillDirection.Up,Request.Types.FillDirection.Right,Request.Types.FillDirection.Left})await s.FillAsync(s.Revision,P(2),P(3,2),d);
   revision=s.Revision;bool failed=false;try{await s.FillAsync(revision,P(1),P(257),Request.Types.FillDirection.Down);}catch(WorkbookSessionException){failed=true;}Check(failed&&s.Revision==revision,"area rejection atomic");
   using(var c=new CancellationTokenSource()){c.Cancel();try{await s.FillAsync(revision,P(1),P(3),Request.Types.FillDirection.Down,c.Token);throw new Exception("Expected cancellation");}catch(OperationCanceledException){}}Check(s.Revision==revision,"pre-dispatch cancellation unchanged");
   await s.ApplyAsync(s.Revision,[new Edit{At=P(1),Number=WorkbookSession.Encode(42)}]);
   using(var c=new CancellationTokenSource()){var pending=s.FillAsync(s.Revision,P(1),P(3),Request.Types.FillDirection.Down,c.Token);c.Cancel();try{await pending;}catch(OperationCanceledException){}}
   Check(WorkbookSession.Decode((await s.ReadAsync(P(3),P(3))).Cells.Single().Value)==42,"dispatched cancelled Fill drains committed state");
   string root=Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetTempPath(),path=Path.Combine(root,"pivot-fill-"+Guid.NewGuid()+".pivot");await s.SaveAsync(s.Revision,path);Check(!s.IsDirty,"Fill saved");await s.NewAsync(s.Revision,discard:true);await s.OpenAsync(s.Revision,path);Check(WorkbookSession.Decode((await s.ReadAsync(P(3),P(3))).Cells.Single().Value)==42,"WB5 Fill restore");
  }finally{await s.DisposeAsync();}
  Check(s.FinalCharge==0,"zero charge");await File.WriteAllTextAsync(Path.Combine(output,"fill.json"),JsonSerializer.Serialize(new{status="passed",remaining_charge=s.FinalCharge}));
 }
}
