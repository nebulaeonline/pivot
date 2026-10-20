using System.Globalization;
using System.Diagnostics;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Media;
using Pivot.Client;
using Pivot.Protocol.Application.V1;

namespace Pivot.Avalonia;

/// <summary>Paints only 20 x 10 cells. Scrolling changes the Rust viewport;
/// no worksheet-sized control collection or client calculation model.</summary>
public sealed class WorksheetView : Control
{
    public const int Rows = 20, Columns = 10;
    private const double Header = 48, WidthPerCell = 94, HeightPerCell = 26;
    private readonly Dictionary<(uint, uint), Cell> cells = [];
    private TaskCompletionSource<TimeSpan>? paint;
    private long paintStart;
    private bool awaitingPaint;
    public Task<TimeSpan> PaintCompletion => paint?.Task ?? Task.FromResult(TimeSpan.Zero);
    public Cell? CellAt(Position at) => cells.GetValueOrDefault((at.Row, at.Column));
    public uint FirstRow { get; private set; } = 1;
    public uint FirstColumn { get; private set; } = 1;
    public int VisibleRows => Math.Clamp((int)(Bounds.Height / HeightPerCell) - 1, 1, Rows);
    public Position Selected { get; private set; } = new() { Row = 1, Column = 1 };
    public event Action? ViewportChanged;
    public event Action? SelectionChanged;
    public Cell? SelectedCell => cells.GetValueOrDefault((Selected.Row, Selected.Column));
    public WorksheetView() {
        MinHeight = 260; MaxHeight = (Rows + 1) * HeightPerCell; MinWidth = Header + Columns * WidthPerCell; Focusable = true; ClipToBounds = true;
        SizeChanged += (_, _) => {
            FirstRow = Math.Min(FirstRow, (uint)(1025 - VisibleRows));
            cells.Clear(); ViewportChanged?.Invoke();
        };
    }
    protected override Size MeasureOverride(Size availableSize) => new(Header + Columns * WidthPerCell,
        Math.Min((Rows + 1) * HeightPerCell, double.IsFinite(availableSize.Height) ? availableSize.Height : (Rows + 1) * HeightPerCell));
    public void SetCells(IEnumerable<Cell> values)
    {
        paint?.TrySetCanceled(); paint = new(TaskCreationOptions.RunContinuationsAsynchronously);
        paintStart = Stopwatch.GetTimestamp(); awaitingPaint = true;
        cells.Clear(); foreach (var cell in values) cells[(cell.At.Row, cell.At.Column)] = cell;
        InvalidateVisual();
    }
    public void Scroll(int rows, int columns = 0)
    {
        uint r = (uint)Math.Clamp((int)FirstRow + rows, 1, 1025 - VisibleRows);
        uint c = (uint)Math.Clamp((int)FirstColumn + columns, 1, 1025 - Columns);
        if (r == FirstRow && c == FirstColumn) return;
        FirstRow = r; FirstColumn = c; cells.Clear(); InvalidateVisual(); ViewportChanged?.Invoke();
    }
    protected override void OnPointerWheelChanged(PointerWheelEventArgs e)
    {
        base.OnPointerWheelChanged(e);
        if (e.KeyModifiers.HasFlag(KeyModifiers.Shift)) Scroll(0, e.Delta.Y > 0 ? -3 : 3);
        else Scroll(e.Delta.Y > 0 ? -3 : 3);
        e.Handled = true;
    }
    protected override void OnPointerPressed(PointerPressedEventArgs e)
    {
        base.OnPointerPressed(e); var p = e.GetPosition(this);
        int c = (int)Math.Floor((p.X - Header) / WidthPerCell), r = (int)Math.Floor(p.Y / HeightPerCell) - 1;
        if (c is < 0 or >= Columns || r < 0 || r >= VisibleRows) return;
        Select(new Position { Row = FirstRow + (uint)r, Column = FirstColumn + (uint)c }); e.Handled = true;
    }
    public void Select(Position at) { Selected = at.Clone(); InvalidateVisual(); SelectionChanged?.Invoke(); }
    public override void Render(DrawingContext context)
    {
        base.Render(context);
        var border = new Pen(Brushes.LightGray, 1);
        context.FillRectangle(Brushes.White, new Rect(Bounds.Size));
        for (int row = -1; row < VisibleRows; row++) for (int col = -1; col < Columns; col++) {
            double x = col < 0 ? 0 : Header + col * WidthPerCell;
            double y = (row + 1) * HeightPerCell;
            var rect = new Rect(x, y, col < 0 ? Header : WidthPerCell, HeightPerCell);
            uint r = FirstRow + (uint)Math.Max(row, 0), c = FirstColumn + (uint)Math.Max(col, 0);
            bool selected = row >= 0 && col >= 0 && r == Selected.Row && c == Selected.Column;
            context.DrawRectangle(row < 0 || col < 0 ? Brushes.WhiteSmoke : selected ? Brushes.AliceBlue : Brushes.White, selected ? new Pen(Brushes.DodgerBlue, 2) : border, rect);
            string text = ""; IBrush ink = Brushes.Black;
            if (row < 0 && col >= 0) text = Address(new Position { Row = 1, Column = c }).TrimEnd('1');
            else if (row >= 0 && col < 0) text = r.ToString(CultureInfo.InvariantCulture);
            else if (row >= 0 && col >= 0 && cells.TryGetValue((r, c), out var cell)) {
                if (cell.Error.Length > 0) { text = "#ERROR"; ink = Brushes.Firebrick; }
                else if (cell.Value is not null && cell.ContentCase != Cell.ContentOneofCase.Blank) text = WorkbookSession.Decode(cell.Value).ToString(CultureInfo.InvariantCulture);
            }
            using (context.PushClip(rect.Deflate(3)))
                context.DrawText(new FormattedText(text, CultureInfo.InvariantCulture, FlowDirection.LeftToRight, new Typeface("Inter"), 13, ink), new Point(x + 5, y + 4));
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
