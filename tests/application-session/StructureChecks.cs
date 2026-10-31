using Pivot.Client;
using Pivot.Protocol.Application.V1;
internal static class StructureChecks {
 static Position P(uint r,uint c=1)=>new(){Row=r,Column=c};
 static void Check(bool ok,string why){if(!ok)throw new Exception("Structure: "+why);}
 public static async Task Run(string binary,string output) {
  await using var s=await WorkbookSession.StartAsync(binary);
  var unchanged=s.Revision;var noop=await s.StructureAsync(unchanged,false,false,2,1);Check(!noop.StructureChanged&&s.Revision==unchanged,"empty semantic no-op");
  await s.ApplyAsync(s.Revision,[new Edit{At=P(2),Entry="7"},new Edit{At=P(5,2),Entry="=$A$2*2"},new Edit{At=P(5,3),Entry="=IF(FALSE,A2,1)"},new Edit{At=P(1,4),Text="001234"}]);
  var ids=await s.ReadAsync(P(2),P(2));string target=ids.Cells.Single().CellId;string formula=(await s.ReadAsync(P(5,2),P(5,2))).Cells.Single().CellId;
  await s.StructureAsync(s.Revision,false,false,2,2);Check((await s.ReadAsync(P(4),P(4))).Cells.Single().CellId==target,"surviving target identity");Check((await s.ReadAsync(P(7,2),P(7,2))).Cells.Single().Formula=="=$A$4*2","absolute reference follows target");
  await s.StructureAsync(s.Revision,false,true,4,1);
  var broken=(await s.ReadAsync(P(6,2),P(6,3))).Cells;Check(broken.Single(c=>c.At.Column==2).ErrorCode=="ReferenceDeleted"&&broken.Single(c=>c.At.Column==2).CellId==formula,"local fault and formula identity");Check(WorkbookSession.Decode(broken.Single(c=>c.At.Column==3).Result.Decimal)==1,"lazy branch unaffected");
  var block=await s.CopyBlockAsync(s.Revision,P(6,2),P(6,2));Check(block.Clipboard.Span[..8].SequenceEqual("PIVOTCB5"u8),"CB5 native header");
  await using var other=await WorkbookSession.StartAsync(binary);await other.PasteBlockAsync(other.Revision,P(1),P(1),block.Clipboard.ToByteArray(),true);Check((await other.ReadAsync(P(1),P(1))).Cells.Single().ErrorCode=="ReferenceDeleted","foreign fault survives without invented local targets");
  await s.PasteBlockAsync(s.Revision,P(10,2),P(10,2),block.Clipboard.ToByteArray(),true);await s.FillAsync(s.Revision,P(10,2),P(11,2),Request.Types.FillDirection.Down);Check((await s.ReadAsync(P(11,2),P(11,2))).Cells.Single().ErrorCode=="ReferenceDeleted","Fill carries fault snapshot");
  await s.ApplyAsync(s.Revision,[new Edit{At=P(4),Entry="100"}]);Check((await s.ReadAsync(P(6,2),P(6,2))).Cells.Single().ErrorCode=="ReferenceDeleted","new occupant does not heal");
  await s.ApplyAsync(s.Revision,[new Edit{At=P(6,2),Entry="=A4*2"}]);Check(WorkbookSession.Decode((await s.ReadAsync(P(6,2),P(6,2))).Cells.Single().Result.Decimal)==200,"normal formula repair");
  await s.FormatAsync(s.Revision,P(65536,20),P(65536,20),new CellStyle(Bold:true),32);var result=await s.StructureAsync(s.Revision,false,false,1,1);Check(result.FormattingClipped,"actual clipped presentation notice");
  string path=Path.Combine(output,"structure.pivot");await s.SaveAsync(s.Revision,path);Check((await File.ReadAllBytesAsync(path)).AsSpan(0,9).SequenceEqual("PIVOTWB12"u8),"WB11 native format");await s.NewAsync(s.Revision,discard:true);await s.OpenAsync(s.Revision,path);Check((await s.ReadAsync(P(12,2),P(12,2))).Cells.Single().ErrorCode=="ReferenceDeleted","persisted transferred fault");Check((await s.ReadAsync(P(2,4),P(2,4))).Cells.Single().Result.Text=="001234","String remains exact");
  await s.ApplyAsync(s.Revision,[new Edit{At=P(65536),Entry="1"}]);string before=s.Revision;try {await s.StructureAsync(before,false,false,1,1);throw new Exception("protected tail accepted");}catch(WorkbookSessionException){}Check(s.Revision==before,"tail refusal atomic");
 }
}
