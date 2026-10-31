using Pivot.Protocol.Application.V1;
namespace Pivot.Client;
public sealed partial class WorkbookSession {
 public Task<Response> SortBoundsAsync(string expected,Position first,Position last,CancellationToken cancellation=default)=>
  CallAsync(new Request{Operation=Request.Types.Operation.SortBounds,Revision=expected,First=first.Clone(),Last=last.Clone()},cancellation);
 public Task<Response> SortAsync(string expected,Position first,Position last,bool excludeHeader,IEnumerable<SortKey> keys,bool moveFormatting=true,CancellationToken cancellation=default){
  var request=new Request{Operation=Request.Types.Operation.Sort,Revision=expected,First=first.Clone(),Last=last.Clone(),SortExcludeHeader=excludeHeader,SortMoveFormatting=moveFormatting};
  request.SortKeys.AddRange(keys.Select(k=>k.Clone()));return CallAsync(request,cancellation);
 }
}
