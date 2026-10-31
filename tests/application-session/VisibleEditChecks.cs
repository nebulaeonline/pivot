using System.Text.Json;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
internal static class VisibleEditChecks {
 static Position P(uint r,uint c=1)=>new(){Row=r,Column=c};
 static void Check(bool ok,string message){if(!ok)throw new Exception("Visible edit: "+message);}
 static async Task Ready(WorkbookSession s){for(int i=0;i<300;i++){if((await s.ReadFilteredAsync(1,12,1,5)).FilterStatus!=3)return;await Task.Delay(10);}throw new Exception("visible edit map timeout");}
 public static async Task Run(string binary,string output){
  Directory.CreateDirectory(output);await using var s=await WorkbookSession.StartAsync(binary);
  await s.ApplyAsync(s.Revision,[new Edit{At=P(2),Text="shown"},new Edit{At=P(5),Text="shown"},new Edit{At=P(12),Text="shown"},new Edit{At=P(2,2),Text="001234"},new Edit{At=P(5,2),Formula="=A2"},new Edit{At=P(8,2),Text="hidden"},new Edit{At=P(12,2),Text="old"}]);
  await s.SetFilterAsync(s.Revision,new FilterDefinition{First=P(1),Last=P(20,5),Conditions={new FilterCondition{Column=1,Predicates={new FilterPredicate{Operator=0,Text="shown"}}}}});await Ready(s);
  var observation=System.Diagnostics.Stopwatch.StartNew();double fillPreviewMs=0,clearPreviewMs=0,clearRoundtripMs=0;
  var hidden=(await s.ReadAsync(P(8,2),P(8,2))).Cells.Single();int confirmations=0;
  s.ConfirmVisibleEditAsync=(q,r,c)=>{confirmations++;fillPreviewMs=observation.Elapsed.TotalMilliseconds;Check(r.VisibleRows.SequenceEqual(new uint[]{5,12})&&r.VisibleWidth==1&&r.VisibleSkippedRows==6,"exact rows, columns, skipped edges");return Task.FromResult(true);};
  observation.Restart();await s.FillVisibleAsync(s.Revision,P(5,2),P(12,2),Request.Types.FillDirection.Down);await Ready(s);
  Check((await s.ReadAsync(P(12,2),P(12,2))).Cells.Single().Formula=="=A9","actual source offset");Check((await s.ReadAsync(P(8,2),P(8,2))).Cells.Single().Equals(hidden),"hidden direct target unchanged");
  using(var cancel=new CancellationTokenSource()){
   s.RequestDispatchedForTesting=()=>{s.RequestDispatchedForTesting=null;cancel.Cancel();};
   try{await s.ClearVisibleAsync(s.Revision,P(5,2),P(12,2),Request.Types.ClearMode.Both,cancel.Token);throw new Exception("cancelled preview accepted");}catch(OperationCanceledException){}finally{s.RequestDispatchedForTesting=null;}
  }
  string revision=s.Revision;s.ConfirmVisibleEditAsync=(q,r,c)=>Task.FromResult(false);
  try{await s.ClearVisibleAsync(s.Revision,P(5,2),P(12,2),Request.Types.ClearMode.Both);throw new Exception("cancel accepted");}catch(OperationCanceledException){}Check(s.Revision==revision,"cancel preserves revision");
  s.ConfirmVisibleEditAsync=async(q,r,c)=>{await s.ReapplyFilterAsync();await Ready(s);return true;};
  try{await s.ClearVisibleAsync(s.Revision,P(5,2),P(12,2),Request.Types.ClearMode.Both);throw new Exception("stale map accepted");}catch(WorkbookSessionException){}Check(s.Revision==revision,"stale map refused");
  string interaction="before";s.CaptureFilterInteractionAsync=c=>Task.FromResult(interaction);s.ConfirmVisibleEditAsync=(q,r,c)=>{interaction="after";return Task.FromResult(true);};
  try{await s.ClearVisibleAsync(s.Revision,P(5,2),P(12,2),Request.Types.ClearMode.Contents);throw new Exception("stale interaction accepted");}catch(OperationCanceledException){}Check(s.Revision==revision,"stale interaction refused");s.CaptureFilterInteractionAsync=null;
  s.ConfirmVisibleEditAsync=(q,r,c)=>Task.FromResult(true);await s.ClearVisibleAsync(s.Revision,P(5,2),P(12,2),Request.Types.ClearMode.Both);await Ready(s);var frame=await s.ReadAsync(P(5,2),P(12,2));Check(frame.Cells.Where(c=>c.At.Row is 5 or 12).All(c=>c.ContentCase==Cell.ContentOneofCase.Blank),"clear contents");Check(frame.Cells.Single(c=>c.At.Row==8).Equals(hidden),"Clear Both hidden intact");
  await s.SetFilterAsync(s.Revision,null);revision=s.Revision;await s.FillVisibleAsync(s.Revision,P(1),P(1,2),Request.Types.FillDirection.Down);Check(s.Revision==revision,"degenerate reviewed no-op");
  s.ConfirmVisibleEditAsync=(q,r,c)=>{clearPreviewMs=observation.Elapsed.TotalMilliseconds;return Task.FromResult(true);};
  observation.Restart();await s.ClearVisibleAsync(s.Revision,P(1,3),P(4096,3),Request.Types.ClearMode.Formatting);clearRoundtripMs=observation.Elapsed.TotalMilliseconds;
  revision=s.Revision;try{await s.ClearVisibleAsync(s.Revision,P(1,3),P(4097,3),Request.Types.ClearMode.Contents);throw new Exception("oversize accepted");}catch(WorkbookSessionException e){Check(e.Message.Contains("4096"),"actionable limit");}Check(s.Revision==revision,"oversize refused before mutation");
  // Membership changes caused by this transaction cannot shorten its fixed write set.
  await s.NewAsync(s.Revision,discard:true);
  await s.ApplyAsync(s.Revision,[new Edit{At=P(2),Number=WorkbookSession.Encode(1m)},new Edit{At=P(5),Number=WorkbookSession.Encode(0m)},new Edit{At=P(12),Number=WorkbookSession.Encode(0m)},new Edit{At=P(2,2),Formula="=A2"},new Edit{At=P(5,2),Number=WorkbookSession.Encode(1m)},new Edit{At=P(12,2),Number=WorkbookSession.Encode(1m)}]);
  await s.SetFilterAsync(s.Revision,new FilterDefinition{First=P(1),Last=P(20,5),Conditions={new FilterCondition{Column=2,Predicates={new FilterPredicate{Operator=4,Decimal=WorkbookSession.Encode(0m)}}}}});await Ready(s);
  s.ConfirmVisibleEditAsync=(q,r,c)=>{Check(r.VisibleRows.SequenceEqual(new uint[]{2,5,12}),"pre-operation membership fixed");return Task.FromResult(true);};
  await s.FillVisibleAsync(s.Revision,P(2,2),P(12,2),Request.Types.FillDirection.Down);await Ready(s);
  var changed=(await s.ReadAsync(P(2,2),P(12,2))).Cells;Check(changed.Single(c=>c.At.Row==5).Formula=="=A5"&&changed.Single(c=>c.At.Row==12).Formula=="=A12","all original destinations written before filter hides them");
  await s.SetFilterAsync(s.Revision,null);await s.ApplyAsync(s.Revision,[new Edit{At=P(5),Number=WorkbookSession.Encode(1m)},new Edit{At=P(12),Number=WorkbookSession.Encode(1m)}]);
  await s.SetFilterAsync(s.Revision,new FilterDefinition{First=P(1),Last=P(20,5),Conditions={new FilterCondition{Column=2,Predicates={new FilterPredicate{Operator=4,Decimal=WorkbookSession.Encode(0m)}}}}});await Ready(s);
  await s.ClearVisibleAsync(s.Revision,P(2,2),P(12,2),Request.Types.ClearMode.Both);await Ready(s);Check((await s.ReadAsync(P(2,2),P(12,2))).Cells.All(c=>c.ContentCase==Cell.ContentOneofCase.Blank),"Clear Both publishes all targets before filter refresh hides them");
  Check(confirmations==1,"initial exact plan review count");await s.DisposeAsync();Check(s.FinalCharge==0,"zero charges");await File.WriteAllTextAsync(Path.Combine(output,"visible-edit.json"),JsonSerializer.Serialize(new{status="passed",actual_rows=true,clear_both=true,immutable_fill=true,stale_refusal=true,limits="4096/256",fill_preview_roundtrip_ms=fillPreviewMs,clear4096_preview_roundtrip_ms=clearPreviewMs,clear4096_prepare_review_commit_roundtrip_ms=clearRoundtripMs,timing_qualification="One observed SDK/session round trip per case, automatic test confirmation, not core-only or physical GUI latency",remaining_charge=s.FinalCharge}));
 }
}
