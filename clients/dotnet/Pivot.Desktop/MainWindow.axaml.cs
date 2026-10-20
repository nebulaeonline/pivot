using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
using System;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using System.Text.Json;
using Avalonia;
using Avalonia.Media.Imaging;

namespace Pivot.Desktop;

public partial class MainWindow : Window
{
    private WorkbookSession? session;
    private CancellationTokenSource? refresh;
    private long displayRequest;
    private bool shuttingDown, allowClose, editing;
    private double lastUiApply, lastPaintDelay, lastVisibleUpdate;
    private string displayedRevision = "";
    private string displayedGeneration = "";
    public MainWindow()
    {
        InitializeComponent();
        TimingLabel.IsVisible = Environment.GetEnvironmentVariable("PIVOT_DIAGNOSTICS") == "1" || Environment.GetEnvironmentVariable("PIVOT_UI_SMOKE_REPORT") is not null;
        SetEditing(false);
        Sheet.SelectionChanged += SelectCell;
        Sheet.ViewportChanged += () => _ = RefreshAsync();
        Opened += async (_, _) => await StartAsync();
        Closing += async (_, e) => {
            if (allowClose) return;
            e.Cancel = true; if (shuttingDown) return;
            shuttingDown = true; refresh?.Cancel(); SetEditing(false);
            if (session is not null) await session.DisposeAsync();
            allowClose = true; Close();
        };
    }
    private async Task StartAsync()
    {
        try {
            string path = Environment.GetEnvironmentVariable("PIVOT_SESSION_PATH") ?? Path.Combine(AppContext.BaseDirectory, OperatingSystem.IsWindows() ? "pivot-session.exe" : "pivot-session");
            if (!File.Exists(path)) throw new FileNotFoundException("Rust session executable missing. Start with scripts/run_desktop.py or set PIVOT_SESSION_PATH.");
            session = await WorkbookSession.StartAsync(path);
            if (shuttingDown) { await session.DisposeAsync(); return; }
            await RefreshAsync(); SetEditing(displayedRevision.Length != 0);
            if (Environment.GetEnvironmentVariable("PIVOT_UI_SMOKE_REPORT") is { } report) await SmokeAsync(report);
        } catch (Exception e) {
            StatusLabel.Text = e.Message;
            if (Environment.GetEnvironmentVariable("PIVOT_UI_SMOKE_REPORT") is { } report) {
                await File.WriteAllTextAsync(report, JsonSerializer.Serialize(new { status = "failed", error = e.ToString() })); Close();
            }
        }
    }
    private void SetEditing(bool enabled) { CommitButton.IsEnabled = ClearButton.IsEnabled = MoveButton.IsEnabled = FormulaEntry.IsEnabled = enabled && !shuttingDown; }
    private void SelectCell()
    {
        AddressLabel.Text = Pivot.Avalonia.WorksheetView.Address(Sheet.Selected);
        var cell = Sheet.SelectedCell;
        FormulaEntry.Text = cell?.ContentCase switch {
            Cell.ContentOneofCase.Formula => cell.Formula,
            Cell.ContentOneofCase.Number => WorkbookSession.Decode(cell.Number).ToString(CultureInfo.InvariantCulture),
            _ => ""
        };
        if (cell?.Error.Length > 0) StatusLabel.Text = cell.Error;
    }
    private async Task RefreshAsync()
    {
        if (session is null || shuttingDown) return;
        refresh?.Cancel(); refresh?.Dispose(); refresh = new CancellationTokenSource();
        var cancellation = refresh.Token; long request = ++displayRequest;
        var visible = Stopwatch.StartNew();
        try {
            await Task.Delay(35, cancellation); // Coalesce scrolling before queuing Rust work.
            var frame = await session.ReadAsync(new Position { Row = Sheet.FirstRow, Column = Sheet.FirstColumn },
                new Position { Row = Math.Min(1024, Sheet.FirstRow + (uint)Sheet.VisibleRows - 1), Column = Sheet.FirstColumn + 9 }, cancellation);
            if (shuttingDown || request != displayRequest || frame.Revision != session.Revision) return;
            var dispatch = Stopwatch.StartNew();
            Sheet.SetCells(frame.Cells); displayedRevision = frame.Revision; displayedGeneration = frame.Generation;
            StatusLabel.Text = $"Ready · {frame.Cells.Count} populated cells · {frame.Cells.Count(c => c.Error.Length > 0)} formula errors";
            SelectCell();
            lastUiApply = dispatch.Elapsed.TotalMilliseconds;
            TimingLabel.Text = $"Viewport round trip {session.LastRoundTrip.TotalMilliseconds:F2} ms · Rust {session.LastRustExecutionNs / 1e6:F2} ms · encode C# {session.LastClientSerialization.TotalMilliseconds:F2} / Rust {session.LastRustSerializationNs / 1e6:F2} ms · UI apply {dispatch.Elapsed.TotalMilliseconds:F2} ms (paint measured separately)";
            var painted = await Sheet.PaintCompletion.WaitAsync(TimeSpan.FromSeconds(5), cancellation);
            if (request == displayRequest && !shuttingDown) { lastPaintDelay = painted.TotalMilliseconds; lastVisibleUpdate = visible.Elapsed.TotalMilliseconds; }
        } catch (OperationCanceledException) { }
          catch (Exception e) { if (request == displayRequest && !shuttingDown) StatusLabel.Text = e.Message; }
    }
    private async Task EditAsync(Edit edit)
    {
        if (session is null || editing || shuttingDown) return;
        editing = true; SetEditing(false); refresh?.Cancel(); ++displayRequest;
        try { await session.ApplyAsync(displayedRevision, [edit]); await RefreshAsync(); }
        catch (WorkbookSessionException e) when (e.IsConflict) { await RefreshAsync(); StatusLabel.Text = "Worksheet changed; view refreshed. Review the cell before retrying."; }
        catch (Exception e) { StatusLabel.Text = e.Message; }
        finally { editing = false; SetEditing(true); }
    }
    private async void CommitClick(object? sender, RoutedEventArgs e)
        => await CommitAsync();
    private async Task CommitAsync()
    {
        string source = FormulaEntry.Text ?? "";
        string text = source.Trim();
        if (text.Length == 0) { await EditAsync(new Edit { At = Sheet.Selected.Clone(), Clear = true }); return; }
        if (text.StartsWith('=')) { await EditAsync(new Edit { At = Sheet.Selected.Clone(), Formula = source }); return; }
        int fractionalDigits = text.IndexOf('.') is var dot && dot >= 0 ? text.Length - dot - 1 : 0;
        if (decimal.TryParse(text, NumberStyles.AllowLeadingSign | NumberStyles.AllowDecimalPoint, CultureInfo.InvariantCulture, out decimal value)
            && ((decimal.GetBits(value)[3] >> 16) & 255) == fractionalDigits)
            await EditAsync(new Edit { At = Sheet.Selected.Clone(), Number = WorkbookSession.Encode(value) });
        else StatusLabel.Text = "Enter a representable exact decimal (scale ≤28) or a formula beginning with =.";
    }
    private async void ClearClick(object? sender, RoutedEventArgs e) => await EditAsync(new Edit { At = Sheet.Selected.Clone(), Clear = true });
    private async void MoveClick(object? sender, RoutedEventArgs e)
    {
        try { await EditAsync(new Edit { At = Sheet.Selected.Clone(), MoveTo = Pivot.Avalonia.WorksheetView.ParseAddress(MoveTarget.Text ?? "") }); }
        catch (Exception ex) { StatusLabel.Text = ex.Message; }
    }
    private void EntryKeyDown(object? sender, KeyEventArgs e) { if (e.Key == Key.Enter) { CommitClick(sender, new RoutedEventArgs()); e.Handled = true; } }
    private void UpClick(object? sender, RoutedEventArgs e) => Sheet.Scroll(-Sheet.VisibleRows);
    private void DownClick(object? sender, RoutedEventArgs e) => Sheet.Scroll(Sheet.VisibleRows);
    private void LeftClick(object? sender, RoutedEventArgs e) => Sheet.Scroll(0, -10);
    private void RightClick(object? sender, RoutedEventArgs e) => Sheet.Scroll(0, 10);

    private async Task SmokeAsync(string report)
    {
        void Check(bool condition, string message) { if (!condition) throw new InvalidOperationException(message); }
        Position P(uint r, uint c) => new() { Row = r, Column = c };
        var timings = new System.Collections.Generic.List<object>();
        async Task Enter(uint r, uint c, string value) {
            Sheet.Select(P(r,c)); FormulaEntry.Text = value;
            var clock = Stopwatch.StartNew(); await CommitAsync();
            Check(displayedRevision == session!.Revision, "Displayed frame must match current exact revision.");
            Check(displayedGeneration.Length > 0, "Displayed frame must identify its calculation generation.");
            timings.Add(new { edit_to_render_ms = clock.Elapsed.TotalMilliseconds, ui_apply_ms = lastUiApply, paint_delay_ms = lastPaintDelay, refresh_to_render_ms = lastVisibleUpdate });
        }
        decimal Value(uint r,uint c) => WorkbookSession.Decode(Sheet.CellAt(P(r,c))!.Value);
        await Enter(1,1,"1.2300"); string stable = Sheet.CellAt(P(1,1))!.CellId;
        Check(Sheet.CellAt(P(1,1))!.Number.Scale == 4, "UI input scale");
        string beforeInvalidNumber = session!.Revision;
        Sheet.Select(P(5,5)); FormulaEntry.Text = "0.1234567890123456789012345678901"; await CommitAsync();
        Check(session.Revision == beforeInvalidNumber && Sheet.CellAt(P(5,5)) is null, "UI must not silently round an unrepresentable input.");
        await Enter(1,2,"2"); await Enter(1,3,"=A1+B1"); Check(Value(1,3)==3.23m,"UI formula");
        await Enter(1,1,"5"); Check(Value(1,3)==7m,"UI dependent edit");
        await Enter(1,4,"=("); Check(Sheet.CellAt(P(1,4))!.Error.Length>0,"UI editable draft");
        await Enter(1,4,"=C1*2"); Check(Value(1,4)==14m,"UI repair");
        await EditAsync(new Edit { At=P(1,1), MoveTo=P(2,1) });
        Check(Sheet.CellAt(P(2,1))!.CellId==stable,"UI move stable identity");
        await Enter(1,1,"100"); Check(Value(1,3)==7m,"UI coordinate reuse must not rebind");
        await Enter(1,3,"=A1+B1"); Check(Value(1,3)==102m,"UI explicit rebind");
        await EditAsync(new Edit { At=P(1,2), Clear=true }); Check(Value(1,3)==100m,"UI clear");
        // The editor must submit the displayed revision, even if a background
        // operation has advanced the session before the new frame is installed.
        await session!.ApplyAsync(session.Revision, [new Edit { At=P(2,2), Number=WorkbookSession.Encode(4m) }]);
        Sheet.Select(P(1,1)); FormulaEntry.Text="999"; await CommitAsync();
        Check(Value(1,1)==100m && Value(2,2)==4m,"UI stale edit must conflict and refresh without overwriting.");
        // Superseded viewport loads must never install after a newer request.
        var superseded = RefreshAsync(); Sheet.Scroll(20); await RefreshAsync(); await superseded;
        Check(Sheet.FirstRow==21 && Sheet.CellAt(P(1,3)) is null,"UI scroll bounded frame");
        Sheet.Scroll(-20); await RefreshAsync(); Check(Value(1,3)==100m,"UI scroll back");
        Sheet.Scroll(5000,5000); await RefreshAsync();
        Check(Sheet.FirstRow+(uint)Sheet.VisibleRows-1==1024 && Sheet.FirstColumn+9==1024,"UI scroll respects worksheet bounds");
        Sheet.Scroll(-5000,-5000); await RefreshAsync(); Check(Value(1,3)==100m,"UI return from worksheet edge");
        using (var image = new RenderTargetBitmap(new PixelSize((int)Bounds.Width, (int)Bounds.Height))) {
            image.Render(this); image.Save(Path.ChangeExtension(report,".png"), PngBitmapEncoderOptions.Default);
        }
        await session!.DisposeAsync(); Check(session.FinalCharge==0,"UI session cleanup");
        await File.WriteAllTextAsync(report, JsonSerializer.Serialize(new { status="passed", backend="real Rust", viewport_rows=Sheet.VisibleRows, viewport_columns=10, maximum_viewport_cells=200, remaining_charge=session.FinalCharge, timings, note="Rendered-frame timing includes coalescing and UI scheduling, not physical display presentation." }, new JsonSerializerOptions { WriteIndented=true }));
        Close();
    }
}
