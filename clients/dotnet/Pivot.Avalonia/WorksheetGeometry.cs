using System.Globalization;
using Avalonia;
using Avalonia.Media;
using Avalonia.Input;
using Pivot.Client;
using Pivot.Protocol.Application.V1;

namespace Pivot.Avalonia;

public sealed partial class WorksheetView
{
    private GeometryViewport? geometryViewport;
    private bool geometryEnabled;
    private readonly Dictionary<(uint,uint),GeometryCell> geometryCells=[];
    private GeometryFrame? sectionSelection;
    private Task selectionResolution=Task.CompletedTask;
    private long selectionVersion,resolvedSelectionVersion;
    private string selectionError="";
    private (Position First,Position Last)? previousGeometrySelection;
    private bool backgroundGeometryChange;
    public bool GeometryMode=>geometryEnabled;
    public void EnterGeometryMode(){if(geometryEnabled)return;geometryEnabled=true;GeometryInputReady=false;GeometryHeaderInputReady=false;cells.Clear();styles.Clear();geometryCells.Clear();InvalidateVisual();}
    public bool GeometryInputReady {get;set;}
    public bool GeometryHeaderInputReady {get;set;}
    public bool NeedsGeometryHeaders {
        get {
            if(geometryViewport is not {IsDisposed:false} owner||owner.Frame.Geometry.ActiveCell?.At is not { } original)return false;
            uint Band(IEnumerable<uint> starts,uint at)=>starts.LastOrDefault(n=>n<=at);
            return Band(owner.Frame.Geometry.HorizontalStarts,original.Row)!=Band(owner.Frame.Geometry.HorizontalStarts,Selected.Row)
                ||Band(owner.Frame.Geometry.VerticalStarts,original.Column)!=Band(owner.Frame.Geometry.VerticalStarts,Selected.Column);
        }
    }
    public int GeometryZoomPercent {get;private set;}=100;
    public int PaintedGeometryZoom=>paintedGeometryZoom;
    private int paintedGeometryZoom=100;
    public bool SectionSelectionEnlarged=>sectionSelection?.Enlarged==true;
    public bool SelectionNeedsReview {get;private set;}
    public Position RequestedFirst=>new(){Row=Math.Min(anchor.Row,active.Row),Column=Math.Min(anchor.Column,active.Column)};
    public Position RequestedLast=>new(){Row=Math.Max(anchor.Row,active.Row),Column=Math.Max(anchor.Column,active.Column)};
    public string SectionSelectionError=>selectionError;
    public Func<GeometryViewport,Position,Position,Task<GeometryFrame>>? ResolveSectionSelection {get;set;}
    public void CancelHeadingGesture(){EndPointerSelection(false);CancelGeometryDivider();}
    public async Task<bool> SelectLocalHeadingAsync(GeometryViewport owner,GeometryLocalHeader header,bool extend,bool corner=false)
    {
        if(owner!=geometryViewport||owner.IsDisposed||!GeometryInputReady||!GeometryHeaderInputReady)return false;
        EndPointerSelection(false);long token=++gesture;
        var first=header.SectionFirst.Clone();var last=header.SectionLast.Clone();
        if(!corner){
            if(header.Column){first.Column=last.Column=header.Index;if(extend){first.Row=Math.Min(first.Row,Anchor.Row);last.Row=Math.Max(last.Row,Anchor.Row);first.Column=Math.Min(Anchor.Column,header.Index);last.Column=Math.Max(Anchor.Column,header.Index);}}
            else{first.Row=last.Row=header.Index;if(extend){first.Column=Math.Min(first.Column,Anchor.Column);last.Column=Math.Max(last.Column,Anchor.Column);first.Row=Math.Min(Anchor.Row,header.Index);last.Row=Math.Max(Anchor.Row,header.Index);}}
        }
        var focus=header.Column?new Position{Row=header.SectionFirst.Row,Column=header.Index}:new Position{Row=header.Index,Column=header.SectionFirst.Column};
        bool Current()=>token==gesture&&owner==geometryViewport&&!owner.IsDisposed&&GeometryInputReady&&GeometryHeaderInputReady;
        if(SelectionRequested is null||!await SelectionRequested(focus,false,false,Current)||!Current())return false;
        SelectStructuralBand(first,last,focus);Focus();return true;
    }
    public void SetGeometryZoom(int percent)
    {
        if(percent is <25 or >3200)throw new ArgumentOutOfRangeException(nameof(percent));
        if(percent==GeometryZoomPercent)return;
        EndPointerSelection(false);GeometryZoomPercent=percent;GeometryInputReady=false;ViewportChanged?.Invoke();
    }
    public GeometryRect RequestedGeometryViewport()=>new(){
        Right=Math.Max(1,(long)Math.Ceiling(Math.Max(1,Bounds.Width-Header)*4800/GeometryZoomPercent)),
        Bottom=Math.Max(1,(long)Math.Ceiling(Math.Max(1,Bounds.Height-HeightPerCell)*4800/GeometryZoomPercent))};
    public void SetGeometryViewport(GeometryViewport? viewport,int zoom=100,bool preservePendingSelection=false)
    {
        var old=geometryViewport;
        bool sameScope=old is not null&&viewport is not null&&old.DocumentId==viewport.DocumentId
            &&old.Frame.Worksheet==viewport.Frame.Worksheet&&old.Frame.WindowScope==viewport.Frame.WindowScope;
        bool changed=sameScope&&(old!.Frame.Revision!=viewport!.Frame.Revision||old.Frame.FilterGeneration!=viewport.Frame.FilterGeneration);
        bool ownEditorCommit=preservePendingSelection&&selectionPending&&sameScope
            &&old!.Frame.FilterGeneration==viewport!.Frame.FilterGeneration;
        if(!sameScope||(changed&&!ownEditorCommit)||paintedGeometryZoom!=zoom)EndPointerSelection(false);
        if(changed){previousGeometrySelection=(SelectionFirst,SelectionLast);backgroundGeometryChange=true;}
        if(!sameScope){SelectionNeedsReview=false;previousGeometrySelection=null;backgroundGeometryChange=false;}
        var retainedSelection=sameScope&&!changed&&sectionSelection?.RequestedFirst?.Equals(RequestedFirst)==true&&sectionSelection.RequestedLast?.Equals(RequestedLast)==true?sectionSelection:null;
        if(old is not null)old.Releasing-=GeometryReleasing;
        geometryEnabled=viewport is not null;geometryViewport=viewport;if(viewport is not null)viewport.Releasing+=GeometryReleasing;paintedGeometryZoom=zoom;geometryCells.Clear();sectionSelection=retainedSelection;selectionError="";
        GeometryInputReady=viewport is {IsDisposed:false};
        if(viewport is not null){
            foreach(var cell in viewport.Frame.Geometry.Cells)geometryCells.Add((cell.At.Row,cell.At.Column),cell);
            SetGeometryMask(viewport.Frame);
            if(viewport.Frame.Geometry.ScrollAnchor is { } scroll){FirstRow=scroll.Row;FirstColumn=scroll.Column;}
            SetCells(viewport.Frame.Cells);SetStyles(viewport.Frame.Styles);
            if(viewport.Frame.Geometry.ActiveCell is { } activeCell&&(activeCell.CellId.Length!=0||activeCell.Projected is not null||activeCell.ContentCase!=Cell.ContentOneofCase.Blank))cells[(activeCell.At.Row,activeCell.At.Column)]=activeCell;
            if(viewport.Frame.Geometry.ActiveCell is { } styled&&viewport.Frame.Geometry.ActiveStyle.Length!=0)styles[(styled.At.Row,styled.At.Column)]=CellStyle.Decode(viewport.Frame.Geometry.ActiveStyle);
        }
        if(viewport is null){cells.Clear();styles.Clear();}
        QueueSectionResolution();InvalidateVisual();
    }
    private void QueueSectionResolution()
    {
        ++selectionVersion;
        if(!GeometryMode){sectionSelection=null;resolvedSelectionVersion=selectionVersion;return;}
        if(selectionResolution.IsCompleted)selectionResolution=ResolveSectionsAsync();
    }
    private async Task ResolveSectionsAsync()
    {
        while(resolvedSelectionVersion!=selectionVersion){
            long version=selectionVersion;var owner=geometryViewport;var first=RequestedFirst;var last=RequestedLast;
            if(owner is null||owner.IsDisposed||ResolveSectionSelection is null)return;
            if(owner.Frame.Geometry.Rows==0||owner.Frame.Geometry.Columns==0){resolvedSelectionVersion=version;return;}
            try {
                var resolved=await ResolveSectionSelection(owner,first,last);
                if(version!=selectionVersion||owner!=geometryViewport||owner.IsDisposed)continue;
                if(resolved.RequestedFirst is null||resolved.RequestedLast is null||resolved.EffectiveFirst is null||resolved.EffectiveLast is null
                    ||!resolved.RequestedFirst.Equals(first)||!resolved.RequestedLast.Equals(last))throw new InvalidDataException("Invalid section selection response.");
                sectionSelection=resolved;selectionError="";resolvedSelectionVersion=version;
                if(backgroundGeometryChange&&previousGeometrySelection is { } previous){
                    if(resolved.EffectiveFirst.Row<previous.First.Row||resolved.EffectiveFirst.Column<previous.First.Column
                        ||resolved.EffectiveLast.Row>previous.Last.Row||resolved.EffectiveLast.Column>previous.Last.Column)SelectionNeedsReview=true;
                    backgroundGeometryChange=false;previousGeometrySelection=null;
                }
                InvalidateVisual();SelectionChanged?.Invoke();
            } catch(Exception error) {
                if(version!=selectionVersion||owner!=geometryViewport)continue;
                selectionError=error.Message;return;
            }
        }
    }
    public async Task<bool> EnsureSectionSelectionAsync()
    {
        if(!GeometryMode)return true;
        var waiting=System.Diagnostics.Stopwatch.StartNew();
        do {
            if(resolvedSelectionVersion!=selectionVersion&&selectionResolution.IsCompleted)selectionResolution=ResolveSectionsAsync();
            await selectionResolution;
            if(SelectionNeedsReview||selectionError.Length!=0)return false;
            if(GeometryInputReady&&resolvedSelectionVersion==selectionVersion)return true;
            // A label/layout update may have queued a newer captured viewport.
            // Wait only for presentation reads; never repeat the user's operation.
            await Task.Delay(10);
        }while(GeometryMode&&waiting.Elapsed<TimeSpan.FromSeconds(1));
        return false;
    }
    private Rect GeometryBounds(GeometryCell cell)
    {
        var origin=geometryViewport!.Frame.Geometry.Viewport;var rectangle=cell.Rectangle;
        double scale=paintedGeometryZoom/4800.0;
        return new Rect(Header+(rectangle.Left-origin.Left)*scale,HeightPerCell+(rectangle.Top-origin.Top)*scale,
            (rectangle.Right-rectangle.Left)*scale,(rectangle.Bottom-rectangle.Top)*scale);
    }
    private Position? GeometryPositionAt(Point point)
    {
        if(!GeometryInputReady||geometryViewport is not {IsDisposed:false} owner||point.X<Header||point.Y<HeightPerCell)return null;
        var origin=owner.Frame.Geometry.Viewport;
        try {
            long x=checked(origin.Left*256+(long)Math.Floor((point.X-Header)*4800/paintedGeometryZoom*256));
            long y=checked(origin.Top*256+(long)Math.Floor((point.Y-HeightPerCell)*4800/paintedGeometryZoom*256));
            foreach(var cell in geometryCells.Values){var rectangle=cell.Rectangle;
                if(x>=checked(rectangle.Left*256)&&x<checked(rectangle.Right*256)&&y>=checked(rectangle.Top*256)&&y<checked(rectangle.Bottom*256))return cell.At.Clone();
            }
        } catch(OverflowException) { }
        return null;
    }
    private IEnumerable<GeometryHeader> GeometryHeaders(bool columns)=>geometryViewport!.Frame.Geometry.Headers.Where(h=>h.Column==columns);
    private Rect GeometryHeaderBounds(GeometryHeader header){
        var origin=geometryViewport!.Frame.Geometry.Viewport;double scale=paintedGeometryZoom/4800.0;
        return header.Column?new Rect(Header+(header.First-origin.Left)*scale,0,(header.Last-header.First)*scale,HeightPerCell)
            :new Rect(0,HeightPerCell+(header.First-origin.Top)*scale,Header,(header.Last-header.First)*scale);
    }
    private void GeometryReleasing(){
        GeometryInputReady=false;EndPointerSelection(false);geometryCells.Clear();cells.Clear();styles.Clear();sectionSelection=null;++selectionVersion;InvalidateVisual();
    }
    private async Task<bool> GeometryHeaderPressed(Point point,bool extend,Func<bool> current)
    {
        if(!GeometryHeaderInputReady||!GeometryInputReady||geometryViewport is not {IsDisposed:false})return true;
        bool columns=point.Y<HeightPerCell&&point.X>=Header,rows=point.X<Header&&point.Y>=HeightPerCell;
        if(!columns&&!rows)return false;
        var header=GeometryHeaders(columns).FirstOrDefault(c=>{var rectangle=GeometryHeaderBounds(c);return columns?point.X>=rectangle.Left&&point.X<rectangle.Right:point.Y>=rectangle.Top&&point.Y<rectangle.Bottom;});
        if(header is null)return true;
        var band=(First:header.SectionFirst,Last:header.SectionLast);
        var first=columns?new Position{Row=band.First,Column=extend?Math.Min(anchor.Column,header.Index):header.Index}
            :new Position{Row=extend?Math.Min(anchor.Row,header.Index):header.Index,Column=band.First};
        var last=columns?new Position{Row=band.Last,Column=extend?Math.Max(anchor.Column,header.Index):header.Index}
            :new Position{Row=extend?Math.Max(anchor.Row,header.Index):header.Index,Column=band.Last};
        var focus=columns?new Position{Row=Selected.Row,Column=header.Index}:new Position{Row=header.Index,Column=Selected.Column};
        if(SelectionRequested is not null&&await SelectionRequested(focus,false,false,current)&&current()){SelectStructuralBand(first,last,focus);Focus();}
        return true;
    }
    public Func<GeometryCommand,string,Task>? GeometryResizeRequested {get;set;}
    private sealed record DividerDrag(IPointer Pointer,GeometryViewport Owner,GeometryHeader Header,Point Start,long FirstSize,long SecondSize,bool Shared);
    private DividerDrag? dividerDrag;
    private long dividerDelta;
    private bool BeginGeometryDivider(PointerPressedEventArgs args,Point point)
    {
        if(!GeometryInputReady||!GeometryHeaderInputReady||geometryViewport is not {IsDisposed:false} owner||GeometryResizeRequested is null)return false;
        bool columns=point.Y<HeightPerCell&&point.X>=Header,rows=point.X<Header&&point.Y>=HeightPerCell;
        if(!columns&&!rows)return false;
        var headers=GeometryHeaders(columns).ToArray();
        var header=headers.FirstOrDefault(h=>{var r=GeometryHeaderBounds(h);return Math.Abs((columns?point.X:point.Y)-(columns?r.Right:r.Bottom))<=4;});
        if(header is null)return false;
        var next=headers.FirstOrDefault(h=>h.Index==header.Index+1&&h.First==header.Last);
        if(next is null)return false;
        EndPointerSelection(false);
        dividerDrag=new(args.Pointer,owner,header,point,header.Last-header.First,next.Last-next.First,args.KeyModifiers.HasFlag(KeyModifiers.Alt));dividerDelta=0;
        args.Pointer.Capture(this);Focus();InvalidateVisual();return true;
    }
    private void CancelGeometryDivider()
    {
        var drag=dividerDrag;dividerDrag=null;dividerDelta=0;drag?.Pointer.Capture(null);InvalidateVisual();
    }
    private bool MoveGeometryDivider(PointerEventArgs args)
    {
        var drag=dividerDrag;if(drag is null||args.Pointer!=drag.Pointer)return false;
        if(drag.Owner!=geometryViewport||drag.Owner.IsDisposed||!GeometryInputReady){CancelGeometryDivider();return true;}
        var point=args.GetPosition(this);
        long delta=(long)Math.Round(((drag.Header.Column?point.X-drag.Start.X:point.Y-drag.Start.Y)*4800/paintedGeometryZoom),MidpointRounding.AwayFromZero);
        long low=Math.Max(64-drag.FirstSize,drag.SecondSize-262144),high=Math.Min(262144-drag.FirstSize,drag.SecondSize-64);
        dividerDelta=Math.Clamp(delta,low,high);InvalidateVisual();return true;
    }
    private async Task<bool> FinishGeometryDivider(PointerReleasedEventArgs args)
    {
        var drag=dividerDrag;if(drag is null||args.Pointer!=drag.Pointer)return false;
        MoveGeometryDivider(args);long delta=dividerDelta;
        bool valid=drag.Owner==geometryViewport&&!drag.Owner.IsDisposed&&GeometryInputReady;
        CancelGeometryDivider();
        if(valid&&delta!=0&&GeometryResizeRequested is { } resize){
            var command=new GeometryCommand{Action=GeometryCommand.Types.Action.SetSizes,Horizontal=drag.Header.Column,At=drag.Header.Column?Selected.Row:Selected.Column,First=drag.Header.Index,Last=drag.Header.Index,Size=drag.FirstSize+delta,Paired=true,Shared=drag.Shared};
            await resize(command,$"{(command.Horizontal?"Widths":"Heights")} {command.First}/{command.First+1}: {(drag.FirstSize+delta)/64m:0.######} / {(drag.SecondSize-delta)/64m:0.######} pt; {(command.Shared?"EXPLICIT shared batch across all profiles":"local profile only")}; total preserved");
        }
        return true;
    }
    private void RenderGeometry(DrawingContext context)
    {
        context.FillRectangle(Brushes.WhiteSmoke,new Rect(Bounds.Size));
        if(geometryViewport is not {IsDisposed:false})return;
        var first=SelectionFirst;var last=SelectionLast;var focused=Selected;var border=new Pen(Brushes.LightGray,1);
        using(context.PushClip(new Rect(Header,HeightPerCell,Math.Max(0,Bounds.Width-Header),Math.Max(0,Bounds.Height-HeightPerCell)))){
            foreach(var mapped in geometryCells.Values){
                var at=mapped.At;var rectangle=GeometryBounds(mapped);var style=StyleAt(at);
                bool selected=at.Equals(focused),region=at.Row>=first.Row&&at.Row<=last.Row&&at.Column>=first.Column&&at.Column<=last.Column;
                IBrush fill=region?Brushes.AliceBlue:Brushes.White;
                if(style.Background is uint bg)fill=new SolidColorBrush(Color.FromUInt32(0xff000000|bg));
                context.DrawRectangle(fill,selected?new Pen(ActiveFocus?Brushes.DodgerBlue:Brushes.SlateGray,2):region?new Pen(ActiveFocus?Brushes.DodgerBlue:Brushes.SlateGray,1):border,rectangle);
                var cell=CellAt(at);string text=cell is null?"":cell.Error.Length>0?cell.ErrorCode=="ReferenceDeleted"?"#REF!":"#ERROR":cell.ContentCase==Cell.ContentOneofCase.Text?cell.Text:cell.FormattedNumber.Length>0?cell.FormattedNumber:WorkbookSession.DisplayValue(cell);
                IBrush ink=cell?.Error.Length>0?Brushes.Firebrick:Brushes.Black;
                if(style.Foreground is uint fg)ink=new SolidColorBrush(Color.FromUInt32(0xff000000|fg));
                var face=new Typeface("Inter",style.Italic?FontStyle.Italic:FontStyle.Normal,style.Bold?FontWeight.Bold:FontWeight.Normal);
                var formatted=new FormattedText(text,CultureInfo.InvariantCulture,FlowDirection.LeftToRight,face,13*paintedGeometryZoom/100.0,ink);
                int alignment=style.Alignment;
                if(alignment==0)alignment=cell?.Result?.KindCase==TypedValue.KindOneofCase.Decimal?3:cell?.Result?.KindCase==TypedValue.KindOneofCase.Boolean?2:1;
                double x=alignment==3?Math.Max(rectangle.Left+3,rectangle.Right-3-formatted.Width):alignment==2?Math.Max(rectangle.Left+3,rectangle.Left+(rectangle.Width-formatted.Width)/2):rectangle.Left+3;
                if(rectangle.Width>2&&rectangle.Height>2)using(context.PushClip(rectangle.Deflate(1)))context.DrawText(formatted,new Point(x,rectangle.Top+2));
                bool filterHeader=rectangle.Width>=24&&rectangle.Height>=14&&FilterSummary is {ExcludeHeader:true,NeedsReview:false,First:{ } filterFirst}&&at.Row==filterFirst.Row&&at.Column>=filterFirst.Column&&at.Column<=FilterSummary.Last.Column;
                if(filterHeader){var mark=new FormattedText("\u25be",CultureInfo.InvariantCulture,FlowDirection.LeftToRight,new Typeface("Inter"),12,FilterSummary!.Columns.Contains(at.Column)?Brushes.SlateBlue:Brushes.Gray);context.DrawText(mark,new Point(rectangle.Right-16,rectangle.Top+2));}
                if(at.Row>1&&geometryViewport.Frame.Geometry.HorizontalStarts.Contains(at.Row))context.DrawLine(new Pen(Brushes.SeaGreen,2),rectangle.TopLeft,rectangle.TopRight);
                if(at.Column>1&&geometryViewport.Frame.Geometry.VerticalStarts.Contains(at.Column))context.DrawLine(new Pen(Brushes.SeaGreen,2),rectangle.TopLeft,rectangle.BottomLeft);
                if(visiblePasteFeedback.Contains((at.Row,at.Column)))context.DrawRectangle(null,new Pen(Brushes.SeaGreen,2),rectangle.Deflate(Math.Min(3,Math.Min(rectangle.Width,rectangle.Height)/4)));
            }
        }
        if(dividerDrag is { } drag){
            var header=GeometryHeaderBounds(drag.Header);double shift=dividerDelta*paintedGeometryZoom/4800.0;
            if(drag.Header.Column)context.DrawLine(new Pen(Brushes.DarkOrange,2),new Point(header.Right+shift,0),new Point(header.Right+shift,Bounds.Height));
            else context.DrawLine(new Pen(Brushes.DarkOrange,2),new Point(0,header.Bottom+shift),new Point(Bounds.Width,header.Bottom+shift));
            string sizes=$"{(drag.FirstSize+dividerDelta)/64m:0.######} / {(drag.SecondSize-dividerDelta)/64m:0.######} pt · {(drag.Shared?"shared batch":"local profile")}";
            context.DrawText(new FormattedText(sizes,CultureInfo.InvariantCulture,FlowDirection.LeftToRight,new Typeface("Inter"),12,Brushes.DarkOrange),new Point(Header+8,HeightPerCell+8));
        }
        foreach(bool columns in new[]{true,false})foreach(var mapped in GeometryHeaders(columns)){
            var header=GeometryHeaderBounds(mapped);
            context.DrawRectangle(Brushes.WhiteSmoke,border,header);
            string text=columns?Address(new Position{Row=1,Column=mapped.Index}).TrimEnd('1'):mapped.Index.ToString(CultureInfo.InvariantCulture);
            var label=new FormattedText(text,CultureInfo.InvariantCulture,FlowDirection.LeftToRight,new Typeface("Inter"),12,Brushes.SlateGray);
            using(context.PushClip(header))context.DrawText(label,new Point(header.Left+(header.Width-label.Width)/2,header.Top+(header.Height-label.Height)/2));
        }
    }
}
