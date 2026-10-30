using Pivot.Client;
using Pivot.Protocol.Application.V1;
internal static class RegionChecks {
 static Position P(uint r,uint c=1)=>new(){Row=r,Column=c};
 static void Check(bool ok,string why){if(!ok)throw new Exception("Region formatting: "+why);}
 public static async Task Run(string binary,string output) {
  await using var s=await WorkbookSession.StartAsync(binary);
  for(uint i=0;i<100;i++)await s.FormatAsync(s.Revision,P(1+i*10,3),P(1+i*10,3),new CellStyle(Numeric:1,Bold:true),33);
  var before=await s.ReadStyleSummaryAsync(P(1),P(1000,10));Check((before.MixedStyleFields&33)==33,"full-selection Mixed includes offscreen exceptions");
  await s.FormatAsync(s.Revision,P(1),P(1000,10),new CellStyle(Numeric:2),1);
  var summary=await s.ReadStyleSummaryAsync(P(1),P(1000,10));Check((summary.MixedStyleFields&1)==0&&CellStyle.Decode(summary.SummaryStyle).Numeric==2,"currency overwrites all exceptions");
  Check((summary.MixedStyleFields&32)!=0,"unmodified bold retained");
  await s.FormatAsync(s.Revision,P(1),P(1000,10),new CellStyle(Bold:false),32);
  summary=await s.ReadStyleSummaryAsync(P(1),P(1000,10));Check(summary.MixedStyleFields==0&&summary.FormatRegions==2,"no persistent cell priority");
  var revision=s.Revision;await s.FormatAsync(s.Revision,P(1),P(1000,10),new CellStyle(Bold:false),32);Check(s.Revision==revision,"same explicit assignment no-op");
  await s.FormatRegionAsync(s.Revision,P(1),P(1),new CellStyle(Bold:true),32,defaults:true);
  Check(!CellStyle.Decode((await s.ReadStylesAsync(P(1),P(1))).Styles.Single().Style).Bold,"explicit false independent of worksheet default");
  await s.FormatRegionAsync(s.Revision,P(1),P(1000,10),new(),32,inherit:true);
  Check(CellStyle.Decode((await s.ReadStylesAsync(P(1),P(1))).Styles.Single().Style).Bold,"inherit uses changed default");
  var copied=await s.CopyBlockAsync(s.Revision,P(1),P(1));await s.PasteBlockAsync(s.Revision,P(2000),P(2000),copied.Clipboard.ToByteArray(),true);
  await s.FormatRegionAsync(s.Revision,P(1),P(1),new CellStyle(Bold:false),32,defaults:true);
  Check(CellStyle.Decode((await s.ReadStylesAsync(P(2000),P(2000))).Styles.Single().Style).Bold,"clipboard is explicit snapshot, not source-default link");
  await s.FormatAsync(s.Revision,P(1),P(65536,1024),new(),0,true);
  summary=await s.ReadStyleSummaryAsync(P(1),P(65536,1024));Check(summary.FormatRegions==10&&summary.MixedStyleFields==0,"whole-sheet ten regions");Check(summary.PositionedCells==0&&summary.AllocatedCellIdentities==0,"entire empty worksheet remains graph-free");
  string path=Path.Combine(output,"regions.pivot");await s.SaveAsync(s.Revision,path);await s.NewAsync(s.Revision,discard:true);await s.OpenAsync(s.Revision,path);
  summary=await s.ReadStyleSummaryAsync(P(1),P(65536,1024));Check(summary.FormatRegions==10,"WB10 regions round-trip");
  string old=Path.Combine(output,"obsolete-version.pivot");await File.WriteAllBytesAsync(old,"PIVOTWB9"u8.ToArray());revision=s.Revision;
  try{await s.OpenAsync(s.Revision,old);throw new Exception("obsolete version accepted");}catch(WorkbookSessionException e){Check(e.Message.Contains("unsupported native workbook version WB9"),"explicit obsolete rejection");}
  Check(s.Revision==revision,"failed Open preserves current document");
 }
}
