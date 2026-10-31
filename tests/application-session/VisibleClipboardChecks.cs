using System.Text;
using System.Text.Json;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
internal static class VisibleClipboardChecks {
 static Position P(uint row,uint column=1)=>new(){Row=row,Column=column};
 static void Check(bool ok,string message){if(!ok)throw new Exception("Visible clipboard: "+message);}
 static async Task Ready(WorkbookSession s){for(int i=0;i<200;i++){if((await s.ReadFilteredAsync(1,10,1,5)).FilterStatus!=3)return;await Task.Delay(10);}throw new Exception("visibility timeout");}
 public static async Task Run(string binary,string output){
  Directory.CreateDirectory(output);await using var s=await WorkbookSession.StartAsync(binary);
  await s.ApplyAsync(s.Revision,[new Edit{At=P(2),Text="001234"},new Edit{At=P(3),Text="hidden source"},new Edit{At=P(5),Text="001234"},new Edit{At=P(10),Text="001234"},new Edit{At=P(12),Text="001234"},new Edit{At=P(5,2),Formula="=A2"},new Edit{At=P(11,4),Text="hidden sentinel"}]);
  await s.SetFilterAsync(s.Revision,new FilterDefinition{First=P(1),Last=P(30,5),Conditions={new FilterCondition{Column=1,Predicates={new FilterPredicate{Operator=0,Text="001234"}}}}});await Ready(s);
  var hidden=(await s.ReadAsync(P(11,4),P(11,4))).Cells.Single();var copied=await s.CopyVisibleAsync(s.Revision,P(2),P(5,2));Check(copied.ClipboardKind==6&&copied.ClipboardTextAvailable&&copied.VisibleRows.SequenceEqual(new uint[]{2,5}),"CB6 retains actual source rows and exact Strings");
  int confirmations=0;s.ConfirmVisiblePasteAsync=(q,r,c)=>{confirmations++;Check(r.VisibleRows.SequenceEqual(new uint[]{10,12})&&r.VisibleFirstColumn==4&&r.VisibleWidth==2&&r.VisibleSkippedRows==3,"actual row runs, columns and selected skipped edges");return Task.FromResult(true);};
  await s.PasteVisibleAsync(s.Revision,P(9,4),P(13,5),copied.Clipboard.ToByteArray(),true);await Ready(s);Check(confirmations==1,"one explicit destination confirmation");
  var frame=await s.ReadAsync(P(9,4),P(12,5));Check(frame.Cells.Single(c=>c.At.Row==12&&c.At.Column==5).Formula=="=D9","cross-record formula translation uses real origins");Check(frame.Cells.Single(c=>c.At.Row==11&&c.At.Column==4).Equals(hidden),"hidden direct target remains byte-identical");Check(frame.Cells.Single(c=>c.At.Row==9&&c.At.Column==4).ContentCase==Cell.ContentOneofCase.Blank,"normal hidden Blank anchor remains permitted");Check(frame.Cells.Single(c=>c.At.Row==10&&c.At.Column==4).Text=="001234","leading zeros survive mapped paste");
  s.ConfirmVisiblePasteAsync=(q,r,c)=>Task.FromResult(false);string revision=s.Revision;
  try{await s.PasteVisibleAsync(s.Revision,P(10,4),P(10,4),copied.Clipboard.ToByteArray(),true);throw new Exception("cancel accepted");}catch(OperationCanceledException){}Check(s.Revision==revision,"cancel leaves workbook unchanged");
  s.ConfirmVisiblePasteAsync=async(q,r,c)=>{await s.ReapplyFilterAsync();await Ready(s);return true;};
  try{await s.PasteVisibleAsync(s.Revision,P(10,4),P(10,4),copied.Clipboard.ToByteArray(),true);throw new Exception("stale generation accepted");}catch(WorkbookSessionException){}Check(s.Revision==revision,"generation-only change invalidates plan");
  s.ConfirmVisiblePasteAsync=(q,r,c)=>Task.FromResult(true);s.RevalidateFilteredOperationAsync=(q,c)=>Task.FromResult(false);
  try{await s.PasteVisibleAsync(s.Revision,P(10,4),P(10,4),copied.Clipboard.ToByteArray(),true);throw new Exception("changed clipboard accepted");}catch(OperationCanceledException){}Check(s.Revision==revision,"clipboard race cancels dispatch");s.RevalidateFilteredOperationAsync=null;
  await s.SetFilterAsync(s.Revision,null);await s.PasteBlockAsync(s.Revision,P(40,4),P(40,4),copied.Clipboard.ToByteArray(),true);Check((await s.ReadAsync(P(41,5),P(41,5))).Cells.Single().Formula=="=D38","ordinary paste packs CB6 while preserving each origin");
  var cb5=await s.CopyBlockAsync(s.Revision,P(2),P(3,2));s.ConfirmVisiblePasteAsync=(q,r,c)=>Task.FromResult(true);await s.PasteVisibleAsync(s.Revision,P(50,4),P(50,4),cb5.Clipboard.ToByteArray(),true);
  await s.PasteVisibleAsync(s.Revision,P(60,4),P(60,4),Encoding.UTF8.GetBytes("'001234\t=A2\n'\tTRUE"),false);var external=await s.ReadAsync(P(60,4),P(61,5));Check(external.Cells.Single(c=>c.At.Row==60&&c.At.Column==5).Formula=="=A2"&&external.Cells.Single(c=>c.At.Row==61&&c.At.Column==4).Text=="","external formulas literal; empty String distinct");
  await s.ApplyAsync(s.Revision,[new Edit{At=P(70),Text="001234\tline\nnext"}]);var nativeOnly=await s.CopyVisibleAsync(s.Revision,P(70),P(70));Check(!nativeOnly.ClipboardTextAvailable&&nativeOnly.ClipboardText=="","unfaithful TSV omitted without altering native String");await s.PasteVisibleAsync(s.Revision,P(71),P(71),nativeOnly.Clipboard.ToByteArray(),true);Check((await s.ReadAsync(P(71),P(71))).Cells.Single().Text=="001234\tline\nnext","native-only exact roundtrip");
  await using(var peer=await WorkbookSession.StartAsync(binary)){peer.ConfirmVisiblePasteAsync=(q,r,c)=>Task.FromResult(true);await peer.PasteVisibleAsync(peer.Revision,P(20,3),P(20,3),copied.Clipboard.ToByteArray(),true);Check((await peer.ReadAsync(P(21,4),P(21,4))).Cells.Single().Formula=="=C18","cross-process payload owns source origins without callbacks");await peer.DisposeAsync();Check(peer.FinalCharge==0,"peer zero charge");}
  await s.DisposeAsync();Check(s.FinalCharge==0,"zero charge with abandoned plan cleanup");await File.WriteAllTextAsync(Path.Combine(output,"visible-clipboard.json"),JsonSerializer.Serialize(new{status="passed",native="CB5+CB6",exact_rows=true,hidden_integrity=true,geometric_formulas=true,remaining_charge=s.FinalCharge}));
 }
}
