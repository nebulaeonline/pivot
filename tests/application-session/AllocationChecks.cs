using System.Text.Json;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
internal static class AllocationChecks {
 static Position P(uint r,uint c=1)=>new(){Row=r,Column=c};
 static void Check(bool ok,string why){if(!ok)throw new Exception("Allocation: "+why);}
 public static async Task Run(string binary,string output){
  await using var s=await WorkbookSession.StartAsync(binary);Directory.CreateDirectory(output);
  string diagnostic=Path.Combine(output,"allocation-phases.log");
  async Task SaveCheck(string stage){await File.AppendAllTextAsync(diagnostic,stage+"\n");string name=Path.Combine(Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetTempPath(),"allocation-stage-"+Guid.NewGuid()+".pivot");await s.SaveAsync(s.Revision,name);}
  string[] source=["10","=ALLOCATE($A$1,3,1,2)","=ALLOCATE($A$1,3,2,2)","=ALLOCATE($A$1,3,3,2)","=SUM(A2:A4)","=SUM(A4,A2,A3)","=ALLOCATE(10.005,3,1,2)","=ALLOCATE(10,3,1,A200)","=ALLOCATE(79228162514264337593543950335,10,1,1)","=IF(FALSE,ALLOCATE(10.005,0,1/0,2),7)","=ALLOCATE(0.0000000000000000000000000001,4294967295,4294967295,28)","=SUM(ALLOCATE(10,3,1,28),ALLOCATE(10,3,2,28),ALLOCATE(10,3,3,28))"];
  await s.ApplyAsync(s.Revision,source.Select((v,i)=>new Edit{At=P((uint)i+1),Entry=v}));var frame=await s.ReadAsync(P(1),P((uint)source.Length));Cell At(uint r)=>frame.Cells.Single(c=>c.At.Row==r);
  Check(WorkbookSession.Decode(At(2).Result.Decimal)==3.34m&&WorkbookSession.Decode(At(3).Result.Decimal)==3.33m,"exact settled values");
  Check(WorkbookSession.Decode(At(5).Result.Decimal)==10m&&WorkbookSession.Decode(At(6).Result.Decimal)==10m&&WorkbookSession.Decode(At(12).Result.Decimal)==10m,"production SUM orders and normalizing final addition");
  Check(At(7).ErrorCode=="InvalidArgument"&&At(8).ErrorCode=="TypeMismatch"&&At(9).ErrorCode=="NumericError","invalid input versus group overflow");Check(WorkbookSession.Decode(At(10).Result.Decimal)==7m&&WorkbookSession.Decode(At(11).Result.Decimal)==0m,"lazy and largest index");
  await SaveCheck("initial");
  var captured=await s.CaptureAsync();await s.ApplyAsync(s.Revision,[new Edit{At=P(1),Entry="20"}]);Check(WorkbookSession.Decode((await s.CalculateAsync(captured.Capture,[P(5)])).Cells.Single().Result.Decimal)==10m,"old group capture stable");await s.ReleaseAsync(captured.Capture);
  var image=await s.CopyBlockAsync(s.Revision,P(2),P(2));await s.PasteBlockAsync(s.Revision,P(20,2),P(20,2),image.Clipboard.ToByteArray(),true);Check((await s.ReadAsync(P(20,2),P(20,2))).Cells.Single().Formula=="=ALLOCATE($A$1,3,1,2)","CB5 explicit index not repaired");
  var visible=await s.CopyVisibleAsync(s.Revision,P(2),P(2));s.ConfirmVisiblePasteAsync=(q,r,c)=>Task.FromResult(true);await s.PasteVisibleAsync(s.Revision,P(22,2),P(22,2),visible.Clipboard.ToByteArray(),true);
  s.ConfirmVisibleEditAsync=(q,r,c)=>Task.FromResult(true);await s.FillVisibleAsync(s.Revision,P(20,2),P(22,2),Request.Types.FillDirection.Down);Check((await s.ReadAsync(P(22,2),P(22,2))).Cells.Single().Formula=="=ALLOCATE($A$1,3,1,2)","visible Fill preserves explicit index");
  await s.FillAsync(s.Revision,P(20,2),P(21,2),Request.Types.FillDirection.Down);
  await s.FormatAsync(s.Revision,P(2),P(2),new CellStyle(Numeric:1,Places:0),3);Check(WorkbookSession.Decode((await s.ReadAsync(P(2),P(2))).Cells.Single().Result.Decimal)==6.67m,"format does not alter settlement values");
  await SaveCheck("copy-fill-format");
  await s.ApplyAsync(s.Revision,[new Edit{At=P(30),Number=WorkbookSession.Encode(2m)},new Edit{At=P(31),Number=WorkbookSession.Encode(1m)},new Edit{At=P(30,2),Formula="=ALLOCATE(10,3,2,2)"},new Edit{At=P(31,2),Formula="=ALLOCATE(10,3,1,2)"}]);
  await s.SortAsync(s.Revision,P(30),P(31,2),false,[new SortKey{Column=1}],true);Check((await s.ReadAsync(P(30,2),P(30,2))).Cells.Single().Formula=="=ALLOCATE(10,3,1,2)","sorting moves explicit recipient without renumbering");
  await SaveCheck("sort");
  await s.StructureAsync(s.Revision,false,false,1,1);Check((await s.ReadAsync(P(3),P(3))).Cells.Single().Formula=="=ALLOCATE($A$2,3,1,2)","structure preserves binding and explicit index");
  string path=Path.Combine(Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetTempPath(),"allocation-"+Guid.NewGuid()+".pivot");await s.SaveAsync(s.Revision,path);await s.NewAsync(s.Revision,discard:true);await s.OpenAsync(s.Revision,path);Check(WorkbookSession.Decode((await s.ReadAsync(P(6),P(6))).Cells.Single().Result.Decimal)==20m,"WB15 allocation and SUM reopen");
  await s.SetFilterAsync(s.Revision,new FilterDefinition{First=P(1),Last=P(15,3),Conditions={new FilterCondition{Column=1,Predicates={new FilterPredicate{Operator=11}}}}});
  for(int i=0;i<200;i++){if((await s.ReadFilteredAsync(1,10,1,3)).FilterStatus!=3)break;await Task.Delay(10);}Check((await s.ReadFilteredAsync(1,10,1,3)).FilterStatus==1,"error filter accepts allocation faults");
  await s.DisposeAsync();Check(s.FinalCharge==0,"zero final charge");await File.WriteAllTextAsync(Path.Combine(output,"allocation-session.json"),JsonSerializer.Serialize(new{status="passed",production_sum=true,formats="WB15/CB5/CB6/private14",remaining_charge=s.FinalCharge}));
 }
}
