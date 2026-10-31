using System.Text.Json;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
internal static class SortingChecks {
 static Position P(uint row,uint col=1)=>new(){Row=row,Column=col};
 static void Check(bool ok,string why){if(!ok)throw new Exception("Sorting: "+why);}
 public static async Task Run(string binary,string output){
  await using var s=await WorkbookSession.StartAsync(binary);
  await s.ApplyAsync(s.Revision,[new Edit{At=P(1),Entry="2"},new Edit{At=P(2),Entry="1"},new Edit{At=P(1,2),Entry="=A1*10"},new Edit{At=P(2,2),Entry="=A2*10"},new Edit{At=P(1,4),Entry="=A1"},new Edit{At=P(2,4),Entry="=SUM(A1:A1)"},new Edit{At=P(1,5),Text="001234"}]);
  var before=await s.ReadAsync(P(1),P(2,5));var first=before.Cells.Single(c=>c.At.Equals(P(1))).CellId;
  var bounds=await s.SortBoundsAsync(s.Revision,P(1),P(2));Check(bounds.SortSuggestedLast.Column==2,"adjacent connected data suggestion stops before empty C");
  var old=s.Revision;var sorted=await s.SortAsync(old,P(1),P(2,2),false,[new SortKey{Column=1}]);Check(sorted.SortChanged&&sorted.SortMovedRows==2&&sorted.SortMovedCells==4,"single atomic row permutation");
  var frame=await s.ReadAsync(P(1),P(2,5));Check(frame.Cells.Single(c=>c.At.Equals(P(2))).CellId==first,"identity follows sorted record");
  Check(frame.Cells.Single(c=>c.At.Equals(P(1,4))).Formula=="=A2"&&WorkbookSession.Decode(frame.Cells.Single(c=>c.At.Equals(P(1,4))).Result.Decimal)==2,"external scalar follows target");
  Check(WorkbookSession.Decode(frame.Cells.Single(c=>c.At.Equals(P(2,4))).Result.Decimal)==1,"fixed range current occupant");
  Check(frame.Cells.Single(c=>c.At.Equals(P(1,5))).Result.Text=="001234","outside exact String untouched");
  var expected=s.Revision;var noop=await s.SortAsync(expected,P(1),P(2,2),false,[new SortKey{Column=1}]);Check(!noop.SortChanged&&s.Revision==expected,"identity-placement no-op");
  try{await s.SortAsync(old,P(1),P(2,2),false,[new SortKey{Column=1}]);throw new Exception("stale sort accepted");}catch(WorkbookSessionException e){Check(e.IsConflict,"stale sort refused");}
  try{await s.SortAsync(s.Revision,P(1),P(2,2),false,[new SortKey{Column=1},new SortKey{Column=1}]);throw new Exception("duplicate sort key accepted");}catch(WorkbookSessionException){}
  Check(s.Revision==expected,"refusal leaves revision unchanged");
  string path=Path.Combine(output,"sorted-current.pivot");await s.SaveAsync(s.Revision,path);await s.NewAsync(s.Revision,discard:true);await s.OpenAsync(s.Revision,path);
  Check((await s.ReadAsync(P(2),P(2))).Cells.Single().CellId==first,"WB11 preserves moved identity");
  await s.ApplyAsync(s.Revision,[new Edit{At=P(1,3),Entry="=1/0"},new Edit{At=P(2,3),Entry="=#REF!"}]);
  await s.SortAsync(s.Revision,P(1),P(2,3),false,[new SortKey{Column=3,Descending=true},new SortKey{Column=1,Descending=true}],false);
  Check((await s.ReadAsync(P(1,3),P(2,3))).Cells.All(c=>c.ErrorCode.Length>0),"completed cell errors sortable and retained");
  await s.DisposeAsync();Check(s.FinalCharge==0,"zero final domain charge");
  await File.WriteAllTextAsync(Path.Combine(output,"sorting.json"),JsonSerializer.Serialize(new{status="passed",identity=true,fixed_ranges=true,sort_bounds=true,errors=true,leading_zeros=true,persistence="WB11",remaining_charge=s.FinalCharge}));
 }
}
