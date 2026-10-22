using System.Text;
using System.Text.Json;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
internal static class AggregateChecks {
 static Position P(uint r,uint c=1)=>new(){Row=r,Column=c};
 static void Check(bool b,string m){if(!b)throw new Exception("Aggregate boundary: "+m);}
 static Cell Cell(Response r,uint row,uint col)=>r.Cells.Single(c=>c.At.Equals(P(row,col)));
 public static async Task Run(string binary,string output){
  var s=await WorkbookSession.StartAsync(binary);
  try{
   await s.ApplyAsync(s.Revision,[new Edit{At=P(1),Number=WorkbookSession.Encode(1)},new Edit{At=P(2),Number=WorkbookSession.Encode(2)},new Edit{At=P(3),Number=WorkbookSession.Encode(2)},new Edit{At=P(4),Text="Label"},new Edit{At=P(1,2),Formula="=MIN(A1:A5)"},new Edit{At=P(2,2),Formula="=MAX(A1:A5)"},new Edit{At=P(3,2),Formula="=AVERAGE(A1:A5)"},new Edit{At=P(4,2),Formula="=COUNT(A1:A5)"},new Edit{At=P(5,2),Formula="=SUM(A1:A5)"}]);
   var r=await s.ReadAsync(P(1,2),P(5,2));Check(WorkbookSession.Decode(Cell(r,1,2).Value)==1&&WorkbookSession.Decode(Cell(r,2,2).Value)==2&&WorkbookSession.Decode(Cell(r,3,2).Value)==1.6666666666666666666666666667m&&WorkbookSession.Decode(Cell(r,4,2).Value)==3,"canonical numeric functions");Check(Cell(r,5,2).Error.Contains("Nonnumeric"),"SUM remains strict");
   var capture=await s.CaptureAsync();await s.ApplyAsync(s.Revision,[new Edit{At=P(5),Number=WorkbookSession.Encode(0)}]);
   r=await s.ReadAsync(P(3,2),P(4,2));Check(WorkbookSession.Decode(Cell(r,3,2).Value)==1.25m&&WorkbookSession.Decode(Cell(r,4,2).Value)==4,"zero classified as number");
   var old=await s.CalculateAsync(capture.Capture,[P(4,2)]);Check(WorkbookSession.Decode(old.Cells.Single().Value)==3,"old capture immutable");await s.ReleaseAsync(capture.Capture);
   await s.ApplyAsync(s.Revision,[new Edit{At=P(5),Clear=true},new Edit{At=P(1,3),Formula="=MIN(E1:E5)"}]);Check((await s.ReadAsync(P(1,3),P(1,3))).Cells.Single().Error.Contains("No numeric values"),"no numeric values explicit");
   string root=Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetTempPath();string path=Path.Combine(root,"pivot-aggregate-"+Guid.NewGuid()+".pivot");await s.SaveAsync(s.Revision,path);Check(Encoding.ASCII.GetString((await File.ReadAllBytesAsync(path))[..8])=="PIVOTWB5","new save WB5");
   await s.NewAsync(s.Revision,discard:true);await s.OpenAsync(s.Revision,path,discard:true);Check(!s.IsDirty&&WorkbookSession.Decode((await s.ReadAsync(P(4,2),P(4,2))).Cells.Single().Value)==3,"saved blank remains excluded");
   string fixture=Path.Combine(Directory.GetCurrentDirectory(),"crates","pivot-workbook-document","tests","fixtures","legacy-wb4-aggregates.hex");string legacy=Path.Combine(root,"pivot-legacy-"+Guid.NewGuid()+".pivot");await File.WriteAllBytesAsync(legacy,Convert.FromHexString((await File.ReadAllTextAsync(fixture)).Trim()));
   await s.OpenAsync(s.Revision,legacy,discard:true);Check(!s.IsDirty&&WorkbookSession.Decode((await s.ReadAsync(P(1,2),P(1,2))).Cells.Single().Value)==6,"real legacy WB4 checkpoint and SUM");
   await s.ApplyAsync(s.Revision,[new Edit{At=P(3,3),Formula="=COUNT(A1:A4)"}]);Check(WorkbookSession.Decode((await s.ReadAsync(P(3,3),P(3,3))).Cells.Single().Value)==2,"legacy blank reconstructed");await s.SaveAsync(s.Revision);Check(Encoding.ASCII.GetString((await File.ReadAllBytesAsync(legacy))[..8])=="PIVOTWB5","legacy next save WB5");
  }finally{await s.DisposeAsync();}
  Check(s.FinalCharge==0,"zero charge");await File.WriteAllTextAsync(Path.Combine(output,"aggregates.json"),JsonSerializer.Serialize(new{status="passed",remaining_charge=s.FinalCharge}));
 }
}
