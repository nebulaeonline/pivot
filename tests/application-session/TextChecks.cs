using System.Text;
using System.Text.Json;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
static class TextChecks
{
 static Position P(uint r,uint c)=>new(){Row=r,Column=c};
 static Cell At(Response r,uint row,uint col)=>r.Cells.Single(c=>c.At.Row==row&&c.At.Column==col);
 static void Check(bool ok,string why){if(!ok)throw new Exception(why);}
 static Edit Entry(uint row,uint col,string source)=>new(){At=P(row,col),Entry=source};
 public static async Task Run(string binary,string output)
 {
  var s=await WorkbookSession.StartAsync(binary);
  try {
   await s.ApplyAsync(s.Revision,[Entry(1,1,"  Invoice 🧾 e\u0301  "),Entry(1,2,"'001.20"),Entry(1,3,"''Hello"),Entry(2,1,"2"),Entry(2,2,"=A2+1"),Entry(2,3,"=SUM(A2:A3)")]);
   var r=await s.ReadAsync(P(1,1),P(3,3));
   Check(At(r,1,1).Text=="  Invoice 🧾 e\u0301  "&&At(r,1,1).Value is null&&At(r,1,1).Error=="","Text is valid content without numeric output");
   string stable=At(r,2,1).CellId;var capture=await s.CaptureAsync();
   await s.ApplyAsync(s.Revision,[Entry(2,1,"Label")]);r=await s.ReadAsync(P(2,1),P(2,3));
   Check(At(r,2,1).CellId==stable&&At(r,2,2).ErrorCode=="TypeMismatch"&&At(r,2,3).ErrorCode=="TypeMismatch","Stable identity and explicit type errors");
   Check(WorkbookSession.Decode(At(await s.CalculateAsync(capture.Capture,[P(2,2)]),2,2).Value)==3m,"Old numeric capture survives text replacement");await s.ReleaseAsync(capture.Capture);
   var block=await s.CopyBlockAsync(s.Revision,P(1,1),P(1,3));
   await s.PasteBlockAsync(s.Revision,P(4,1),P(4,1),block.Clipboard.ToByteArray(),true);
   await s.PasteBlockAsync(s.Revision,P(5,1),P(5,1),Encoding.UTF8.GetBytes(block.ClipboardText),false);
   r=await s.ReadAsync(P(4,1),P(5,3));foreach(uint row in new uint[]{4,5}){Check(At(r,row,2).Text=="001.20"&&At(r,row,3).Text=="'Hello","Escaped text native/TSV roundtrip");}
   string root=Path.Combine(Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetTempPath(),"pivot-text-"+Guid.NewGuid());Directory.CreateDirectory(root);string file=Path.Combine(root,"text.pivot");
   await s.SaveAsync(s.Revision,file);Check(!s.IsDirty,"Saved text clean checkpoint");
   var bytes=await File.ReadAllBytesAsync(file);Check(Encoding.ASCII.GetString(bytes[..8])=="PIVOTWB8","WB5 header");
   await s.ApplyAsync(s.Revision,[Entry(9,1,"Unsaved content")]);string revision=s.Revision,document=s.DocumentId;
   foreach(byte version in new byte[]{(byte)'1',(byte)'2',(byte)'3'}) {
    var old=(byte[])bytes.Clone();old[7]=version;string path=Path.Combine(root,"old-"+(char)version+".pivot");await File.WriteAllBytesAsync(path,old);
    try{await s.OpenAsync(s.Revision,path,discard:true);throw new Exception("Old workbook accepted");}catch(WorkbookSessionException){}
    Check(s.Revision==revision&&s.IsDirty&&s.DocumentId==document,"Failed Open preserves dirty live document");
   }
   await s.OpenAsync(s.Revision,file,discard:true);r=await s.ReadAsync(P(1,1),P(2,3));Check(At(r,1,1).Text=="  Invoice 🧾 e\u0301  "&&At(r,2,3).ErrorCode=="TypeMismatch"&&!s.IsDirty,"WB4 text and marker restore");
   await s.ApplyAsync(s.Revision,[Entry(2,1,"5")]);Check(WorkbookSession.Decode(At(await s.ReadAsync(P(2,2),P(2,3)),2,2).Value)==6m,"Restored marker becomes numeric normally");
  }finally{await s.DisposeAsync();}
  Check(s.FinalCharge==0,"Text session zero-charge close");await File.WriteAllTextAsync(Path.Combine(output,"text.json"),JsonSerializer.Serialize(new{status="passed",remaining_charge=s.FinalCharge}));
 }
}
