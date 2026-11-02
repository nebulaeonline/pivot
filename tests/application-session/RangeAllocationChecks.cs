using System.Text.Json;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
internal static class RangeAllocationChecks {
 static Position P(uint r,uint c=1)=>new(){Row=r,Column=c};
 static void Check(bool ok,string why){if(!ok)throw new Exception("Range allocation: "+why);}
 public static async Task Run(string binary,string output){
  await using var s=await WorkbookSession.StartAsync(binary);Directory.CreateDirectory(output);
  await s.ApplyAsync(s.Revision,[new Edit{At=P(1,2),Entry="3"},new Edit{At=P(2,2),Entry="1"},new Edit{At=P(1),Formula="=ALLOCATEWEIGHTEDRANGE(10,1,2,$B$1:$B$2)"},new Edit{At=P(2),Formula="=ALLOCATEWEIGHTEDRANGE(10,2,2,$B$1:$B$2)"},new Edit{At=P(3),Formula="=SUM(A1:A2)"},new Edit{At=P(4),Formula="=ALLOCATEWEIGHTEDRANGE(10,1,2,C1:C1)"},new Edit{At=P(5),Formula="=ALLOCATEWEIGHTEDRANGE(10,1,2,B1:C2)"}]);
  var frame=await s.ReadAsync(P(1),P(5));Cell At(uint r)=>frame.Cells.Single(c=>c.At.Row==r);
  Check(WorkbookSession.Decode(At(1).Result.Decimal)==7.5m&&WorkbookSession.Decode(At(3).Result.Decimal)==10m,"exact range values and SUM");
  Check(At(4).ErrorCode=="TypeMismatch"&&At(5).ErrorCode=="InvalidArgument","strict missing slot and geometry");
  var capture=await s.CaptureAsync();
  await s.SortAsync(s.Revision,P(1,2),P(2,2),false,[new SortKey{Column=2}],true);
  Check(WorkbookSession.Decode((await s.ReadAsync(P(1),P(1))).Cells.Single().Result.Decimal)==2.5m,"range order follows physical position after sort");
  Check(WorkbookSession.Decode((await s.CalculateAsync(capture.Capture,[P(1)])).Cells.Single().Result.Decimal)==7.5m,"immutable range capture");await s.ReleaseAsync(capture.Capture);
  var image=await s.CopyBlockAsync(s.Revision,P(1),P(1));await s.PasteBlockAsync(s.Revision,P(10),P(10),image.Clipboard.ToByteArray(),true);
  Check(WorkbookSession.Decode((await s.ReadAsync(P(10),P(10))).Cells.Single().Result.Decimal)==2.5m,"CB5 absolute range");
  string path=Path.Combine(Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetTempPath(),"range-"+Guid.NewGuid()+".pivot");await s.SaveAsync(s.Revision,path);await s.NewAsync(s.Revision,discard:true);await s.OpenAsync(s.Revision,path);
  Check(WorkbookSession.Decode((await s.ReadAsync(P(10),P(10))).Cells.Single().Result.Decimal)==2.5m,"WB14 range reopen");
  await s.DisposeAsync();Check(s.FinalCharge==0,"zero final charge");await File.WriteAllTextAsync(Path.Combine(output,"range-session.json"),JsonSerializer.Serialize(new{status="passed",remaining_charge=s.FinalCharge}));
 }
}
