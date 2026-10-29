using Pivot.Client;
using Pivot.Protocol.Application.V1;
internal static class FormattingChecks {
 static Position P(uint r,uint c=1)=>new(){Row=r,Column=c};
 static void Check(bool b,string why){if(!b)throw new Exception("Formatting: "+why);}
 public static async Task Run(string binary,string output){
  await using var s=await WorkbookSession.StartAsync(binary);
  await s.ApplyAsync(s.Revision,[new Edit{At=P(1),Entry="1234.5678"},new Edit{At=P(2),Entry="=A1*2"},new Edit{At=P(3),Entry="TRUE"},new Edit{At=P(4),Entry="123 dollars"}]);
  var capture=await s.CaptureAsync();
  var style=new CellStyle(Numeric:2,Grouping:true,Bold:true,Background:0xffeb9c);
  await s.FormatAsync(s.Revision,P(1),P(8),style,1023);
  var frame=await s.ReadAsync(P(1),P(8));
  Check(frame.Styles.Count==8&&frame.Cells.Count==4,"style-only blanks have no cell identity");
  var first=frame.Cells.Single(c=>c.At.Row==1);Check(first.FormattedNumber=="$1,234.57"&&WorkbookSession.Decode(first.Result.Decimal)==1234.5678m,"display separate from exact value");
  Check(frame.Cells.Single(c=>c.At.Row==2).FormattedNumber=="$2,469.14","downstream exact calculation formatted");
  Check(frame.Cells.Single(c=>c.At.Row==3).FormattedNumber==""&&frame.Cells.Single(c=>c.At.Row==4).Result.Text=="123 dollars","no numeric coercion");
  var before=s.Revision;await s.FormatAsync(before,P(1),P(8),new CellStyle(Bold:true),32);Check(before==s.Revision,"no-op patch");
  await s.FormatAsync(s.Revision,P(1),P(8),new CellStyle(Italic:true),64);Check(CellStyle.Decode((await s.ReadStylesAsync(P(1),P(1))).Styles.Single().Style).Bold,"untouched fields preserved");
  Check(WorkbookSession.Decode((await s.CalculateAsync(capture.Capture,[P(1)])).Cells.Single().Result.Decimal)==1234.5678m,"old calculation capture unchanged");await s.ReleaseAsync(capture.Capture);
  var copy=await s.CopyBlockAsync(s.Revision,P(5),P(8));await s.PasteBlockAsync(s.Revision,P(5,3),P(5,3),copy.Clipboard.ToByteArray(),true);
  Check((await s.ReadAsync(P(5,3),P(8,3))).Cells.Count==0&&CellStyle.Decode((await s.ReadStylesAsync(P(5,3),P(5,3))).Styles.Single().Style).Background==0xffeb9c,"styled native blank remains graph-free");
  await s.PasteBlockAsync(s.Revision,P(1),P(1),System.Text.Encoding.UTF8.GetBytes("5.123"),false);
  Check((await s.ReadAsync(P(1),P(1))).Cells.Single().FormattedNumber=="$5.12","external TSV keeps style");
  string path=Path.Combine(Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetTempPath(),"format-"+Guid.NewGuid()+".pivot");await s.SaveAsync(s.Revision,path);
  Check(!s.IsDirty,"save clean");await s.NewAsync(s.Revision,discard:true);await s.OpenAsync(s.Revision,path);
  Check(CellStyle.Decode((await s.ReadStylesAsync(P(8,3),P(8,3))).Styles.Single().Style).Background==0xffeb9c,"WB9 style-only reopen");
  await s.FormatAsync(s.Revision,P(1),P(1),new(),0,true);Check((await s.ReadAsync(P(1),P(1))).Cells.Single().FormattedNumber=="5.123","reset retains exact content");
 }
}
