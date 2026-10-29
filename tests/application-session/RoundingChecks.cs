using Pivot.Client;
using Pivot.Protocol.Application.V1;
internal static class RoundingChecks {
 static Position P(uint r,uint c=1)=>new(){Row=r,Column=c};
 static void Check(bool ok,string why){if(!ok)throw new Exception("Rounding: "+why);}
 public static async Task Run(string binary,string output){
  await using var s=await WorkbookSession.StartAsync(binary);
  string[] entries=["1.255","=ROUND(A1,2)","=ROUNDUP(-2.1,0)","=ROUNDDOWN(-125,-1)","=ROUND(79228162514264337593543950335,-1)","=ROUND(1,0.5)","=ROUND(TRUE,0)","=IF(FALSE,ROUNDUP(1,-100),7)"];
  await s.ApplyAsync(s.Revision,entries.Select((source,i)=>new Edit{At=P((uint)i+1),Entry=source}));
  var frame=await s.ReadAsync(P(1),P(8));Cell At(uint r)=>frame.Cells.Single(c=>c.At.Row==r);
  Check(WorkbookSession.Decode(At(2).Result.Decimal)==1.26m&&WorkbookSession.Decode(At(3).Result.Decimal)==-3m&&WorkbookSession.Decode(At(4).Result.Decimal)==-120m,"signs, ties and negative precision");
  Check(At(5).ErrorCode=="NumericError"&&At(6).ErrorCode=="InvalidArgument"&&At(7).ErrorCode=="TypeMismatch","typed failures");
  Check(WorkbookSession.Decode(At(8).Result.Decimal)==7m,"inactive overflowing branch not demanded");
  var capture=await s.CaptureAsync();await s.ApplyAsync(s.Revision,[new Edit{At=P(1),Entry="2.345"}]);
  Check(WorkbookSession.Decode((await s.CalculateAsync(capture.Capture,[P(2)])).Cells.Single().Result.Decimal)==1.26m,"old capture stable");await s.ReleaseAsync(capture.Capture);
  var copy=await s.CopyBlockAsync(s.Revision,P(2),P(2));await s.PasteBlockAsync(s.Revision,P(2,2),P(2,2),copy.Clipboard.ToByteArray(),true);Check((await s.ReadAsync(P(2,2),P(2,2))).Cells.Single().Formula=="=ROUND(B1,2)","native formula translation");
  await s.FormatAsync(s.Revision,P(2),P(2),new CellStyle(Numeric:1,Places:0),3);
  var cell=(await s.ReadAsync(P(2),P(2))).Cells.Single();Check(cell.FormattedNumber=="2"&&WorkbookSession.Decode(cell.Result.Decimal)==2.35m,"display rounding stays independent");
  string path=Path.Combine(Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetTempPath(),"rounding-"+Guid.NewGuid()+".pivot");await s.SaveAsync(s.Revision,path);await s.NewAsync(s.Revision,discard:true);await s.OpenAsync(s.Revision,path);
  Check(WorkbookSession.Decode((await s.ReadAsync(P(2),P(2))).Cells.Single().Result.Decimal)==2.35m,"WB9 rounding definitions reopen");
 }
}
