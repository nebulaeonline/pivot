using System.Text.Json;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
internal static class WeightedChecks {
 static Position P(uint r,uint c=1)=>new(){Row=r,Column=c};
 static void Check(bool ok,string why){if(!ok)throw new Exception("Weighted: "+why);}
 public static async Task Run(string binary,string output){
  await using var s=await WorkbookSession.StartAsync(binary);Directory.CreateDirectory(output);
  string diagnostic=Path.Combine(output,"weighted-phases.log");
  async Task SaveCheck(string stage){await File.AppendAllTextAsync(diagnostic,stage+"\n");string name=Path.Combine(Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetTempPath(),"weighted-stage-"+Guid.NewGuid()+".pivot");await s.SaveAsync(s.Revision,name);}
  string[] source=["10","=ALLOCATEWEIGHTED($A$1,1,2,1,2,3)","=ALLOCATEWEIGHTED($A$1,2,2,1,2,3)","=ALLOCATEWEIGHTED($A$1,3,2,1,2,3)","=SUM(A2:A4)","=SUM(A4,A2,A3)","=ALLOCATEWEIGHTED(10.005,1,2,1,2,3)","=ALLOCATEWEIGHTED(10,1,A200,1,2,3)","=ALLOCATEWEIGHTED(79228162514264337593543950335,1,1,1,1,1,1,1,1,1,1,1,1)","=IF(FALSE,ALLOCATEWEIGHTED(10.005,1,2,0,1/0),7)","=ALLOCATEWEIGHTED(0.0000000000000000000000000001,64,28,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1)","=SUM(ALLOCATEWEIGHTED(10,1,28,1,1,1),ALLOCATEWEIGHTED(10,2,28,1,1,1),ALLOCATEWEIGHTED(10,3,28,1,1,1))"];
  await s.ApplyAsync(s.Revision,source.Select((v,i)=>new Edit{At=P((uint)i+1),Entry=v}));var frame=await s.ReadAsync(P(1),P((uint)source.Length));Cell At(uint r)=>frame.Cells.Single(c=>c.At.Row==r);
  Check(WorkbookSession.Decode(At(2).Result.Decimal)==1.67m&&WorkbookSession.Decode(At(3).Result.Decimal)==3.33m,"exact settled values");
  Check(WorkbookSession.Decode(At(5).Result.Decimal)==10m&&WorkbookSession.Decode(At(6).Result.Decimal)==10m&&WorkbookSession.Decode(At(12).Result.Decimal)==10m,"production SUM orders and normalizing final addition");
  Check(At(7).ErrorCode=="InvalidArgument"&&At(8).ErrorCode=="TypeMismatch"&&At(9).ErrorCode=="NumericError","invalid input versus group overflow");Check(WorkbookSession.Decode(At(10).Result.Decimal)==7m&&WorkbookSession.Decode(At(11).Result.Decimal)==0m,"lazy and largest index");
  await SaveCheck("initial");
  var captured=await s.CaptureAsync();await s.ApplyAsync(s.Revision,[new Edit{At=P(1),Entry="20"}]);Check(WorkbookSession.Decode((await s.CalculateAsync(captured.Capture,[P(5)])).Cells.Single().Result.Decimal)==10m,"old group capture stable");await s.ReleaseAsync(captured.Capture);
  var image=await s.CopyBlockAsync(s.Revision,P(2),P(2));await s.PasteBlockAsync(s.Revision,P(20,2),P(20,2),image.Clipboard.ToByteArray(),true);Check((await s.ReadAsync(P(20,2),P(20,2))).Cells.Single().Formula=="=ALLOCATEWEIGHTED($A$1,1,2,1,2,3)","CB5 explicit index not repaired");
  var visible=await s.CopyVisibleAsync(s.Revision,P(2),P(2));s.ConfirmVisiblePasteAsync=(q,r,c)=>Task.FromResult(true);await s.PasteVisibleAsync(s.Revision,P(22,2),P(22,2),visible.Clipboard.ToByteArray(),true);
  s.ConfirmVisibleEditAsync=(q,r,c)=>Task.FromResult(true);await s.FillVisibleAsync(s.Revision,P(20,2),P(22,2),Request.Types.FillDirection.Down);Check((await s.ReadAsync(P(22,2),P(22,2))).Cells.Single().Formula=="=ALLOCATEWEIGHTED($A$1,1,2,1,2,3)","visible Fill preserves explicit index");
  await s.FillAsync(s.Revision,P(20,2),P(21,2),Request.Types.FillDirection.Down);
  await s.FormatAsync(s.Revision,P(2),P(2),new CellStyle(Numeric:1,Places:0),3);Check(WorkbookSession.Decode((await s.ReadAsync(P(2),P(2))).Cells.Single().Result.Decimal)==3.33m,"format does not alter settlement values");
  await SaveCheck("copy-fill-format");
  await s.ApplyAsync(s.Revision,[new Edit{At=P(30),Number=WorkbookSession.Encode(2m)},new Edit{At=P(31),Number=WorkbookSession.Encode(1m)},new Edit{At=P(30,2),Formula="=ALLOCATEWEIGHTED(10,2,2,1,2,3)"},new Edit{At=P(31,2),Formula="=ALLOCATEWEIGHTED(10,1,2,1,2,3)"}]);
  await s.SortAsync(s.Revision,P(30),P(31,2),false,[new SortKey{Column=1}],true);Check((await s.ReadAsync(P(30,2),P(30,2))).Cells.Single().Formula=="=ALLOCATEWEIGHTED(10,1,2,1,2,3)","sorting moves explicit recipient without renumbering");
  await SaveCheck("sort");
  await s.ApplyAsync(s.Revision,[new Edit{At=P(40,2),Entry="3"},new Edit{At=P(41,2),Entry="1"},new Edit{At=P(40,3),Formula="=ALLOCATEWEIGHTED(10,1,2,$B$40,$B$41)"}]);
  await s.SortAsync(s.Revision,P(40,2),P(41,2),false,[new SortKey{Column=2}],true);
  var association=await s.ReadAsync(P(40,2),P(41,3));
  Check(WorkbookSession.Decode(association.Cells.Single(c=>c.At.Row==40&&c.At.Column==2).Result.Decimal)==1m,"weight cells sorted independently");
  var selected=association.Cells.Single(c=>c.At.Row==40&&c.At.Column==3);
  Check(WorkbookSession.Decode(selected.Result.Decimal)==7.5m&&selected.Formula=="=ALLOCATEWEIGHTED(10,1,2,$B$41,$B$40)","ordinal follows ordered bound arguments, not current worksheet row");

  await s.StructureAsync(s.Revision,false,false,1,1);Check((await s.ReadAsync(P(3),P(3))).Cells.Single().Formula=="=ALLOCATEWEIGHTED($A$2,1,2,1,2,3)","structure preserves binding and explicit index");
  string path=Path.Combine(Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetTempPath(),"weighted-"+Guid.NewGuid()+".pivot");await s.SaveAsync(s.Revision,path);await s.NewAsync(s.Revision,discard:true);await s.OpenAsync(s.Revision,path);Check(WorkbookSession.Decode((await s.ReadAsync(P(6),P(6))).Cells.Single().Result.Decimal)==20m,"WB14 allocation and SUM reopen");
  await s.SetFilterAsync(s.Revision,new FilterDefinition{First=P(1),Last=P(15,3),Conditions={new FilterCondition{Column=1,Predicates={new FilterPredicate{Operator=11}}}}});
  for(int i=0;i<200;i++){if((await s.ReadFilteredAsync(1,10,1,3)).FilterStatus!=3)break;await Task.Delay(10);}Check((await s.ReadFilteredAsync(1,10,1,3)).FilterStatus==1,"error filter accepts allocation faults");
  await s.DisposeAsync();Check(s.FinalCharge==0,"zero final charge");await File.WriteAllTextAsync(Path.Combine(output,"weighted-session.json"),JsonSerializer.Serialize(new{status="passed",production_sum=true,formats="WB14/CB5/CB6/private14",remaining_charge=s.FinalCharge}));
 }
}
