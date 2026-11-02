using Pivot.Protocol.Application.V1;
namespace Pivot.Client;
public sealed partial class WorkbookSession {
    public IReadOnlyList<ProducerInfo> Producers {get;private set;}=Array.Empty<ProducerInfo>();
    public bool IsReadOnly => IsWindowScope && Windows.Any(w=>w.Id==WorksheetId&&w.Bound);
    public Task<Response> CreateWorksheetProducerAsync(string expected,string worksheet,Position first,Position last,CancellationToken cancellation=default)=>
        CallAsync(new Request{Operation=Request.Types.Operation.CreateWorksheetProducer,Revision=expected,SourceWorksheet=worksheet,First=first.Clone(),Last=last.Clone()},cancellation);
    public Task<Response> CreateFixtureProducerAsync(string expected,string fixture,bool generationQualified=false,CancellationToken cancellation=default)=>
        CallAsync(new Request{Operation=Request.Types.Operation.CreateFixtureProducer,Revision=expected,Fixture=fixture,GenerationQualified=generationQualified},cancellation);
    public Task<Response> CreateBoundWindowAsync(string expected,string name,string producer,CancellationToken cancellation=default)=>
        CallAsync(new Request{Operation=Request.Types.Operation.CreateBoundWindow,Revision=expected,SheetName=name,ProducerId=producer},cancellation);
    public Task<Response> RefreshProducerAsync(string expected,string producer,uint fixtureVersion=0,CancellationToken cancellation=default)=>
        CallAsync(new Request{Operation=Request.Types.Operation.RefreshProducer,Revision=expected,ProducerId=producer,FixtureVersion=fixtureVersion},cancellation);
    public Task<Response> BeginProducerRefreshAsync(string expected,string producer,CancellationToken cancellation=default)=>
        CallAsync(new Request{Operation=Request.Types.Operation.BeginProducerRefresh,Revision=expected,ProducerId=producer},cancellation);
    public Task<Response> CompleteProducerRefreshAsync(string expected,string producer,string ticket,uint fixtureVersion=0,CancellationToken cancellation=default)=>
        CallAsync(new Request{Operation=Request.Types.Operation.CompleteProducerRefresh,Revision=expected,ProducerId=producer,ProducerTicket=ticket,FixtureVersion=fixtureVersion},cancellation);
    public Task<Response> FailProducerRefreshAsync(string expected,string producer,string ticket,bool cancel=false,CancellationToken cancellation=default)=>
        CallAsync(new Request{Operation=cancel?Request.Types.Operation.CancelProducerRefresh:Request.Types.Operation.FailProducerRefresh,Revision=expected,ProducerId=producer,ProducerTicket=ticket},cancellation);
    public Task<Response> DisconnectProducerAsync(string expected,string producer,CancellationToken cancellation=default)=>
        CallAsync(new Request{Operation=Request.Types.Operation.DisconnectProducer,Revision=expected,ProducerId=producer},cancellation);
    public Task<Response> ReconnectProducerAsync(string expected,string producer,string fixture,CancellationToken cancellation=default)=>
        CallAsync(new Request{Operation=Request.Types.Operation.ReconnectProducer,Revision=expected,ProducerId=producer,Fixture=fixture},cancellation);
    public Task<Response> PrepareProducerDeleteAsync(string expected,string producer,CancellationToken cancellation=default)=>
        CallAsync(new Request{Operation=Request.Types.Operation.PrepareProducerDelete,Revision=expected,ProducerId=producer},cancellation);
    public Task<Response> DeleteProducerAsync(string expected,string producer,string ticket,CancellationToken cancellation=default)=>
        CallAsync(new Request{Operation=Request.Types.Operation.DeleteProducer,Revision=expected,ProducerId=producer,ProducerDeleteTicket=ticket},cancellation);
    public Task<Response> DetachBoundWindowAsync(string expected,string window,string name,CancellationToken cancellation=default)=>
        CallAsync(new Request{Operation=Request.Types.Operation.DetachBoundWindow,Revision=expected,Worksheet=window,WindowScope=true,SheetName=name},cancellation);
}
