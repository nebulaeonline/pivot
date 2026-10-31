using Google.Protobuf;
using Pivot.Protocol.Application.V1;
namespace Pivot.Client;
public sealed partial class WorkbookSession {
 public FilterSummary? FilterSummary {get;private set;}
 public uint FilterStatus {get;private set;}
 public string FilterGeneration {get;private set;}="";
 public string FilterError {get;private set;}="";
 public Func<Request,Response,CancellationToken,Task<bool>>? ConfirmHiddenRowsAsync {get;set;}
 public Func<CancellationToken,Task<string>>? CaptureFilterInteractionAsync {get;set;}
 public Func<Request,CancellationToken,Task<bool>>? RevalidateFilteredOperationAsync {get;set;}
 public Task<Response> GetFilterAsync(CancellationToken cancellation=default)=>CallAsync(new Request{Operation=Request.Types.Operation.GetFilter,Revision=Revision},cancellation);
 public Task<Response> SetFilterAsync(string expected,FilterDefinition? definition,CancellationToken cancellation=default)=>CallAsync(new Request{Operation=Request.Types.Operation.SetFilter,Revision=expected,FilterDefinition=definition?.Clone()},cancellation);
 public Task<Response> ValidateFilterAsync(string expected,FilterDefinition definition,CancellationToken cancellation=default)=>CallAsync(new Request{Operation=Request.Types.Operation.ValidateFilter,Revision=expected,FilterDefinition=definition.Clone()},cancellation);
 public Task<Response> ReapplyFilterAsync(CancellationToken cancellation=default)=>CallAsync(new Request{Operation=Request.Types.Operation.ReapplyFilter,Revision=Revision},cancellation);
 public Task<Response> ReadFilteredAsync(uint firstVisible,uint count,uint firstColumn,uint lastColumn,CancellationToken cancellation=default)=>CallAsync(new Request{Operation=Request.Types.Operation.ReadFiltered,Revision=Revision,First=new Position{Row=1,Column=firstColumn},Last=new Position{Row=1,Column=lastColumn},FilterFirstVisible=firstVisible,FilterRowCount=count},cancellation);
 private static bool FilterBulk(Request.Types.Operation op)=>op is Request.Types.Operation.Apply or Request.Types.Operation.CopyBlock or Request.Types.Operation.PasteBlock or Request.Types.Operation.Fill or Request.Types.Operation.Format or Request.Types.Operation.Sort or Request.Types.Operation.Structure or Request.Types.Operation.ExportCsv;
 private async Task<Response> CallAsync(Request request,CancellationToken cancellation){
  // Capture the exact logical operation before previewing it or awaiting a modal decision.
  request=request.Clone();if(request.Worksheet.Length==0)request.Worksheet=WorksheetId;
  if(request.Revision.Length==0)request.Revision=Revision;
  if(request.ComputationalState.Length==0)request.ComputationalState=ComputationalState;
  if(request.Operation==Request.Types.Operation.PasteVisible)return await ConfirmAndPasteVisibleAsync(request,cancellation).ConfigureAwait(false);
  bool singleEdit=request.Operation==Request.Types.Operation.Apply&&request.Edits.Count==1&&request.Edits[0].ActionCase!=Edit.ActionOneofCase.MoveTo;
  if(FilterSummary is not null&&FilterBulk(request.Operation)&&!singleEdit&&!IsBindingProof&&!IsProjectionProof){
   string? interaction=CaptureFilterInteractionAsync is { } capture?await capture(cancellation).ConfigureAwait(false):null;
   var preview=await SendAsync(new Request{Operation=Request.Types.Operation.PrepareFilteredOperation,Worksheet=request.Worksheet,Revision=request.Revision,ComputationalState=request.ComputationalState,FilterOperation=request.ToByteString()},cancellation).ConfigureAwait(false);
   if(preview.FilterHiddenRows!=0){
    var confirm=ConfirmHiddenRowsAsync??throw new WorkbookSessionException("The operation includes filtered-out rows; an explicit exact-operation confirmation is required.");
    if(!await confirm(request.Clone(),preview.Clone(),cancellation).ConfigureAwait(false))throw new OperationCanceledException("Hidden-row operation cancelled.",cancellation);
    cancellation.ThrowIfCancellationRequested();request.FilterConfirmation=preview.FilterConfirmation;
    if(CaptureFilterInteractionAsync is { } current&&interaction!=await current(cancellation).ConfigureAwait(false))throw new OperationCanceledException("The operation target changed during confirmation.",cancellation);
    if(RevalidateFilteredOperationAsync is { } validate&&!await validate(request.Clone(),cancellation).ConfigureAwait(false))throw new OperationCanceledException("The clipboard payload or operation context changed during confirmation.",cancellation);
    if(CaptureFilterInteractionAsync is { } final&&interaction!=await final(cancellation).ConfigureAwait(false))throw new OperationCanceledException("The operation target changed before dispatch.",cancellation);
   }
  }
  return await SendAsync(request,cancellation).ConfigureAwait(false);
 }
}
