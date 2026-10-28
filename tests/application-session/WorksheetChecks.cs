using System.Text;
using System.Text.Json;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
internal static class WorksheetChecks {
 static Position P(uint r,uint c=1)=>new(){Row=r,Column=c};
 static Edit E(uint r,uint c,string text)=>new(){At=P(r,c),Entry=text};
 static void Check(bool condition,string text){if(!condition)throw new Exception("Worksheets: "+text);}
 static async Task<decimal> Value(WorkbookSession s,uint r,uint c=1)=>WorkbookSession.Decode((await s.ReadAsync(P(r,c),P(r,c))).Cells.Single().Value);
 public static async Task Run(string binary,string output){
  var s=await WorkbookSession.StartAsync(binary);
  try{
   string summary=s.WorksheetId;Check(s.Worksheets.Count==1&&s.Worksheets[0].Name=="Sheet1"&&!s.IsDirty,"clean initial sheet");
   string revision=s.Revision;s.SelectWorksheet(summary);Check(s.Revision==revision&&!s.IsDirty,"switch is view state");
   var added=await s.AddSheetAsync(s.Revision,"Revenue");string revenue=added.Worksheet;Check(revenue!=summary&&s.Worksheets.Count==2&&s.WorksheetId==summary&&s.IsDirty,"add stable sheet");
   s.SelectWorksheet(revenue);await s.ApplyAsync(s.Revision,[E(1,1,"10"),E(2,1,"20")]);
   s.SelectWorksheet(summary);await s.ApplyAsync(s.Revision,[E(1,1,"2"),E(1,2,"=SUM(Revenue!$A$1:A2,A1)"),E(1,3,"=Revenue!A1")]);
   Check(await Value(s,1,2)==32&&await Value(s,1,3)==10,"cross-sheet values");
   var capture=await s.CaptureAsync();
   await s.RenameSheetAsync(s.Revision,revenue,"O'Brien Sales");var frame=await s.ReadAsync(P(1,2),P(1,3));
   Check(frame.Worksheet==summary&&frame.Cells.All(c=>c.Worksheet==summary),"response sheet affinity");
   Check(frame.Cells[0].Formula=="=SUM('O''Brien Sales'!$A$1:A2,A1)"&&frame.Cells[0].AuthoredFormula=="=SUM(Revenue!$A$1:A2,A1)","effective quoted name and authored provenance");
   var copy=await s.CopyBlockAsync(s.Revision,P(1,2),P(1,2));Check(copy.ClipboardText.Contains("'O''Brien Sales'!"),"effective clipboard qualifier");
   await s.PasteBlockAsync(s.Revision,P(2,2),P(2,2),copy.Clipboard.ToByteArray(),true);Check(await Value(s,2,2)==30,"qualified paste/dollar axes");
   await s.FillAsync(s.Revision,P(1,2),P(3,2),Request.Types.FillDirection.Down);Check(await Value(s,3,2)==30,"qualified fill");
   // The queued read retains its sheet even if the caller switches before it drains.
   Task<Response> queued=s.ReadAsync(P(1),P(1));s.SelectWorksheet(revenue);var previous=await queued;Check(previous.Worksheet==summary&&WorkbookSession.Decode(previous.Cells.Single().Value)==2,"queued read not redirected");
   await s.ApplyAsync(s.Revision,[E(1,1,"100")]);var frozen=await s.CalculateAsync(capture.Capture,[P(1,2)]);Check(frozen.Worksheet==summary&&WorkbookSession.Decode(frozen.Cells.Single().Value)==32,"all-sheet immutable capture");await s.ReleaseAsync(capture.Capture);
   await s.ApplyAsync(s.Revision,[new Edit{At=P(1),MoveTo=P(3)},E(1,1,"999")]);
   s.SelectWorksheet(summary);Check(await Value(s,1,2)==1021&&await Value(s,1,3)==100,"range occupants and stable scalar binding");
   revision=s.Revision;try{await s.AddSheetAsync(revision,"o'brien sales");throw new Exception("duplicate accepted");}catch(WorkbookSessionException){}Check(s.Revision==revision,"duplicate atomic");
   string path=Path.Combine(Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetTempPath(),"worksheets-"+Guid.NewGuid()+".pivot");
   await s.SaveAsync(s.Revision,path);Check(!s.IsDirty&&Encoding.ASCII.GetString((await File.ReadAllBytesAsync(path))[..8])=="PIVOTWB8","whole workbook saved");
   ulong generation=s.DocumentGeneration;s.SelectWorksheet(revenue);await s.OpenAsync(s.Revision,path);Check(s.DocumentGeneration>generation&&s.WorksheetId==summary&&!s.IsDirty,"same-file reopen resets view context");
   Check(s.Worksheets.Count==2&&s.Worksheets[1].Id==revenue&&s.Worksheets[1].Name=="O'Brien Sales"&&await Value(s,1,3)==100,"directory and binding restored");
  }finally{await s.DisposeAsync();}
  Check(s.FinalCharge==0,"runtime fully released");await File.WriteAllTextAsync(Path.Combine(output,"worksheets.json"),JsonSerializer.Serialize(new{status="passed",remaining_charge=s.FinalCharge}));
 }
}
