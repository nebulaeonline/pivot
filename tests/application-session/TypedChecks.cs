using System.Text;
using System.Text.Json;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
internal static class TypedChecks {
 static Position P(uint r,uint c=1)=>new(){Row=r,Column=c};
 static void Check(bool ok,string why){if(!ok)throw new Exception("Typed values: "+why);}
 static async Task<Cell> Cell(WorkbookSession s,uint r,uint c=1)=>(await s.ReadAsync(P(r,c),P(r,c))).Cells.Single();
 public static async Task Run(string binary,string output){
  var s=await WorkbookSession.StartAsync(binary);
  try {
   string[] entries=["TRUE","hello","'","=IF(A1,A2&\"!\",1/0)","=LEN(\"😀a\")","=MID(\"😀abc\",2,79228162514264337593543950335)","=AND(FALSE,1/0)","=OR(TRUE,1/0)","=COUNT(\"42\",TRUE,7)","=1+TRUE","=MID(\"abc\",1.5,1)","=\"é\"=\"é\"","=\"He said \"\"yes\"\"\""];
   await s.ApplyAsync(s.Revision,entries.Select((text,i)=>new Edit{At=P((uint)i+1),Entry=text}));
   Check((await Cell(s,1)).Result.Boolean&&WorkbookSession.DisplayValue(await Cell(s,1))=="TRUE","Boolean literal and rendering");
   Check((await Cell(s,3)).Result.KindCase==TypedValue.KindOneofCase.Text&&(await Cell(s,3)).Result.Text=="","empty literal String");
   Check((await Cell(s,4)).Result.Text=="hello!"&&WorkbookSession.DisplayValue(await Cell(s,4))=="hello!","calculated String and lazy IF");
   Check(WorkbookSession.Decode((await Cell(s,5)).Result.Decimal)==2&&(await Cell(s,6)).Result.Text=="abc","Unicode and huge MID count");
   Check(!(await Cell(s,7)).Result.Boolean&&(await Cell(s,8)).Result.Boolean,"lazy logical calls");
   Check(WorkbookSession.Decode((await Cell(s,9)).Result.Decimal)==1,"typed aggregate filtering");
   Check((await Cell(s,10)).ErrorCode=="TypeMismatch"&&(await Cell(s,11)).ErrorCode=="InvalidArgument","typed errors");
   Check(!(await Cell(s,12)).Result.Boolean&&(await Cell(s,13)).Result.Text=="He said \"yes\"","ordinal comparison and quote escaping");
   var cap=await s.CaptureAsync();await s.ApplyAsync(s.Revision,[new Edit{At=P(1),Entry="'FALSE"}]);
   Check((await s.CalculateAsync(cap.Capture,[P(1)])).Cells.Single().Result.Boolean&&(await Cell(s,1)).Result.Text=="FALSE","captured type unchanged after edit");await s.ReleaseAsync(cap.Capture);
   var copy=await s.CopyBlockAsync(s.Revision,P(1),P(3));await s.PasteBlockAsync(s.Revision,P(1,3),P(1,3),copy.Clipboard.ToByteArray(),true);
   Check((await Cell(s,3,3)).Result.KindCase==TypedValue.KindOneofCase.Text,"typed clipboard preserves empty text");
   string before=s.Revision;byte[] obsolete=copy.Clipboard.ToByteArray();obsolete[7]=(byte)'2';
   try{await s.PasteBlockAsync(s.Revision,P(20),P(20),obsolete,true);throw new Exception("obsolete clipboard accepted");}catch(WorkbookSessionException){}Check(s.Revision==before,"obsolete native clipboard atomic refusal");
   string root=Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetTempPath();Directory.CreateDirectory(root);
   string path=Path.Combine(root,"typed-"+Guid.NewGuid()+".pivot");
   await s.SaveAsync(s.Revision,path);Check(Encoding.ASCII.GetString((await File.ReadAllBytesAsync(path))[..9])=="PIVOTWB11","WB8");
   byte[] old=await File.ReadAllBytesAsync(path);old[7]=(byte)'7';string oldPath=path+".wb7";await File.WriteAllBytesAsync(oldPath,old);before=s.Revision;
   try{await s.OpenAsync(before,oldPath);throw new Exception("WB7 accepted");}catch(WorkbookSessionException){}Check(s.Revision==before&&!s.IsDirty,"WB7 refusal preserves clean workbook");
   await s.OpenAsync(s.Revision,path);
   Check((await Cell(s,1)).Result.Text=="FALSE"&&(await Cell(s,3)).Result.KindCase==TypedValue.KindOneofCase.Text&&!s.IsDirty,"whole typed document restored");
  }finally{await s.DisposeAsync();}
  Check(s.FinalCharge==0,"zero runtime charge");await File.WriteAllTextAsync(Path.Combine(output,"typed-values.json"),JsonSerializer.Serialize(new{status="passed",remaining_charge=s.FinalCharge}));
 }
}
