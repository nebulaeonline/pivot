using Google.Protobuf;
using Pivot.Protocol.Application.V1;
namespace Pivot.Client;
public sealed partial class WorkbookSession {
 public Func<Request,Response,CancellationToken,Task<bool>>? ConfirmVisibleEditAsync {get;set;}
 public Task<Response> ClearVisibleAsync(string expected,Position first,Position last,Request.Types.ClearMode mode,CancellationToken cancellation=default)=>CallAsync(new Request{Operation=Request.Types.Operation.ClearVisible,Revision=expected,First=first.Clone(),Last=last.Clone(),ClearMode=mode},cancellation);
 public Task<Response> FillVisibleAsync(string expected,Position first,Position last,Request.Types.FillDirection direction,CancellationToken cancellation=default)=>CallAsync(new Request{Operation=Request.Types.Operation.FillVisible,Revision=expected,First=first.Clone(),Last=last.Clone(),FillDirection=direction},cancellation);
 private async Task<Response> ConfirmAndEditVisibleAsync(Request operation,CancellationToken cancellation){
  string? interaction=CaptureFilterInteractionAsync is {} capture?await capture(cancellation).ConfigureAwait(false):null;
  cancellation.ThrowIfCancellationRequested();
  // Drain preparation even if cancellation arrives after dispatch, then release its ticket.
  var preview=await SendAsync(new Request{Operation=Request.Types.Operation.PrepareVisibleEdit,Revision=operation.Revision,ComputationalState=operation.ComputationalState,Worksheet=operation.Worksheet,FilterOperation=operation.ToByteString()},CancellationToken.None).ConfigureAwait(false);
  try{
   cancellation.ThrowIfCancellationRequested();
   var confirm=ConfirmVisibleEditAsync??throw new WorkbookSessionException("Visible Clear/Fill requires exact-row review.");
   if(!await confirm(operation.Clone(),preview.Clone(),cancellation).ConfigureAwait(false))throw new OperationCanceledException("Visible edit cancelled.",cancellation);
   cancellation.ThrowIfCancellationRequested();
   if(CaptureFilterInteractionAsync is {} current&&interaction!=await current(cancellation).ConfigureAwait(false))throw new OperationCanceledException("Visible edit interaction changed; review again.",cancellation);
   operation.FilterConfirmation=preview.FilterConfirmation;
   return await SendAsync(operation,cancellation).ConfigureAwait(false);
  }finally{
   // Matching ticket only: never invalidate an unrelated later prepared operation.
   // Also harmless after a consumed dispatch, including cancellation before it reached Rust.
   try{await SendAsync(new Request{Operation=Request.Types.Operation.CancelVisibleEdit,FilterConfirmation=preview.FilterConfirmation},CancellationToken.None).ConfigureAwait(false);}catch(Exception) when(closed){}
  }
 }
}
