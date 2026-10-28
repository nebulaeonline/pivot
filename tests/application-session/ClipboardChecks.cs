using System.Diagnostics;
using System.Text;
using System.Text.Json;
using Pivot.Client;
using Pivot.Protocol.Application.V1;

static class ClipboardChecks
{
 static Position P(uint r,uint c)=>new(){Row=r,Column=c};
 static Edit F(uint r,uint c,string source)=>new(){At=P(r,c),Formula=source};
 static void Check(bool ok,string detail){if(!ok)throw new Exception(detail);}
 static Cell At(Response frame,uint r,uint c)=>frame.Cells.Single(x=>x.At.Row==r&&x.At.Column==c);
 static async Task Reject(Func<Task> action){try{await action();throw new Exception("Expected clipboard rejection");}catch(WorkbookSessionException){}}
 public static async Task Run(string binary,string output)
 {
  var source=await WorkbookSession.StartAsync(binary);var target=await WorkbookSession.StartAsync(binary);
  var clock=Stopwatch.StartNew();
  try {
   try {await source.CopyBlockAsync("",P(1,1),P(1,1));throw new Exception("Expected explicit copy revision");}catch(ArgumentException){}
   try {await source.PasteBlockAsync("",P(1,1),P(1,1),[],false);throw new Exception("Expected explicit paste revision");}catch(ArgumentException){}
   await source.PasteBlockAsync(source.Revision,P(2,2),P(2,2),Encoding.UTF8.GetBytes("1.2300\t=B2+1\n-0.000\t=SUM($B$2:B3)"),false);
   string original=source.Revision;var copied=await source.CopyBlockAsync(original,P(2,2),P(3,3));
   Check(source.Revision==original,"Copy is read-only");
   await source.PasteBlockAsync(source.Revision,P(2,2),P(2,2),Encoding.UTF8.GetBytes("99"),false);
   await source.DisposeAsync();Check(source.FinalCharge==0,"Clipboard bytes retain no source session");
   var pasted=await target.PasteBlockAsync(target.Revision,P(7,6),P(7,6),copied.Clipboard.ToByteArray(),true);
   Check(pasted.PastedFirst.Equals(P(7,6))&&pasted.PastedLast.Equals(P(8,7)),"Destination extent");
   var frame=await target.ReadAsync(P(7,6),P(8,7));
   Check(At(frame,7,6).Number.Scale==4&&WorkbookSession.Decode(At(frame,7,6).Number)==1.2300m,"Exact decimal captured before source mutation/close");
   Check(At(frame,8,6).Number.Negative&&At(frame,8,6).Number.Scale==3,"Signed zero and scale");
   Check(At(frame,7,7).Formula=="=F7+1"&&WorkbookSession.Decode(At(frame,7,7).Value)==2.23m,"Native scalar translation");
   Check(At(frame,8,7).Formula=="=SUM($B$2:F8)","Native mixed absolute SUM");
   string before=target.Revision;
   await Reject(()=>target.PasteBlockAsync(before,P(10,10),P(11,12),copied.Clipboard.ToByteArray(),true));Check(target.Revision==before,"Shape rollback");
   await Reject(()=>target.PasteBlockAsync(before,P(10,10),P(10,10),new byte[]{1,2,3},true));Check(target.Revision==before,"Malformed native rollback");
   await Reject(()=>target.PasteBlockAsync(original,P(10,10),P(10,10),copied.Clipboard.ToByteArray(),true));Check(target.Revision==before,"Stale revision rollback");
   using(var cancel=new CancellationTokenSource()) {cancel.Cancel();try{await target.PasteBlockAsync(before,P(10,10),P(10,10),copied.Clipboard.ToByteArray(),true,cancel.Token);throw new Exception("Expected cancellation");}catch(OperationCanceledException){}Check(target.Revision==before,"Pre-dispatch cancellation");}
   await target.PasteBlockAsync(before,P(10,10),P(10,10),Encoding.UTF8.GetBytes("=B2+1"),false);
   Check(At(await target.ReadAsync(P(10,10),P(10,10)),10,10).Formula=="=B2+1","External formula remains literal");
   await target.ApplyAsync(target.Revision,[F(20,1,"=1+2")]);
   var rejected=(await target.CopyBlockAsync(target.Revision,P(20,1),P(20,1))).Clipboard.ToByteArray();
   rejected[24]=3; // Explicit bounded non-executable draft kind; no reference annotations.
   await target.PasteBlockAsync(target.Revision,P(20,2),P(20,2),rejected,true);
   Check(At(await target.ReadAsync(P(20,2),P(20,2)),20,2).Error.Length>0,"Copied rejected text cannot auto-promote despite valid syntax");
   string root=Path.Combine(Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetTempPath(),"pivot-clipboard-"+Guid.NewGuid());Directory.CreateDirectory(root);string path=Path.Combine(root,"clipboard.pivot");
   await target.SaveAsync(target.Revision,path);Check(Encoding.ASCII.GetString((await File.ReadAllBytesAsync(path))[..8])=="PIVOTWB8","WB5 format");
   await target.OpenAsync(target.Revision,path,discard:true);
   Check(At(await target.ReadAsync(P(20,2),P(20,2)),20,2).Error.Length>0,"WB3 preserves constrained rejected draft state");
   Check(At(await target.ReadAsync(P(8,7),P(8,7)),8,7).Formula=="=SUM($B$2:F8)","WB3 restores dollar-axis source and selection binding");
   await target.ApplyAsync(target.Revision,[F(20,2,"=1+2")]);Check(WorkbookSession.Decode(At(await target.ReadAsync(P(20,2),P(20,2)),20,2).Value)==3m,"Explicit resubmission executes rejected text");
  } finally {await source.DisposeAsync();await target.DisposeAsync();}
  Check(target.FinalCharge==0,"Clipboard destination zero-charge close");
  await File.WriteAllTextAsync(Path.Combine(output,"clipboard.json"),JsonSerializer.Serialize(new{status="passed",milliseconds=clock.Elapsed.TotalMilliseconds,source_remaining_charge=source.FinalCharge,destination_remaining_charge=target.FinalCharge},new JsonSerializerOptions{WriteIndented=true}));
 }
}
