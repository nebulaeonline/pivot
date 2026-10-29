using System.Globalization;
using System.Diagnostics;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Media;
using Avalonia.Threading;
using Pivot.Client;
using Pivot.Protocol.Application.V1;

namespace Pivot.Avalonia;

/// <summary>Paints only 20 x 10 cells. Scrolling changes the Rust viewport;
/// no worksheet-sized control collection or client calculation model.</summary>
public sealed partial class WorksheetView : Control
{
    public bool ActiveFocus { get; set; }
    public const int Rows = 20, Columns = 10;
    private const double Header = 48, WidthPerCell = 94, HeightPerCell = 26;
    private readonly Dictionary<(uint, uint), Cell> cells = [];
    private readonly Dictionary<(uint,uint),CellStyle> styles=[];
    public CellStyle StyleAt(Position at)=>styles.GetValueOrDefault((at.Row,at.Column))??new();
    public void SetStyles(IEnumerable<StyledPosition> values) {styles.Clear();foreach(var v in values)styles[(v.At.Row,v.At.Column)]=CellStyle.Decode(v.Style);InvalidateVisual();}
    private TaskCompletionSource<TimeSpan>? paint;
    private long paintStart;
    private bool awaitingPaint;
    public Task<TimeSpan> PaintCompletion => paint?.Task ?? Task.FromResult(TimeSpan.Zero);
    public Cell? CellAt(Position at) => cells.GetValueOrDefault((at.Row, at.Column));
    public uint FirstRow { get; private set; } = 1;
    public uint FirstColumn { get; private set; } = 1;
    public int VisibleRows => Math.Clamp((int)(Bounds.Height / HeightPerCell) - 1, 1, Rows);
    private Position active = new() { Row=1, Column=1 }, anchor = new() { Row=1, Column=1 };
    public Position Selected => active.Clone();
    public Position Anchor => anchor.Clone();
    public Position SelectionFirst => new() { Row=Math.Min(anchor.Row,active.Row), Column=Math.Min(anchor.Column,active.Column) };
    public Position SelectionLast => new() { Row=Math.Max(anchor.Row,active.Row), Column=Math.Max(anchor.Column,active.Column) };
    public bool HasRegion => anchor.Row!=active.Row || anchor.Column!=active.Column;
    private readonly DispatcherTimer dragTimer = new() { Interval=TimeSpan.FromMilliseconds(75) };
    private IPointer? dragPointer;
    private Point dragPoint;
    private bool dragReady, selectionPending;
    private long gesture;
    private Position previousAnchor = new() { Row=1,Column=1 }, previousActive = new() { Row=1,Column=1 };
    public bool IsSelecting => dragPointer is not null;
    public bool IsSelectionPending => selectionPending;
    public event Action? ViewportChanged;
    public event Action? SelectionChanged;
    public Func<Position, bool, bool, Func<bool>, Task<bool>>? SelectionRequested { get; set; }
    public void RestoreView(uint firstRow,uint firstColumn,Position selectedAnchor,Position selectedActive) {
        EndPointerSelection(false);
        FirstRow=Math.Clamp(firstRow,1,1025-(uint)VisibleRows);FirstColumn=Math.Clamp(firstColumn,1,1025-Columns);
        anchor=new(){Row=Math.Clamp(selectedAnchor.Row,1,1024),Column=Math.Clamp(selectedAnchor.Column,1,1024)};
        active=new(){Row=Math.Clamp(selectedActive.Row,1,1024),Column=Math.Clamp(selectedActive.Column,1,1024)};
        cells.Clear();styles.Clear();InvalidateVisual();
    }
    public Cell? SelectedCell => cells.GetValueOrDefault((Selected.Row, Selected.Column));
    public WorksheetView() {
        MinHeight = 260; MaxHeight = (Rows + 1) * HeightPerCell; MinWidth = Header + Columns * WidthPerCell; Focusable = true; ClipToBounds = true;
        dragTimer.Tick += (_, _) => UpdateDrag();
        DetachedFromVisualTree += (_, _) => EndPointerSelection(false);
        SizeChanged += (_, _) => {
            FirstRow = Math.Min(FirstRow, TotalVisualRows + 1 - (uint)VisibleRows);
            cells.Clear();styles.Clear(); ViewportChanged?.Invoke();
        };
    }
    protected override Size MeasureOverride(Size availableSize) => new(Header + Columns * WidthPerCell,
        Math.Min((Rows + 1) * HeightPerCell, double.IsFinite(availableSize.Height) ? availableSize.Height : (Rows + 1) * HeightPerCell));
    public void SetCells(IEnumerable<Cell> values)
    {
        paint?.TrySetCanceled(); paint = new(TaskCreationOptions.RunContinuationsAsynchronously);
        paintStart = Stopwatch.GetTimestamp(); awaitingPaint = true;
        cells.Clear();styles.Clear(); foreach (var cell in values) cells[(cell.At.Row, cell.At.Column)] = cell;
        InvalidateVisual();
    }
    public void Scroll(int rows, int columns = 0)
    {
        uint r = (uint)Math.Clamp((int)FirstRow + rows, 1, (int)TotalVisualRows + 1 - VisibleRows);
        uint c = (uint)Math.Clamp((int)FirstColumn + columns, 1, 1025 - Columns);
        if (r == FirstRow && c == FirstColumn) return;
        FirstRow = r; FirstColumn = c; cells.Clear();styles.Clear(); InvalidateVisual(); ViewportChanged?.Invoke();
    }
    protected override void OnPointerWheelChanged(PointerWheelEventArgs e)
    {
        base.OnPointerWheelChanged(e);
        if (e.KeyModifiers.HasFlag(KeyModifiers.Shift)) Scroll(0, e.Delta.Y > 0 ? -3 : 3);
        else Scroll(e.Delta.Y > 0 ? -3 : 3);
        e.Handled = true;
    }
    private Position? PositionAt(Point point) {
        int index=(int)Math.Floor(point.Y/HeightPerCell)-1;
        uint row;
        if(ProjectionFrame is not null){var mapped=MappedRow(index);if(mapped is null || mapped.Kind!=0)return null;row=mapped.OrdinaryRow;}
        else row=(uint)Math.Clamp((int)FirstRow+index,1,1024);
        return new Position{Row=row,Column=(uint)Math.Clamp((int)FirstColumn+(int)Math.Floor((point.X-Header)/WidthPerCell),1,1024)};
    }
    protected override async void OnPointerPressed(PointerPressedEventArgs e)
    {
        base.OnPointerPressed(e);
        var point=e.GetPosition(this);
        if (!e.GetCurrentPoint(this).Properties.IsLeftButtonPressed || point.X<Header || point.X>=Header+Columns*WidthPerCell
            || point.Y<HeightPerCell || point.Y>=(VisibleRows+1)*HeightPerCell) return;
        if(ProjectionTargetAt(point) is { } projected){EndPointerSelection(false);e.Handled=true;if(ProjectionRequested is not null)await ProjectionRequested(projected);return;}
        var ordinary=PositionAt(point);if(ordinary is null)return;
        EndPointerSelection(false);
        previousAnchor=Anchor; previousActive=Selected;
        long current=++gesture;
        dragPointer=e.Pointer; dragPoint=point; dragReady=false;
        e.Pointer.Capture(this); Focus(); e.Handled=true;
        bool extend=e.KeyModifiers.HasFlag(KeyModifiers.Shift);
        bool doubleClick=e.ClickCount==2 && !extend;
        selectionPending=true;
        bool accepted=SelectionRequested is not null && await SelectionRequested(ordinary,extend,doubleClick,()=>gesture==current);
        if(gesture==current) selectionPending=false;
        if (gesture!=current || dragPointer!=e.Pointer) return;
        if (!accepted || doubleClick) { EndPointerSelection(false); return; }
        dragReady=true; UpdateDrag(); dragTimer.Start();
    }
    protected override void OnPointerMoved(PointerEventArgs e)
    {
        base.OnPointerMoved(e);
        if (dragPointer!=e.Pointer) return;
        dragPoint=e.GetPosition(this); if(dragReady) UpdateDrag(); e.Handled=true;
    }
    protected override void OnPointerReleased(PointerReleasedEventArgs e)
    {
        base.OnPointerReleased(e);
        if (dragPointer!=e.Pointer || e.InitialPressMouseButton!=MouseButton.Left) return;
        dragPoint=e.GetPosition(this); if(dragReady) UpdateDrag();
        EndPointerSelection(false,true); e.Handled=true;
    }
    protected override void OnPointerCaptureLost(PointerCaptureLostEventArgs e)
    {
        base.OnPointerCaptureLost(e); if(dragPointer is not null) EndPointerSelection(false);
    }
    private void UpdateDrag()
    {
        if (!dragReady || dragPointer is null) return;
        int rows=dragPoint.Y<HeightPerCell ? -1 : dragPoint.Y>=(VisibleRows+1)*HeightPerCell ? 1 : 0;
        int columns=dragPoint.X<Header ? -1 : dragPoint.X>=Header+Columns*WidthPerCell ? 1 : 0;
        Scroll(rows,columns);
        // Beyond-edge distance must not create a large coordinate jump per tick.
        var clamped=new Point(Math.Clamp(dragPoint.X,Header,Header+Columns*WidthPerCell-1),
            Math.Clamp(dragPoint.Y,HeightPerCell,(VisibleRows+1)*HeightPerCell-1));
        if(PositionAt(clamped) is { } at) Select(at,true);
    }
    public void EndPointerSelection(bool cancel, bool allowPending=false)
    {
        var pointer=dragPointer; bool restore=cancel && (pointer is not null || selectionPending);
        dragPointer=null; dragReady=false; dragTimer.Stop();
        if(!allowPending) { ++gesture; selectionPending=false; }
        pointer?.Capture(null);
        if(restore) { anchor=previousAnchor.Clone(); active=previousActive.Clone(); InvalidateVisual(); SelectionChanged?.Invoke(); Reveal(active); }
    }
    public void Select(Position at, bool extend=false)
    {
        if(at.Row is <1 or >1024 || at.Column is <1 or >1024) throw new ArgumentOutOfRangeException(nameof(at));
        if(ProjectedSelection is not null){ProjectedSelection=null;MissingProjectedRecord=false;ProjectedSelectionChanged?.Invoke();}
        if(!extend) anchor=at.Clone();
        if(active.Row==at.Row && active.Column==at.Column && extend) return;
        active=at.Clone(); InvalidateVisual(); SelectionChanged?.Invoke();
    }
    public Rect CellBounds(Position at) => new(Header + ((int)at.Column - FirstColumn) * WidthPerCell,
        (1 + OrdinaryIndex(at.Row)) * HeightPerCell, WidthPerCell, HeightPerCell);
    public bool IsVisibleCell(Position at) => OrdinaryIndex(at.Row)>=0 && OrdinaryIndex(at.Row)<VisibleRows
        && at.Column >= FirstColumn && at.Column < FirstColumn + Columns;
    public void Reveal(Position at) => Scroll(
        VisualRow(at.Row) < FirstRow ? (int)VisualRow(at.Row) - (int)FirstRow : VisualRow(at.Row) >= FirstRow + VisibleRows ? (int)VisualRow(at.Row) - (int)FirstRow - VisibleRows + 1 : 0,
        at.Column < FirstColumn ? (int)at.Column - (int)FirstColumn : at.Column >= FirstColumn + Columns ? (int)at.Column - (int)FirstColumn - Columns + 1 : 0);
    public override void Render(DrawingContext context)
    {
        base.Render(context);
        var border = new Pen(Brushes.LightGray, 1);
        context.FillRectangle(Brushes.White, new Rect(Bounds.Size));
        var first=SelectionFirst; var last=SelectionLast;
        for (int row = -1; row < VisibleRows; row++) for (int col = -1; col < Columns; col++) {
            double x = col < 0 ? 0 : Header + col * WidthPerCell;
            double y = (row + 1) * HeightPerCell;
            var rect = new Rect(x, y, col < 0 ? Header : WidthPerCell, HeightPerCell);
            uint r = FirstRow + (uint)Math.Max(row, 0), c = FirstColumn + (uint)Math.Max(col, 0);
            if(row>=0 && RenderProjection(context,row,col,rect))continue;
            if(row>=0 && ProjectionFrame is not null)r=MappedRow(row)!.OrdinaryRow;
            bool selected = row >= 0 && col >= 0 && r == active.Row && c == active.Column;
            bool region = row >= 0 && col >= 0 && r>=first.Row && r<=last.Row && c>=first.Column && c<=last.Column;
            var style=row>=0&&col>=0 ? StyleAt(new Position{Row=r,Column=c}) : new CellStyle();
            IBrush fill=row<0 || col<0 ? Brushes.WhiteSmoke : selected ? Brushes.White : region ? ActiveFocus ? Brushes.AliceBlue : Brushes.WhiteSmoke : Brushes.White;
            if(style.Background is uint bg)fill=new SolidColorBrush(Color.FromUInt32(0xff000000|bg));
            context.DrawRectangle(fill, selected ? new Pen(ActiveFocus ? Brushes.DodgerBlue : Brushes.SlateGray,2) : border,rect);
            string text = ""; IBrush ink = Brushes.Black;
            if (row < 0 && col >= 0) text = Address(new Position { Row = 1, Column = c }).TrimEnd('1');
            else if (row >= 0 && col < 0) text = r.ToString(CultureInfo.InvariantCulture);
            else if (row >= 0 && col >= 0 && cells.TryGetValue((r, c), out var cell)) {
                if (cell.Error.Length > 0) { text = "#ERROR"; ink = Brushes.Firebrick; }
                else if (cell.ContentCase == Cell.ContentOneofCase.Text) text = cell.Text;
                else if (cell.ContentCase != Cell.ContentOneofCase.Blank) text = cell.FormattedNumber.Length>0 ? cell.FormattedNumber : WorkbookSession.DisplayValue(cell);
            }
            if(style.Foreground is uint fg)ink=new SolidColorBrush(Color.FromUInt32(0xff000000|fg));
            var typeface=new Typeface("Inter",style.Italic?FontStyle.Italic:FontStyle.Normal,style.Bold?FontWeight.Bold:FontWeight.Normal);
            var formatted=new FormattedText(text,CultureInfo.InvariantCulture,FlowDirection.LeftToRight,typeface,13,ink);
            int align=style.Alignment;
            if(align==0 && row>=0 && col>=0 && cells.TryGetValue((r,c),out var content))
                align=content.Error.Length>0?1:content.Result?.KindCase==TypedValue.KindOneofCase.Decimal?3:content.Result?.KindCase==TypedValue.KindOneofCase.Boolean?2:1;
            double left=align==3?Math.Max(x+5,x+rect.Width-5-formatted.Width):align==2?Math.Max(x+5,x+(rect.Width-formatted.Width)/2):x+5;
            using (context.PushClip(rect.Deflate(3))) context.DrawText(formatted,new Point(left,y+4));
        }
        if(HasRegion && ProjectionFrame is null) {
            var begin=CellBounds(first); var end=CellBounds(last);
            using(context.PushClip(new Rect(Header,HeightPerCell,Columns*WidthPerCell,VisibleRows*HeightPerCell)))
                context.DrawRectangle(null,new Pen(ActiveFocus ? Brushes.DodgerBlue : Brushes.SlateGray,1),new Rect(begin.TopLeft,end.BottomRight));
        }
        if (awaitingPaint) { awaitingPaint = false; paint?.TrySetResult(Stopwatch.GetElapsedTime(paintStart)); }
    }
    public static string Address(Position p)
    {
        uint c = p.Column; string name = "";
        while (c > 0) { c--; name = (char)('A' + c % 26) + name; c /= 26; }
        return name + p.Row.ToString(CultureInfo.InvariantCulture);
    }
    public static Position ParseAddress(string text)
    {
        int i = 0; uint column = 0;
        text = text.Trim().ToUpperInvariant();
        while (i < text.Length && text[i] is >= 'A' and <= 'Z') {
            column = checked(column * 26 + (uint)(text[i++] - 'A' + 1));
            if (column > 1024) throw new FormatException("Column must be between A and AMJ.");
        }
        if (column == 0 || !uint.TryParse(text.AsSpan(i), NumberStyles.None, CultureInfo.InvariantCulture, out uint row) || row is < 1 or > 1024)
            throw new FormatException("Enter a worksheet address such as B22 (rows and columns 1–1024).");
        return new Position { Row = row, Column = column };
    }
}
