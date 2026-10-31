using Google.Protobuf;
using Pivot.Protocol.Application.V1;
namespace Pivot.Client;
public sealed partial class WorkbookSession {
 public Func<Request,Response,CancellationToken,Task<bool>>? ConfirmVisiblePasteAsync {get;set;}
 private static uint NativeClipboardKind(byte[] payload,bool native)=>!native?0u:payload.AsSpan().StartsWith("PIVOTCB6"u8)?6u:5u;
 public Task<Response> CopyVisibleAsync(string expected,Position first,Position last,CancellationToken cancellation=default)=>CallAsync(new Request{Operation=Request.Types.Operation.CopyVisible,Revision=expected,First=first.Clone(),Last=last.Clone()},cancellation);
 public Task<Response> PasteVisibleAsync(string expected,Position first,Position last,byte[] payload,bool native,CancellationToken cancellation=default){
  if(payload.Length>262144)throw new WorkbookSessionException("Clipboard payload exceeds 256 KiB.");
  return CallAsync(new Request{Operation=Request.Types.Operation.PasteVisible,Revision=expected,First=first.Clone(),Last=last.Clone(),Clipboard=ByteString.CopyFrom(payload),NativeClipboard=native,ClipboardKind=NativeClipboardKind(payload,native)},cancellation);
 }
 private async Task<Response> ConfirmAndPasteVisibleAsync(Request operation,CancellationToken cancellation){
  string? interaction=CaptureFilterInteractionAsync is { } capture?await capture(cancellation).ConfigureAwait(false):null;
  var preview=await SendAsync(new Request{Operation=Request.Types.Operation.PrepareVisiblePaste,Revision=operation.Revision,ComputationalState=operation.ComputationalState,Worksheet=operation.Worksheet,FilterOperation=operation.ToByteString()},cancellation).ConfigureAwait(false);
  var confirm=ConfirmVisiblePasteAsync??throw new WorkbookSessionException("Paste Visible requires review of the actual destination rows.");
  if(!await confirm(operation.Clone(),preview.Clone(),cancellation).ConfigureAwait(false))throw new OperationCanceledException("Visible paste cancelled.",cancellation);
  cancellation.ThrowIfCancellationRequested();operation.FilterConfirmation=preview.FilterConfirmation;
  if(CaptureFilterInteractionAsync is { } current&&interaction!=await current(cancellation).ConfigureAwait(false))throw new OperationCanceledException("Visible destination changed during review.",cancellation);
  if(RevalidateFilteredOperationAsync is { } validate&&!await validate(operation.Clone(),cancellation).ConfigureAwait(false))throw new OperationCanceledException("Clipboard changed during visible paste review.",cancellation);
  if(CaptureFilterInteractionAsync is { } final&&interaction!=await final(cancellation).ConfigureAwait(false))throw new OperationCanceledException("Visible destination changed before dispatch.",cancellation);
  return await SendAsync(operation,cancellation).ConfigureAwait(false);
 }
}
