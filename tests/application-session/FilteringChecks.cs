using System.Text;
using System.Text.Json;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
internal static class FilteringChecks {
 static Position P(uint row,uint column=1)=>new(){Row=row,Column=column};
 static void Check(bool yes,string message){if(!yes)throw new Exception("Filtering: "+message);}
 static FilterDefinition TextFilter()=>new(){First=P(1),Last=P(4,2),ExcludeHeader=true,Conditions={new FilterCondition{Column=1,Predicates={new FilterPredicate{Operator=0,Text="001234"}}}}};
 static async Task<Response> Frame(WorkbookSession s){for(int i=0;i<200;i++){var frame=await s.ReadFilteredAsync(1,10,1,3);if(frame.FilterStatus!=3)return frame;await Task.Delay(10);}throw new Exception("Filter refresh did not finish");}
 public static async Task Run(string binary,string output){
  await using var s=await WorkbookSession.StartAsync(binary);
  await s.ApplyAsync(s.Revision,[new Edit{At=P(1),Text="Code"},new Edit{At=P(2),Text="001234"},new Edit{At=P(3),Number=WorkbookSession.Encode(1234)},new Edit{At=P(4),Text="other"},new Edit{At=P(1,3),Formula="=SUM(A3:A3)"}]);
  var original=(await s.ReadAsync(P(2),P(2))).Cells.Single().CellId;string computational=s.ComputationalState;
  await s.SetFilterAsync(s.Revision,TextFilter());Check(s.ComputationalState==computational,"definition changes do not change computational state");
  var frame=await Frame(s);Check(frame.FilterStatus==1&&frame.FilterRows.Take(3).SequenceEqual(new uint[]{1,2,5}),"real worksheet rows, header and exact typed key");
  var invalid=TextFilter();invalid.Conditions[0].Predicates[0]=new FilterPredicate{Operator=0,DecimalText="0."+new string('1',29)};string unchanged=s.Revision;
  try{await s.ValidateFilterAsync(unchanged,invalid);throw new Exception("rounded over-precision criterion accepted");}catch(WorkbookSessionException){}Check(s.Revision==unchanged,"exact Decimal96 criterion validation never rounds or mutates definitions");
  Check(frame.Cells.All(c=>frame.FilterRows.Contains(c.At.Row)),"map, addresses, values coherent");
  Check(WorkbookSession.Decode((await s.ReadAsync(P(1,3),P(1,3))).Cells.Single().Result.Decimal)==1234,"aggregate still includes hidden rows");
  int prompts=0;s.ConfirmHiddenRowsAsync=(q,p,c)=>{prompts++;Check(p.FilterHiddenRows==2&&p.FilterFootprints.Single().Last.Row==4,"copy full source footprint");return Task.FromResult(false);};
  string before=s.Revision;try{await s.CopyBlockAsync(before,P(1),P(4));throw new Exception("cancelled copy ran");}catch(OperationCanceledException){}
  Check(prompts==1&&s.Revision==before,"one declined confirmation, no mutation");
  s.ConfirmHiddenRowsAsync=async(q,p,c)=>{await s.ReapplyFilterAsync();await Frame(s);return true;};
  before=s.Revision;try{await s.CopyBlockAsync(before,P(1),P(4));throw new Exception("stale filter generation accepted");}catch(WorkbookSessionException e){Check(e.Message.Contains("stale"),"generation-only change invalidates confirmation");}Check(s.Revision==before,"Reapply and stale confirmation do not dirty state");
  s.ConfirmHiddenRowsAsync=(q,p,c)=>{Check(p.FilterHiddenRows==2&&p.FilterFootprints.Single().Last.Row==65536&&p.FilterFootprints.Single().Last.Column==1024,"worksheet defaults cover hidden positions over the full sheet");return Task.FromResult(true);};
  computational=s.ComputationalState;await s.FormatRegionAsync(s.Revision,P(1),P(1),new CellStyle(Bold:true),32,defaults:true);Check(s.ComputationalState==computational&&s.FilterStatus==1,"formatting requalifies visibility without recalculation");
  s.ConfirmHiddenRowsAsync=(q,p,c)=>{Check(p.FilterHiddenRows==2&&p.FilterFootprints.Single().Last.Row==4,"CSV includes the complete addressed hidden rows");return Task.FromResult(true);};
  string csv=Path.Combine(output,"filtered-all-rows.csv");await s.ExportCsvAsync(s.Revision,csv,P(1),P(4,3));Check((await File.ReadAllTextAsync(csv)).Contains("001234"),"CSV preserves exact identifier text");
  s.ConfirmHiddenRowsAsync=async(q,p,c)=>{Check(p.FilterHiddenRows==1&&p.FilterFootprints.Single().First.Row==2&&p.FilterFootprints.Single().Last.Row==3,"paste actual payload-derived destination");await s.SetFilterAsync(s.Revision,null);return true;};
  try{await s.PasteBlockAsync(s.Revision,P(2),P(2),Encoding.UTF8.GetBytes("8\n9"),false);throw new Exception("stale confirmation ran");}catch(WorkbookSessionException e){Check(e.IsConflict,"stale ticket refuses exact state");}
  Check((await s.ReadAsync(P(2),P(2))).Cells.Single().CellId==original,"stale operation did not change destination");
  await s.SetFilterAsync(s.Revision,TextFilter());await Frame(s);
  s.ConfirmHiddenRowsAsync=(q,p,c)=>{Check(p.FilterFootprints.Count==2&&p.FilterHiddenRows==1,"Move accounts for source and destination");return Task.FromResult(true);};
  await s.ApplyAsync(s.Revision,[new Edit{At=P(2),MoveTo=P(3,2)}]);
  Check((await s.ReadAsync(P(3,2),P(3,2))).Cells.Single().CellId==original,"confirmed Move preserved its identity");
  frame=await Frame(s);Check(frame.FilterShownDataRows==0,"automatic refresh follows committed changed keys");
  s.ConfirmHiddenRowsAsync=(q,p,c)=>{Check(p.FilterFootprints.Single().Last.Row==65536,"structure confirmation covers shifted rows, not only the inserted band");return Task.FromResult(true);};
  await s.StructureAsync(s.Revision,false,false,1,1);frame=await Frame(s);Check(frame.FilterStatus==2&&frame.FilterSummary.NeedsReview,"structure persists NeedsReview");
  before=s.Revision;await s.ReapplyFilterAsync();Check(s.Revision==before&&s.FilterSummary!.NeedsReview,"Reapply does not repair or rebind NeedsReview");
  string path=Path.Combine(output,"filtered.pivot");await s.SaveAsync(s.Revision,path);Check((await File.ReadAllBytesAsync(path)).AsSpan(0,9).SequenceEqual("PIVOTWB12"u8),"WB12 current format");
  await s.NewAsync(s.Revision,discard:true);await s.OpenAsync(s.Revision,path);frame=await Frame(s);Check(frame.FilterStatus==2&&!s.IsDirty,"NeedsReview saved; operational state not fabricated");
  await s.SetFilterAsync(s.Revision,null);
  await s.NewAsync(s.Revision,discard:true);
  await s.ApplyAsync(s.Revision,(from row in Enumerable.Range(1,20) from column in Enumerable.Range(1,10) select new Edit{At=P((uint)row,(uint)column),Number=WorkbookSession.Encode(row)}).ToArray());
  var large=new FilterDefinition{First=P(1),Last=P(1,240)};for(uint column=1;column<=240;column++)large.Conditions.Add(new FilterCondition{Column=column,Predicates={new FilterPredicate{Operator=0,Text=new string('x',4096)}}});
  await s.SetFilterAsync(s.Revision,large);var full=await s.GetFilterAsync();Check(full.FilterDefinition.Conditions.Count==240,"full near-quota definition fetched independently");frame=await Frame(s);Check(frame.FilterDefinition is null&&frame.FilterSummary.Columns.Count==240&&frame.Cells.Count>0,"near-quota definition does not crowd out viewport values");
  string largePath=Path.Combine(output,"near-quota-filter.pivot");await s.SaveAsync(s.Revision,largePath);await s.NewAsync(s.Revision,discard:true);await s.OpenAsync(s.Revision,largePath);Check((await s.GetFilterAsync()).FilterDefinition.Conditions.Count==240,"near-quota WB12 definition reopens under stable admission");
  await s.NewAsync(s.Revision,discard:true);
  await s.ApplyAsync(s.Revision,[new Edit{At=P(2),Number=WorkbookSession.Encode(7)},new Edit{At=P(5,2),Formula="=$A$2*2"},new Edit{At=P(5,3),Formula="=IF(FALSE,A2,1)"}]);
  await s.ApplyAsync(s.Revision,[new Edit{At=P(8),Number=WorkbookSession.Encode(new decimal(0,0,0,true,4))}]);frame=await Frame(s);Check(frame.Cells.Single(c=>c.At.Row==8).Number is {Negative:true,Scale:4},"mapped viewport preserves exact submitted signed-zero parts");
  await s.ApplyAsync(s.Revision,[new Edit{At=P(9),Formula="=("},new Edit{At=P(1),Text="001234"},new Edit{At=P(1,2),Boolean=true},new Edit{At=P(1,3),Text=""}]);frame=await Frame(s);Check(frame.Cells.Single(c=>c.At.Row==9) is {Formula:"=(",Error.Length:>0},"invalid formula remains an editable cell diagnostic without failing other viewport cells");
  await s.StructureAsync(s.Revision,false,false,2,1);await s.StructureAsync(s.Revision,false,true,3,1);
  var ordinary=await s.ReadAsync(P(5,2),P(5,3));frame=await Frame(s);
  var fault=frame.Cells.Single(c=>c.At.Row==5&&c.At.Column==2);Check(fault.ErrorCode=="ReferenceDeleted"&&fault.Error.Length>0&&fault.Formula=="=#REF!*2","mapped viewport preserves deleted-reference source and diagnostic");
  var lazy=frame.Cells.Single(c=>c.At.Row==5&&c.At.Column==3);Check(lazy.Result.Equals(ordinary.Cells.Single(c=>c.At.Column==3).Result)&&WorkbookSession.Decode(lazy.Result.Decimal)==1,"lazy formula unaffected by deleted unused reference");
  var ordinaryFrame=await s.ReadAsync(P(1),P(10,3));Check(ordinaryFrame.Cells.Count==frame.Cells.Count&&frame.Cells.All(cell=>cell.Equals(ordinaryFrame.Cells.Single(other=>other.At.Equals(cell.At)))),"ordinary and mapped reads preserve identical cell payloads, exact inputs, authored sources and diagnostics");
  await s.SetFilterAsync(s.Revision,new FilterDefinition{First=P(1),Last=P(10,3),Conditions={new FilterCondition{Column=2,Predicates={new FilterPredicate{Operator=11}}}}});frame=await Frame(s);
  Check(frame.FilterRows[0]==5&&frame.Cells.Single(c=>c.At.Row==5&&c.At.Column==2).ErrorCode=="ReferenceDeleted","Is Error filter retains completed faults and usable viewport diagnostics");
  await s.DisposeAsync();Check(s.FinalCharge==0,"zero final charge and joined filter worker");
  await File.WriteAllTextAsync(Path.Combine(output,"filtering.json"),JsonSerializer.Serialize(new{status="passed",typed_exact_zeros=true,mapped_rows=true,exact_operation_confirmation=true,structural_review=true,persistence="WB12",remaining_charge=s.FinalCharge}));
 }
}
