using Pivot.Protocol.Application.V1;
namespace Pivot.Client;
public sealed partial class WorkbookSession {
 public Task<Response> StructureAsync(string expected,bool columns,bool delete,uint start,uint count,CancellationToken cancellation=default)=>
  CallAsync(new Request{Operation=Request.Types.Operation.Structure,Revision=expected,StructureColumns=columns,StructureDelete=delete,StructureStart=start,StructureCount=count},cancellation);
}
