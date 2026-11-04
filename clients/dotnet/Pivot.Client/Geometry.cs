using Pivot.Protocol.Application.V1;
using Google.Protobuf;
namespace Pivot.Client;
/// <summary>Complete bounded frame set. Consumers drop displayed references before
/// disposing the capture; a partial set is never returned.</summary>
public sealed class GeometryViewport : IAsyncDisposable
{
    private readonly WorkbookSession session;
    private int disposed;
    public Response Frame {get;}
    public string Capture {get;}
    public string DocumentId {get;}
    public ulong DocumentGeneration {get;}
    public event Action? Releasing;
    public bool IsDisposed=>Volatile.Read(ref disposed)!=0;
    internal GeometryViewport(WorkbookSession session,string capture,Response frame,string document,ulong generation)
    {this.session=session;Capture=capture;Frame=frame;DocumentId=document;DocumentGeneration=generation;}
    public async ValueTask DisposeAsync()
    {
        if(Interlocked.Exchange(ref disposed,1)!=0)return;
        try {Releasing?.Invoke();} finally {
            Frame.Cells.Clear();Frame.Styles.Clear();Frame.Geometry=null;Frame.FilterVisibleMask=ByteString.Empty;Frame.FilterSummary=null;
            await session.ReleaseGeometryCaptureAsync(Capture,DocumentId,DocumentGeneration);
        }
    }
}
public sealed partial class WorkbookSession
{
    internal Task ReleaseGeometryCaptureAsync(string capture,string document,ulong generation)
    {
        // New/Open/Close already invalidate all old-generation server captures.
        // A late client owner still drops its buffers, but cannot release a name
        // against another document generation or a closed session.
        return closed||DocumentId!=document||DocumentGeneration!=generation?Task.CompletedTask:ReleaseAsync(capture);
    }
    private sealed class SelectionInvocation : IDisposable
    {
        private readonly WorkbookSession owner;private readonly SelectionInvocation? previous;
        internal readonly Func<Request,CancellationToken,Task<bool>> Authorize,Revalidate;
        internal SelectionInvocation(WorkbookSession owner,Func<Request,CancellationToken,Task<bool>> authorize,Func<Request,CancellationToken,Task<bool>> revalidate)
        {this.owner=owner;previous=owner.selectionInvocation.Value;Authorize=authorize;Revalidate=revalidate;owner.selectionInvocation.Value=this;}
        public void Dispose(){owner.selectionInvocation.Value=previous;}
    }
    private readonly AsyncLocal<SelectionInvocation?> selectionInvocation=new();
    /// <summary>Client-only UI invocation guard. Explicit programmatic requests
    /// keep exact coordinates; no selection or confirmation is retained by Rust.</summary>
    public IDisposable BeginSelectionOperation(Func<Request,CancellationToken,Task<bool>> authorize,Func<Request,CancellationToken,Task<bool>> revalidate)=>new SelectionInvocation(this,authorize,revalidate);
    private static bool SelectionOperation(Request.Types.Operation operation)=>operation is
        Request.Types.Operation.ClearRegion or Request.Types.Operation.MoveRegion or Request.Types.Operation.CopyBlock or Request.Types.Operation.CopyValues
        or Request.Types.Operation.CopyScope or Request.Types.Operation.PasteBlock or Request.Types.Operation.CopyVisible or Request.Types.Operation.PasteVisible
        or Request.Types.Operation.Fill or Request.Types.Operation.Format or Request.Types.Operation.ClearVisible or Request.Types.Operation.FillVisible;
    private async Task ValidateSelectionDispatchAsync(Request request,CancellationToken cancellation)
    {
        if(SelectionOperation(request.Operation)&&selectionInvocation.Value is { } invocation&&!await invocation.Revalidate(request.Clone(),cancellation).ConfigureAwait(false))
            throw new OperationCanceledException("The section operation context changed before dispatch.",cancellation);
    }
    public Task<Response> ClearRegionAsync(string expected,Position first,Position last,Request.Types.ClearMode mode=Request.Types.ClearMode.Contents,CancellationToken cancellation=default)=>
        CallAsync(new Request{Operation=Request.Types.Operation.ClearRegion,Revision=expected,First=first.Clone(),Last=last.Clone(),ClearMode=mode},cancellation);
    public Task<Response> MoveRegionAsync(string expected,Position first,Position last,Position destination,CancellationToken cancellation=default)=>
        CallAsync(new Request{Operation=Request.Types.Operation.MoveRegion,Revision=expected,First=first.Clone(),Last=last.Clone(),DestinationFirst=destination.Clone()},cancellation);
    public Task<Response> EditGeometryAsync(string expected, IEnumerable<GeometryCommand> commands, CancellationToken cancellation=default)
    {
        var request=new Request { Operation=Request.Types.Operation.GeometryEdit,Revision=expected };
        request.GeometryCommands.AddRange(commands);
        return CallAsync(request,cancellation);
    }
    public Task<Response> PreviewGeometryAsync(string expected, IEnumerable<GeometryCommand> commands, CancellationToken cancellation=default)
    {
        var request=new Request { Operation=Request.Types.Operation.GeometryEdit,Revision=expected,GeometryPreview=true };
        request.GeometryCommands.AddRange(commands.Select(c=>c.Clone()));
        return CallAsync(request,cancellation);
    }
    public Task<Response> ReadGeometryAsync(string capture,GeometryRect viewport,uint after=0,CancellationToken cancellation=default)=>
        CallAsync(new Request { Operation=Request.Types.Operation.GeometryMap,Worksheet=capturedSheets.TryGetValue(capture,out var s)?s.Id:WorksheetId,WindowScope=capturedSheets.TryGetValue(capture,out var w)?w.Window:IsWindowScope,Capture=capture,GeometryViewport=viewport.Clone(),GeometryAfter=after },cancellation);
    public Task<Response> ResolveSectionSelectionAsync(string capture,Position first,Position last,CancellationToken cancellation=default)=>
        CallAsync(new Request { Operation=Request.Types.Operation.GeometrySelection,Worksheet=capturedSheets.TryGetValue(capture,out var s)?s.Id:WorksheetId,WindowScope=capturedSheets.TryGetValue(capture,out var w)?w.Window:IsWindowScope,Capture=capture,First=first.Clone(),Last=last.Clone() },cancellation);

    public Task<Response> ResolvePasteSectionsAsync(string capture,Request operation,CancellationToken cancellation=default)=>
        CallAsync(new Request{Operation=Request.Types.Operation.GeometrySelection,Revision=operation.Revision,Worksheet=operation.Worksheet,WindowScope=operation.WindowScope,
            Capture=capture,First=operation.First.Clone(),Last=operation.Last.Clone(),GeometryPaste=true,GeometryVisiblePaste=operation.Operation==Request.Types.Operation.PasteVisible,
            Clipboard=operation.Clipboard,NativeClipboard=operation.NativeClipboard,ClipboardKind=operation.ClipboardKind},cancellation);
    public async Task<GeometryViewport> LoadGeometryViewportAsync(Position origin,GeometryRect viewport,CancellationToken cancellation=default,Position? active=null)
    {
        var requestedOrigin=origin.Clone();var requestedActive=(active??origin).Clone();var requestedViewport=viewport.Clone();
        var capture=await CaptureAsync(cancellation);
        string document=DocumentId;ulong generation=DocumentGeneration;
        var frame=new Response{Worksheet=capture.Worksheet,WindowScope=capture.WindowScope,Revision=capture.Revision,
            ComputationalState=capture.ComputationalState,CaptureState=capture.ComputationalState,
            CaptureIdentity=capture.Capture,Generation=capture.Capture};
        bool completed=false;
        try {
            uint after=0;uint? total=null;long payload=1024*1024;
            var positions=new HashSet<(uint,uint)>();
            do {
                var page=await CallAsync(new Request{Operation=Request.Types.Operation.GeometryMap,
                    Capture=capture.Capture,Worksheet=capture.Worksheet,WindowScope=capture.WindowScope,
                    GeometryOrigin=requestedOrigin.Clone(),GeometryActive=requestedActive.Clone(),GeometryViewport=requestedViewport.Clone(),GeometryAfter=after},cancellation);
                var geometry=page.Geometry??throw new IOException("Missing canonical geometry frame.");
                if(page.CaptureIdentity!=capture.Capture||page.CaptureState!=capture.ComputationalState||page.Revision!=capture.Revision
                    ||page.Worksheet!=capture.Worksheet||page.WindowScope!=capture.WindowScope
                    ||geometry.Cells.Count>256||geometry.Total>4096||geometry.Viewport is null)
                    throw new IOException("Incoherent geometry capture.");
                if(total is null){
                    total=geometry.Total;frame.Geometry=geometry.Clone();frame.Geometry.Cells.Clear();
                    frame.BoundProjection=geometry.Positional;frame.BoundRows=geometry.Rows;frame.BoundColumns=geometry.Columns;
                    frame.ProducerId=page.ProducerId;frame.ProducerGeneration=page.ProducerGeneration;
                    frame.ProducerConfiguration=page.ProducerConfiguration;frame.ProducerStatus=page.ProducerStatus;
                    frame.FilterVisibleMask=geometry.VisibleMask;frame.FilterSummary=geometry.FilterSummary?.Clone();
                    frame.FilterShownDataRows=geometry.FilterShownDataRows;frame.FilterTotalDataRows=geometry.FilterTotalDataRows;
                    payload=checked(payload+geometry.Headers.Sum(h=>h.CalculateSize()*4L+128)+((geometry.ActiveCell?.CalculateSize()??0)*4L+512));
                    if(geometry.LocalHeaders.Count>8192)throw new IOException("Too many section-local headings.");
                    payload=checked(payload+geometry.LocalHeaders.Sum(h=>h.CalculateSize()*4L+128));
                    foreach(var h in geometry.LocalHeaders){
                        if(h.SectionFirst is null||h.SectionLast is null||h.Index==0||h.First<0||h.Last<=h.First||h.Edge<0
                            ||h.SectionFirst.Row==0||h.SectionFirst.Column==0||h.SectionLast.Row<h.SectionFirst.Row||h.SectionLast.Column<h.SectionFirst.Column
                            ||h.SectionLast.Row>geometry.Rows||h.SectionLast.Column>geometry.Columns
                            ||h.Index<(h.Column?h.SectionFirst.Column:h.SectionFirst.Row)||h.Index>(h.Column?h.SectionLast.Column:h.SectionLast.Row))
                            throw new IOException("Invalid section-local heading.");
                    }
                    frame.FilterStatus=geometry.FilterStatus;frame.FilterError=geometry.FilterError;frame.FilterGeneration=geometry.VisibilityStamp;
                } else if(total!=geometry.Total||!frame.Geometry.Viewport.Equals(geometry.Viewport)
                    ||!frame.Geometry.HorizontalStarts.SequenceEqual(geometry.HorizontalStarts)
                    ||!frame.Geometry.VerticalStarts.SequenceEqual(geometry.VerticalStarts)||frame.FilterGeneration!=geometry.VisibilityStamp||!frame.FilterVisibleMask.Equals(geometry.VisibleMask))
                    throw new IOException("Geometry pages changed within a capture.");
                var data=page.Cells.Select(c=>(c.At.Row,c.At.Column)).ToHashSet();
                var styles=page.Styles.Select(s=>(s.At.Row,s.At.Column)).ToHashSet();
                if(data.Count!=page.Cells.Count||styles.Count!=page.Styles.Count||data.Count!=geometry.Cells.Count||styles.Count!=geometry.Cells.Count)
                    throw new IOException("Incomplete geometry cell payload.");
                foreach(var cell in geometry.Cells){
                    if(cell.At is null||cell.Rectangle is null||cell.At.Row is 0 or > WorkbookCapacity.MaximumRows||cell.At.Column is 0 or > WorkbookCapacity.MaximumColumns
                        ||cell.Rectangle.Left<0||cell.Rectangle.Top<0||cell.Rectangle.Right<=cell.Rectangle.Left||cell.Rectangle.Bottom<=cell.Rectangle.Top
                        ||!positions.Add((cell.At.Row,cell.At.Column))||!data.Contains((cell.At.Row,cell.At.Column))||!styles.Contains((cell.At.Row,cell.At.Column)))
                        throw new IOException("Invalid or duplicate geometry cell.");
                }
                payload=checked(payload+(page.Cells.Sum(c=>(long)c.CalculateSize())+page.Styles.Sum(c=>(long)c.CalculateSize())+geometry.Cells.Sum(c=>(long)c.CalculateSize()))*4L+geometry.Cells.Count*512L);
                if(payload>8*1024*1024)throw new IOException("Viewport exceeds buffered presentation limits; zoom in.");
                frame.Geometry.Cells.Add(geometry.Cells);frame.Cells.Add(page.Cells);frame.Styles.Add(page.Styles);
                uint next=checked(after+(uint)geometry.Cells.Count);
                if(geometry.HasNext){if(geometry.Next!=next||next<=after||next>=geometry.Total)throw new IOException("Invalid geometry cursor.");after=geometry.Next;}
                else {if(next!=geometry.Total||geometry.Next!=0)throw new IOException("Incomplete geometry viewport.");break;}
            }while(true);
            if(frame.Geometry.Cells.Count!=total)throw new IOException("Incomplete geometry frame set.");
            frame.Geometry.HasNext=false;frame.Geometry.Next=0;
            completed=true;
            return new GeometryViewport(this,capture.Capture,frame,document,generation);
        } finally {
            if(!completed){frame.Cells.Clear();frame.Styles.Clear();frame.Geometry?.Cells.Clear();await ReleaseAsync(capture.Capture);}
        }
    }
}
