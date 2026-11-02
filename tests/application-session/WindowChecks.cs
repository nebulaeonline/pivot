using Pivot.Client;
using Pivot.Protocol.Application.V1;
using System.Text.Json;
static class WindowChecks {
 static Position P(uint row,uint col=1)=>new(){Row=row,Column=col};
 static Edit E(uint row,string source)=>new(){At=P(row),Entry=source};
 static void Check(bool value,string message){if(!value)throw new Exception("Windows: "+message);}
 public static async Task Run(string binary,string output){
  var s=await WorkbookSession.StartAsync(binary);
  try{
   string sheet=s.WorksheetId;
   var created=await s.CreateWindowAsync(s.Revision,"Lines",100,10);string id=created.Worksheet;
   Check(created.WindowScope&&s.Windows.Single().Id==id&&s.Worksheets.Count==1,"independent directory");
   s.SelectWindow(id);await s.ApplyAsync(s.Revision,[E(1,"3"),E(2,"1"),E(3,"=ALLOCATEWEIGHTEDRANGE(10,1,2,A1:A2)"),E(4,"'001234")]);
   await s.FormatAsync(s.Revision,P(4),P(4),new CellStyle(Bold:true),32);
   Check(CellStyle.Decode((await s.ReadStylesAsync(P(4),P(4))).Styles.Single().Style).Bold,"Window formatting ownership");
   await s.SortAsync(s.Revision,P(1),P(2),false,[new SortKey{Column=1}]);
   Check(WorkbookSession.Decode((await s.ReadAsync(P(3),P(3))).Cells.Single().Value)==2.5m,"Window sorting changes spatial weights");
   await s.SortAsync(s.Revision,P(1),P(2),false,[new SortKey{Column=1,Descending=true}]);
   await s.SetFilterAsync(s.Revision,new FilterDefinition{First=P(1),Last=P(4),Conditions={new FilterCondition{Column=1,Predicates={new FilterPredicate{Operator=0,Decimal=WorkbookSession.Encode(3)}}}}});
   Response filtered=await s.ReadFilteredAsync(1,10,1,3);
   for(int i=0;filtered.FilterStatus==3&&i<200;i++){await Task.Delay(10);filtered=await s.ReadFilteredAsync(1,10,1,3);}
   Check(filtered.WindowScope&&filtered.FilterStatus==1&&filtered.FilterTotalVisibleRows==97&&filtered.FilterRows.Take(2).SequenceEqual(new uint[]{1,5}),"Window visibility uses declared extent and original rows");
   await s.SetFilterAsync(s.Revision,null);
   await s.StructureAsync(s.Revision,false,false,10,2);await s.StructureAsync(s.Revision,false,true,10,2);
   await s.CopyBetweenScopesAsync(s.Revision,P(1),P(4),sheet,false,P(10));
   var captured=await s.CaptureAsync();
   Task<Response> queued=s.ReadAsync(P(3),P(3));s.SelectWorksheet(sheet);var old=await queued;
   Check(old.WindowScope&&old.Worksheet==id&&WorkbookSession.Decode(old.Cells.Single().Value)==7.5m,"queued Window scope stays qualified");
   await s.ApplyAsync(s.Revision,[E(1,"=SUM(Lines|A1:A2)"),E(2,"=Lines|A1")]);
   var values=await s.ReadAsync(P(1),P(2));Check(WorkbookSession.Decode(values.Cells[0].Value)==4m,"canonical cross-scope calculation");
   await s.RenameWindowAsync(s.Revision,id,"O'Brien");
   Check((await s.ReadAsync(P(2),P(2))).Cells.Single().Formula=="='O''Brien'|A1","rename preserves binding");
   s.SelectWindow(id);
   string before=s.Revision;
   try{await s.CopyBlockAsync(before,P(1),P(1));throw new Exception("native Window copy accepted");}catch(WorkbookSessionException){}
   Check(s.Revision==before,"clipboard refusal does not mutate");
   var text=await s.CopyValuesAsync(before,P(4),P(4));Check(text.Clipboard.Length==0&&text.ClipboardText.Contains("'001234"),"explicit identifier values export");
   var preview=await s.PrepareWindowDeleteAsync(s.Revision,id);Check(preview.WindowInboundReferences==2,"exact inbound occurrences");
   await s.DeleteWindowAsync(s.Revision,id,preview.WindowDeleteTicket);
   var frozen=await s.CalculateAsync(captured.Capture,[P(3)]);Check(frozen.WindowScope&&WorkbookSession.Decode(frozen.Cells.Single().Value)==7.5m,"retired Window capture remains usable");
   await s.ReleaseAsync(captured.Capture);
   await s.CreateWindowAsync(s.Revision,"O'Brien",100,10);
   s.SelectWorksheet(sheet);
   Check((await s.ReadAsync(P(1),P(2))).Cells.All(c=>c.ErrorCode=="ReferenceDeleted"),"reused name never repairs");
   string path=Path.Combine(output,"windows-"+Guid.NewGuid()+".pivot");
   await s.SaveAsync(s.Revision,path);await s.OpenAsync(s.Revision,path);
   Check(s.Windows.Count==1&&(await s.ReadAsync(P(1),P(2))).Cells.All(c=>c.ErrorCode=="ReferenceDeleted"),"WB15 tombstones survive reopen");
  }finally{await s.DisposeAsync();}
  Check(s.FinalCharge==0,"all retained ownership released");
  await File.WriteAllTextAsync(Path.Combine(output,"windows.json"),JsonSerializer.Serialize(new{status="passed",remaining_charge=s.FinalCharge}));
 }
}
