using Avalonia.Controls;
using Avalonia.Platform.Storage;
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
            shuttingDown = true;
            if (session is not null && !await ConfirmDiscardAsync()) { shuttingDown = false; return; }
            refresh?.Cancel(); SetEditing(false);
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
                await File.WriteAllTextAsync(report, JsonSerializer.Serialize(new { status = "failed", error = e.ToString() })); if(session is not null) await session.DisposeAsync(); allowClose=true; Close();
            }
        }
    }
    private void UpdateDocument()
    {
        if (session is null) return;
        string name = session.DocumentId.Length == 0 ? "No workbook open" : session.DocumentPath.Length == 0 ? "Untitled" : Path.GetFileName(session.DocumentPath);
        Title = $"{name}{(session.IsDirty ? " *" : "")} — Pivot";
        DocumentLabel.Text = $"{name} · {(session.IsDirty ? "Unsaved changes" : "Saved")}{(session.InstallationUncertain ? " · Installation uncertain: reopen to verify" : "")}";
    }
    private async Task<bool> ConfirmDiscardAsync()
    {
        if (session is null || !session.IsDirty) return true;
        var dialog = new Window { Title="Unsaved changes", Width=430, Height=170, CanResize=false, WindowStartupLocation=WindowStartupLocation.CenterOwner };
        var panel = new StackPanel { Margin=new Thickness(16), Spacing=14 };
        panel.Children.Add(new TextBlock { Text="Save changes before continuing?", TextWrapping=global::Avalonia.Media.TextWrapping.Wrap });
        var buttons = new StackPanel { Orientation=global::Avalonia.Layout.Orientation.Horizontal, Spacing=12 };
        foreach (var choice in new[] { "Save", "Discard", "Cancel" }) { var button=new Button { Content=choice }; button.Click += (_,_)=>dialog.Close(choice); buttons.Children.Add(button); }
        panel.Children.Add(buttons); dialog.Content=panel;
        var result=await dialog.ShowDialog<string?>(this);
        if (result=="Save") return await SaveDocumentAsync(false);
        return result=="Discard";
    }
    private async Task<bool> SaveDocumentAsync(bool saveAs, string? chosenPath = null)
    {
        if (session is null || session.DocumentId.Length==0) return false;
        try {
            string? path=chosenPath;
            if (path is null && (saveAs || session.DocumentPath.Length==0)) {
                var file=await StorageProvider.SaveFilePickerAsync(new FilePickerSaveOptions { Title="Save Pivot workbook", SuggestedFileName="Workbook.pivot", DefaultExtension="pivot", ShowOverwritePrompt=true });
                path=file?.TryGetLocalPath(); if (path is null) return false;
            }
            await session.SaveAsync(session.Revision,path); UpdateDocument();
            StatusLabel.Text=session.IsDirty ? "Saved an earlier generation; newer changes remain unsaved." : "Workbook saved.";
            return !session.IsDirty;
        } catch (Exception e) { UpdateDocument(); StatusLabel.Text=e.Message; return false; }
    }
    private bool fileAction;
    private async Task FileActionAsync(Func<Task> action)
    {
        if (fileAction || editing || shuttingDown || session is null) return;
        fileAction=true;
        try { await action(); }
        catch (Exception e) { StatusLabel.Text=e.Message; }
        finally { fileAction=false; UpdateDocument(); SetEditing(session.DocumentId.Length!=0); }
    }
    private async void SaveClick(object? sender,RoutedEventArgs e) => await FileActionAsync(async()=>{await SaveDocumentAsync(false);});
    private async void SaveAsClick(object? sender,RoutedEventArgs e) => await FileActionAsync(async()=>{await SaveDocumentAsync(true);});
    private async void NewClick(object? sender,RoutedEventArgs e) => await FileActionAsync(async()=>{
        if (!await ConfirmDiscardAsync()) return;
        refresh?.Cancel(); ++displayRequest; await session!.NewAsync(session.Revision,discard:true); await RefreshAsync();
    });
    private async void OpenClick(object? sender,RoutedEventArgs e) => await FileActionAsync(async()=>{
        var files=await StorageProvider.OpenFilePickerAsync(new FilePickerOpenOptions { Title="Open Pivot workbook", AllowMultiple=false });
        var path=files.Count==1 ? files[0].TryGetLocalPath() : null; if(path is null || !await ConfirmDiscardAsync()) return;
        refresh?.Cancel(); ++displayRequest; await OpenDocumentAsync(path);
    });
    private async Task OpenDocumentAsync(string path) { await session!.OpenAsync(session.Revision,path,discard:true); await RefreshAsync(); }
    private async void CloseDocumentClick(object? sender,RoutedEventArgs e) => await FileActionAsync(async()=>{
        if (!await ConfirmDiscardAsync()) return;
        refresh?.Cancel(); ++displayRequest; await session!.CloseDocumentAsync(session.Revision,discard:true);
        displayedRevision="";displayedGeneration="";Sheet.SetCells([]);FormulaEntry.Text="";StatusLabel.Text="Workbook closed. Choose New or Open.";
    });
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
            SelectCell(); UpdateDocument();
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
        string testRoot=Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT") ?? Path.GetDirectoryName(report)!;
        Directory.CreateDirectory(testRoot);
        string savedPath=Path.Combine(testRoot,"ui-"+Guid.NewGuid()+".pivot");
        Check(await SaveDocumentAsync(true,savedPath),"UI Save As clean checkpoint");
        string savedDocument=session!.DocumentId;
        await session.NewAsync(session.Revision);await RefreshAsync();Check(Sheet.CellAt(P(1,3)) is null,"UI New");
        await OpenDocumentAsync(savedPath);Check(Value(1,3)==100m && session.DocumentId==savedDocument && !session.IsDirty,"UI Open saved workbook");
        await Enter(2,1,"6");Check(session.IsDirty,"Reopened editing marks dirty");
        Check(await SaveDocumentAsync(false),"UI Save existing workbook: "+StatusLabel.Text);
        File.Copy(savedPath,Path.ChangeExtension(report,".pivot"),overwrite:true);
        using (var image = new RenderTargetBitmap(new PixelSize((int)Bounds.Width, (int)Bounds.Height))) {
            image.Render(this); image.Save(Path.ChangeExtension(report,".png"), PngBitmapEncoderOptions.Default);
        }
        await session!.DisposeAsync(); Check(session.FinalCharge==0,"UI session cleanup");
        await File.WriteAllTextAsync(report, JsonSerializer.Serialize(new { status="passed", backend="real Rust", viewport_rows=Sheet.VisibleRows, viewport_columns=10, maximum_viewport_cells=200, remaining_charge=session.FinalCharge, timings, note="Rendered-frame timing includes coalescing and UI scheduling, not physical display presentation." }, new JsonSerializerOptions { WriteIndented=true }));
        allowClose = true; Close();
    }
}
