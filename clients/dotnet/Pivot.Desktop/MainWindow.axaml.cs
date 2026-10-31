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
using Avalonia.Media;
using Avalonia.Input.Platform;

namespace Pivot.Desktop;

public partial class MainWindow : Window
{
    private WorkbookSession? session;
    private CancellationTokenSource? refresh;
    private long displayRequest;
    private bool shuttingDown, allowClose, editing;
    private long clipboardRequest, interactionVersion;
    private KeyModifiers shortcutModifiers;
    private TextBox? clipboardEditor;
    private Position? draftCell;
    private string draftRevision = "", draftOriginal = "";
    private bool synchronizing, draftReplace, frameReady;
    private double lastUiApply, lastPaintDelay, lastVisibleUpdate;
    private string displayedRevision = "";
    private string displayedGeneration = "";
    public MainWindow()
    {
        InitializeComponent();
        InitializeFormatting();
        TimingLabel.IsVisible = Environment.GetEnvironmentVariable("PIVOT_DIAGNOSTICS") == "1" || Environment.GetEnvironmentVariable("PIVOT_UI_SMOKE_REPORT") is not null;
        SetEditing(false);
        InitializeProofInteractions();
        Sheet.FilterColumnRequested+=column=>_ = ConfigureFilterAsync(column);
        Sheet.SelectionChanged += () => { ++interactionVersion; SelectCell(); };
        Sheet.ViewportChanged += () => { ++interactionVersion; frameReady = false; PositionEditor(); _ = RefreshAsync(); };
        Sheet.SelectionRequested = async (at, extend, edit, current) => {
            bool accepted=await SelectAsync(at,extend,current);
            if(accepted && edit) BeginCellEdit(false);
            return accepted;
        };
        FormulaEntry.PropertyChanged += (_, e) => { if (e.Property == TextBox.TextProperty) FormulaChanged(); };
        CellEditor.PropertyChanged += (_, e) => { if (e.Property == TextBox.TextProperty) CellEditorChanged(); };
        AddHandler(KeyDownEvent, ShortcutKeyDown, RoutingStrategies.Tunnel);
        AddHandler(KeyUpEvent, (_, e) => shortcutModifiers=e.KeyModifiers, RoutingStrategies.Tunnel);
        AddHandler(TextInputEvent, (_, e) => { if (IsCommandInput()) e.Handled=true; }, RoutingStrategies.Tunnel);
        Sheet.GotFocus += (_, _) => { clipboardEditor=null; UpdateFocusAppearance(); };
        Sheet.LostFocus += (_, _) => UpdateFocusAppearance();
        foreach (var input in new[] { CellEditor, FormulaEntry }) {
            input.PropertyChanged += (_, e) => {
                if (!synchronizing && (e.Property==TextBox.SelectionStartProperty || e.Property==TextBox.SelectionEndProperty)) ++interactionVersion;
            };
            input.GotFocus += (_, _) => { clipboardEditor=input; UpdateFocusAppearance(); };
            input.LostFocus += (_, _) => UpdateFocusAppearance();
        }
        Activated += (_, _) => UpdateFocusAppearance();
        Deactivated += (_, _) => { Sheet.EndPointerSelection(false); shortcutModifiers=KeyModifiers.None; UpdateFocusAppearance(); };
        Sheet.KeyDown += GridKeyDown;
        Sheet.TextInput += GridTextInput;
        CellEditor.AddHandler(KeyDownEvent, EntryKeyDown, RoutingStrategies.Tunnel);
        Sheet.SizeChanged += (_, _) => PositionEditor();
        Opened += async (_, _) => await StartAsync();
        Closing += async (_, e) => {
            if (allowClose) return;
            e.Cancel = true; if (shuttingDown || editing || fileAction) return;
            ++interactionVersion; shuttingDown = true;
            if (!await CommitAsync() || (session is not null && !await ConfirmDiscardAsync())) { shuttingDown = false; SetEditing(true); await RefreshAsync(); return; }
            refresh?.Cancel(); SetEditing(false);
            if (session is not null) await session.DisposeAsync();
            allowClose = true; Close();
        };
    }
    private void UpdateFocusAppearance()
    {
        bool focused=IsActive && (Sheet.IsFocused || CellEditor.IsFocused);
        Sheet.ActiveFocus=focused; Sheet.InvalidateVisual();
        CellEditor.BorderBrush=CellEditor.IsFocused && IsActive ? Brushes.DodgerBlue : Brushes.SlateGray;
        CellEditor.BorderThickness=new Thickness(2);
    }
    private void CaretOptionChanged(object? sender, RoutedEventArgs e)
    {
        // Zero keeps the native caret steady; it does not draw a caret on a selected cell.
        var interval=BlinkCaret?.IsChecked==false ? TimeSpan.Zero : TimeSpan.FromMilliseconds(500);
        if (CellEditor is not null) CellEditor.CaretBlinkInterval=interval;
        if (FormulaEntry is not null) FormulaEntry.CaretBlinkInterval=interval;
    }
    private bool IsCommandInput() => shortcutModifiers.HasFlag(KeyModifiers.Meta)
        || (shortcutModifiers.HasFlag(KeyModifiers.Control) && !shortcutModifiers.HasFlag(KeyModifiers.Alt)); // AltGr remains text input.
    private async void ShortcutKeyDown(object? sender, KeyEventArgs e)
    {
        shortcutModifiers=e.KeyModifiers;
        var primary=OperatingSystem.IsMacOS() ? KeyModifiers.Meta : KeyModifiers.Control;
        if ((e.KeyModifiers & primary)==0 || e.KeyModifiers.HasFlag(KeyModifiers.Alt)) return;
        var focused=FocusManager?.GetFocusedElement();
        if (focused==Sheet && e.Key is Key.D or Key.R) {
            e.Handled=true;
            bool reverse=e.KeyModifiers.HasFlag(KeyModifiers.Shift);
            await FillAsync(e.Key==Key.D ? (reverse ? Request.Types.FillDirection.Up : Request.Types.FillDirection.Down) : (reverse ? Request.Types.FillDirection.Left : Request.Types.FillDirection.Right));
            return;
        }
        if (focused!=Sheet && focused!=CellEditor && focused!=FormulaEntry) return;
        if(e.Key==Key.X && focused==Sheet && session?.IsBindingProof==true){e.Handled=true;StatusLabel.Text="Cut is unavailable in this experimental document; use Duplicate for bound formulas.";return;}
        if (e.Key==Key.A && focused is TextBox box) { e.Handled=true; box.SelectAll(); return; }
        if (e.Key is Key.C or Key.V) {
            Sheet.EndPointerSelection(false); e.Handled=true; await ClipboardAsync(e.Key==Key.V);
        }
    }
    private async void CopyClick(object? sender,RoutedEventArgs e) => await ClipboardAsync(false);
    private async void PasteClick(object? sender,RoutedEventArgs e) => await ClipboardAsync(true);
    private static readonly DataFormat<byte[]> OldCellBlockFormat = DataFormat.CreateBytesPlatformFormat(OperatingSystem.IsMacOS() ? "org.pivot.cell-block-v1" : "application/x-pivot-cell-block-v1");
    private static readonly DataFormat<byte[]> OldCellBlock2Format = DataFormat.CreateBytesPlatformFormat(OperatingSystem.IsMacOS() ? "org.pivot.cell-block-v2" : "application/x-pivot-cell-block-v2");
    private static readonly DataFormat<byte[]> OldCellBlock3Format = DataFormat.CreateBytesPlatformFormat(OperatingSystem.IsMacOS() ? "org.pivot.cell-block-v3" : "application/x-pivot-cell-block-v3");
    private static readonly DataFormat<byte[]> OldCellBlock4Format = DataFormat.CreateBytesPlatformFormat(OperatingSystem.IsMacOS() ? "org.pivot.cell-block-v4" : "application/x-pivot-cell-block-v4");
    private static readonly DataFormat<byte[]> CellBlockFormat = DataFormat.CreateBytesPlatformFormat(OperatingSystem.IsMacOS() ? "org.pivot.cell-block-v5" : "application/x-pivot-cell-block-v5");
    private async Task ClipboardAsync(bool paste, Func<Task<string?>>? readText = null, Action<string>? diagnose = null, bool visible = false)
    {
        if (session is null || editing || fileAction || shuttingDown || Clipboard is null) {diagnose?.Invoke("blocked at entry");return;}
        if(Sheet.ProjectedSelection is not null){StatusLabel.Text="Projected clipboard operations are not supported in this proof.";return;}
        try {
            var input=clipboardEditor ?? (draftCell is null ? null : inlineEditing ? CellEditor : FormulaEntry);
            if (input==CellEditor && draftCell is null) input=null;
            if(visible && (input is not null || session.IsBindingProof || session.IsProjectionProof || session.FilterStatus is not (0 or 1)))throw new InvalidOperationException("Visible commands require a qualified ordinary worksheet selection.");
            if(input is null&&(!Sheet.HasVisibleRows||!FilterBulkReady)){StatusLabel.Text="Bulk clipboard actions need a qualified filter view. Wait, Reapply or Clear Filter.";return;}
            if (!paste) {
                if(input is not null) {
                    string selected=input.SelectedText ?? "";
                    if(selected.Length>0) await Clipboard.SetTextAsync(selected);
                } else {
                    long copyInteraction=interactionVersion;string copyRevision=session.Revision,copyMap=session.FilterGeneration;
                    var copied=visible?await session.CopyVisibleAsync(displayedRevision,Sheet.SelectionFirst,Sheet.SelectionLast):await session.CopyBlockAsync(displayedRevision,Sheet.SelectionFirst,Sheet.SelectionLast);
                    if(copyInteraction!=interactionVersion || editing || fileAction || shuttingDown || (visible&&(copied.Revision!=copyRevision||copied.FilterGeneration!=copyMap))) {diagnose?.Invoke("copy invalidated");return;}
                    var item=new DataTransferItem();item.Set(visible?VisibleBlockFormat:CellBlockFormat,copied.Clipboard.ToByteArray());if(copied.ClipboardTextAvailable)item.SetText(copied.ClipboardText);
                    var transfer=new DataTransfer();transfer.Add(item);await Clipboard.SetDataAsync(transfer);diagnose?.Invoke("native copy published");
                    if(visible)StatusLabel.Text=$"Copied {copied.VisibleRows.Count} visible rows × {copied.VisibleWidth} columns."+(copied.ClipboardTextAvailable?"":" Available for Pivot only: tabs or line breaks prevent plain-text table export.");
                }
                return;
            }
            long request=++clipboardRequest, interaction=interactionVersion;
            string before=input?.Text ?? "";
            int start=input is null ? 0 : Math.Min(input.SelectionStart,input.SelectionEnd);
            int end=input is null ? 0 : Math.Max(input.SelectionStart,input.SelectionEnd);
            var target=Sheet.Selected.Clone();
            string revision=session.Revision, expected=displayedRevision;
            byte[]? native=null;
            string? value;
            if(readText is not null) value=await readText();
            else {
                Task<ClipboardRead.Snapshot> ReadSnapshot() {
                    // The API verifies current clipboard ownership. Avoid an OLE
                    // round trip for our own data; never dispose clipboard-owned data.
                    return ClipboardRead.CaptureAsync<IAsyncDataTransfer>(Clipboard.TryGetInProcessDataAsync,Clipboard.TryGetDataAsync,async (data,owned)=>{
                        diagnose?.Invoke($"clipboard owned={owned}");
                        return await ReadWorksheetClipboardAsync(data,input is null);

                    });
                }
                var acquired=await ClipboardRead.AcquireAsync(ReadSnapshot,
                    ()=>request==clipboardRequest && interaction==interactionVersion && !editing && !fileAction && !shuttingDown,
                    diagnose);
                native=acquired.Bytes;value=acquired.Text;
            }
            if (request!=clipboardRequest || interaction!=interactionVersion || value is null || editing || fileAction || shuttingDown || session.Revision!=revision
                || Sheet.Selected.Row!=target.Row || Sheet.Selected.Column!=target.Column
                || (input is not null && (input.Text!=before || Math.Min(input.SelectionStart,input.SelectionEnd)!=start || Math.Max(input.SelectionStart,input.SelectionEnd)!=end))) {
                diagnose?.Invoke($"paste invalidated: request={request!=clipboardRequest}; interaction={interaction!=interactionVersion}; noValue={value is null}; revision={session.Revision!=revision}; target={Sheet.Selected.Row!=target.Row || Sheet.Selected.Column!=target.Column}");return;
            }
            if (input is null) {
                if (!frameReady || session.DocumentId.Length==0) {diagnose?.Invoke("paste has no ready document frame");return;}
                if(native is null && System.Text.Encoding.UTF8.GetByteCount(value)>262144) throw new InvalidDataException("Clipboard payload exceeds 256 KiB.");
                byte[] payload=native ?? System.Text.Encoding.UTF8.GetBytes(value);
                if(payload.Length>262144) throw new InvalidDataException("Clipboard payload exceeds 256 KiB.");
                bool expand=!Sheet.HasRegion;
                editing=true;SetEditing(false);refresh?.Cancel();++displayRequest;
                try {
                    diagnose?.Invoke("engine paste begin");
                    var pasted=visible?await session.PasteVisibleAsync(expected,Sheet.SelectionFirst,Sheet.SelectionLast,payload,native is not null):await session.PasteBlockAsync(expected,Sheet.SelectionFirst,Sheet.SelectionLast,payload,native is not null);
                    diagnose?.Invoke("engine paste completed");
                    if(!visible && expand && interaction==interactionVersion) {Sheet.Select(pasted.PastedFirst);Sheet.Select(pasted.PastedLast,true);Sheet.Reveal(Sheet.Selected);}
                    await RefreshAsync();UpdateDocument();diagnose?.Invoke("paste frame refreshed");
                    if(visible){Sheet.HighlightPastedRows(pasted.VisibleRows,pasted.VisibleFirstColumn,pasted.VisibleWidth);StatusLabel.Text=$"Pasted {pasted.VisibleRows.Count*pasted.VisibleWidth} cells into worksheet rows {VisibleRowRuns(pasted.VisibleRows)}; hidden rows were skipped.";}
                } catch(WorkbookSessionException e) {
                    diagnose?.Invoke("engine paste rejected: "+e.GetType().Name);
                    if(!visible && native is null && !Sheet.HasRegion && value.IndexOfAny(['\r','\n','\t'])<0 && value.Length<=4096 && !(session.IsBindingProof && value.TrimStart().StartsWith("=",StringComparison.Ordinal) && value.Contains("@{",StringComparison.Ordinal))) {
                        CaptureDraft();draftRevision=expected;draftReplace=true;
                        synchronizing=true;FormulaEntry.Text=CellEditor.Text=value;synchronizing=false;
                        inlineEditing=true;PositionEditor();CellEditor.Focus();
                    }
                    StatusLabel.Text=e.Message;
                } finally {editing=false;SetEditing(session.DocumentId.Length!=0);}
                return;
            }
            value=value.TrimEnd('\r','\n');
            string pastedSource=before[..start]+value+before[end..];
            string editorPayload=pastedSource.StartsWith('\'') ? pastedSource[1..] : pastedSource;
            if(value.IndexOfAny(['\r','\n','\t'])>=0 || System.Text.Encoding.UTF8.GetByteCount(editorPayload)>4096) {
                StatusLabel.Text="Paste one single-line cell (up to 4096 UTF-8 bytes) inside an editor."; return;
            }
            input.Text=before[..start]+value+before[end..]; input.CaretIndex=start+value.Length;
            input.SelectionStart=input.SelectionEnd=input.CaretIndex; input.Focus();
        } catch (OperationCanceledException) { diagnose?.Invoke("clipboard operation cancelled"); /* A cancelled read never installs or retries an edit. */ }
        catch (Exception e) { diagnose?.Invoke("clipboard exception: "+e.GetType().Name); StatusLabel.Text="Clipboard operation failed: "+e.Message; }
    }
    private double startupReadyMs;
    private static string RustExecutablePath() => Environment.GetEnvironmentVariable("PIVOT_SESSION_PATH") ?? Path.Combine(AppContext.BaseDirectory, OperatingSystem.IsWindows() ? "pivot-session.exe" : "pivot-session");
    private async Task StartAsync()
    {
        try {
            string path = RustExecutablePath();
            if (!File.Exists(path)) throw new FileNotFoundException("Rust session executable missing. Start with scripts/run_desktop.py or set PIVOT_SESSION_PATH.");
            session = await WorkbookSession.StartAsync(path);
            ConfigureFilterSession(session);
            if (shuttingDown) { await session.DisposeAsync(); return; }
            if(Environment.GetEnvironmentVariable("PIVOT_BINDING_PROOF")=="1") await session.StartBindingsAsync();
            else if(Environment.GetEnvironmentVariable("PIVOT_PROJECTION_PROOF")=="1") await session.StartProofAsync();
            await RefreshAsync(); SetEditing(displayedRevision.Length != 0);
            startupReadyMs=Program.Startup.Elapsed.TotalMilliseconds;
            if (Environment.GetEnvironmentVariable("PIVOT_UI_PEER_REPORT") is { } peerReport)
                await ClipboardPeerAsync(peerReport,Environment.GetEnvironmentVariable("PIVOT_UI_PEER_MODE") ?? "");
            else if (Environment.GetEnvironmentVariable("PIVOT_UI_SMOKE_REPORT") is { } report) await SmokeAsync(report);
        } catch (Exception e) {
            StatusLabel.Text = e.Message;
            if ((Environment.GetEnvironmentVariable("PIVOT_UI_SMOKE_REPORT") ?? Environment.GetEnvironmentVariable("PIVOT_UI_PEER_REPORT")) is { } report) {
                await File.WriteAllTextAsync(report, JsonSerializer.Serialize(new DesktopReport { status = "failed", error = e.ToString() }, ReportJsonContext.Default.DesktopReport)); if(session is not null) await session.DisposeAsync(); allowClose=true; Close();
            }
        }
    }
    private void UpdateDocument()
    {
        if (session is null) return;
        if(session.IsProjectionProof) {
            Title="Experimental projection document — Pivot";
            DocumentLabel.Text="Experimental · In memory · Cannot save"+(session.IsDirty?" · Modified":"");
            return;
        }
        string name = session.DocumentId.Length == 0 ? "No workbook open" : session.DocumentPath.Length == 0 ? "Untitled" : Path.GetFileName(session.DocumentPath);
        bool pending = draftCell is not null && (draftReplace || (FormulaEntry.Text ?? "") != draftOriginal);
        Title = $"{name}{(session.IsDirty || pending ? " *" : "")} — Pivot";
        DocumentLabel.Text = $"{name} · {(pending ? "Uncommitted edit" : session.IsDirty ? "Unsaved changes" : session.DocumentPath.Length == 0 ? "Untitled" : "Saved")}{(session.InstallationUncertain ? " · Installation uncertain: reopen to verify" : "")}";
        if(session.DocumentId.Length!=0) DocumentLabel.Text+=$" · {session.PositionedCells:N0}/{session.PositionedCellLimit:N0} positions · {session.AllocatedCellIdentities:N0}/{session.AllocatedCellIdentityLimit:N0} identities";
    }
    private async Task<bool> ConfirmDiscardAsync()
    {
        if (session is null || !session.IsDirty) return true;
        bool proof=session.IsProjectionProof;
        var dialog = new Window { Title=proof?"Discard experimental document":"Unsaved changes", Width=430, Height=170, CanResize=false, WindowStartupLocation=WindowStartupLocation.CenterOwner };
        var panel = new StackPanel { Margin=new Thickness(16), Spacing=14 };
        panel.Children.Add(new TextBlock { Text=proof?"Discard this experimental in-memory document? It cannot be saved.":"Save changes before continuing?", TextWrapping=global::Avalonia.Media.TextWrapping.Wrap });
        var buttons = new StackPanel { Orientation=global::Avalonia.Layout.Orientation.Horizontal, Spacing=12 };
        foreach (var choice in proof?new[] { "Discard", "Cancel" }:new[] { "Save", "Discard", "Cancel" }) { var button=new Button { Content=choice }; button.Click += (_,_)=>dialog.Close(choice); buttons.Children.Add(button); }
        panel.Children.Add(buttons); dialog.Content=panel;
        var result=await dialog.ShowDialog<string?>(this);
        if (result=="Save") return await SaveDocumentAsync(false);
        return result=="Discard";
    }
    private async Task<bool> SaveDocumentAsync(bool saveAs, string? chosenPath = null)
    {
        if(session?.IsProjectionProof==true){StatusLabel.Text="Experimental document cannot be saved. Discard or cancel.";return false;}
        if (session is null || session.DocumentId.Length==0 || !await CommitAsync()) return false;
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
        ++interactionVersion; fileAction=true; SetEditing(false);
        try { if (await CommitAsync()) await action(); }
        catch (Exception e) { StatusLabel.Text=e.Message; }
        finally { fileAction=false; UpdateDocument(); SetEditing(session.DocumentId.Length!=0); }
    }
    private async void SaveClick(object? sender,RoutedEventArgs e) => await FileActionAsync(async()=>{await SaveDocumentAsync(false);});
    private async void SaveAsClick(object? sender,RoutedEventArgs e) => await FileActionAsync(async()=>{await SaveDocumentAsync(true);});
    private async void NewClick(object? sender,RoutedEventArgs e) => await FileActionAsync(async()=>{
        if (!await ConfirmDiscardAsync()) return;
        refresh?.Cancel(); ++displayRequest; await session!.NewAsync(session.Revision,discard:true); Sheet.Select(new Position{Row=1,Column=1}); await RefreshAsync();
    });
    private async void OpenClick(object? sender,RoutedEventArgs e) => await FileActionAsync(async()=>{
        var files=await StorageProvider.OpenFilePickerAsync(new FilePickerOpenOptions { Title="Open Pivot workbook", AllowMultiple=false });
        var path=files.Count==1 ? files[0].TryGetLocalPath() : null; if(path is null || !await ConfirmDiscardAsync()) return;
        refresh?.Cancel(); ++displayRequest; await OpenDocumentAsync(path);
    });
    private async Task OpenDocumentAsync(string path) { await session!.OpenAsync(session.Revision,path,discard:true); Sheet.Select(new Position{Row=1,Column=1}); await RefreshAsync(); }
    private async void CloseDocumentClick(object? sender,RoutedEventArgs e) => await FileActionAsync(async()=>{
        if (!await ConfirmDiscardAsync()) return;
        refresh?.Cancel(); ++displayRequest; await session!.CloseDocumentAsync(session.Revision,discard:true);
        displayedRevision="";displayedGeneration="";Sheet.SetProofFrame(null);Sheet.SetCells([]);FormulaEntry.Text="";StatusLabel.Text="Workbook closed. Choose New or Open.";
    });
    private void SetEditing(bool enabled) {
        UpdateFillMenu();
        bool bulkReady=FilterBulkReady;
        CopyCellsButton.IsEnabled=PasteCellsButton.IsEnabled=enabled&&Sheet.HasVisibleRows&&(bulkReady||draftCell is not null);
        FilterBar.IsEnabled=enabled&&!shuttingDown&&!fileAction;
        FormatFields.IsEnabled=enabled&&Sheet.HasVisibleRows&&bulkReady;MoveButton.IsEnabled=enabled&&Sheet.HasVisibleRows&&bulkReady;
        ClearButton.IsEnabled = FormulaEntry.IsEnabled = CellEditor.IsEnabled = enabled && Sheet.HasVisibleRows && !shuttingDown && !fileAction && Sheet.ProjectedSelection is null;
        WorksheetBar.IsVisible=session is not null && !session.IsBindingProof && !session.IsProjectionProof;
        WorksheetBar.IsEnabled=enabled&&!shuttingDown&&!fileAction;
        BindingPanel.IsVisible=session?.IsBindingProof==true;BindingPanel.IsEnabled=enabled&&!shuttingDown&&!fileAction;
        ProofPanel.IsVisible=session?.IsProjectionProof==true;ProofPanel.IsEnabled=enabled && !shuttingDown && !fileAction;
        // Pointer capture survives the asynchronous commit preceding a selection.
        Sheet.IsEnabled=(enabled || editing) && Sheet.HasVisibleRows && !shuttingDown && !fileAction;
    }
    private void SelectCell()
    {
        UpdateFillMenu();
        if(ShowProjectedSelection())return;
        if(!Sheet.HasVisibleRows){AddressLabel.Text="";SelectionLabel.Text="No visible worksheet rows. Clear Filter or review its criteria.";if(draftCell is null){synchronizing=true;FormulaEntry.Text="";synchronizing=false;}return;}
        AddressLabel.Text = Pivot.Avalonia.WorksheetView.Address(Sheet.Selected);
        SelectionLabel.Text=$"Selection {Pivot.Avalonia.WorksheetView.Address(Sheet.SelectionFirst)}:{Pivot.Avalonia.WorksheetView.Address(Sheet.SelectionLast)} · Active {AddressLabel.Text} · Copy/Paste target selection · Edit/Clear/Move target active cell only";
        uint hidden=Sheet.HiddenRows(Sheet.SelectionFirst.Row,Sheet.SelectionLast.Row);if(hidden!=0)SelectionLabel.Text+=$" · Includes {hidden:N0} filtered-out rows";
        if(Sheet.SelectedCell is {SourceProducer.Length:>0} binding) SelectionLabel.Text+=$" · Bound {binding.SourceProducerName}.{binding.SourceOutputName} · Producer {binding.SourceProducer} · Output {binding.SourceOutput}";
        if (draftCell is not null) { PositionEditor(); return; }
        synchronizing = true;
        FormulaEntry.Text = CellSource();
        synchronizing = false;
        var cell = Sheet.SelectedCell;
        if (cell?.Error.Length > 0) StatusLabel.Text = cell.Error;
    }
    private string CellSource() => Sheet.SelectedCell?.ContentCase switch {
            Cell.ContentOneofCase.Boolean => Sheet.SelectedCell.Boolean ? "TRUE" : "FALSE",
            Cell.ContentOneofCase.Text => "'" + Sheet.SelectedCell.Text,
            Cell.ContentOneofCase.Formula => Sheet.SelectedCell.Formula,
            Cell.ContentOneofCase.Number => WorkbookSession.Decode(Sheet.SelectedCell.Number).ToString(CultureInfo.InvariantCulture),
            _ => ""
        };
    private async Task RefreshAsync()
    {
        if (session is null || shuttingDown) return;
        if(displayedRevision!=session.Revision)frameReady=false;
        refresh?.Cancel(); refresh?.Dispose(); refresh = new CancellationTokenSource();
        var cancellation = refresh.Token; long request = ++displayRequest;
        EnsureWorksheetViews();
        string worksheet=session.WorksheetId,document=session.DocumentId;ulong documentGeneration=session.DocumentGeneration;long view=worksheetViewEpoch;

        var visible = Stopwatch.StartNew();
        try {
            await Task.Delay(35, cancellation); // Coalesce scrolling before queuing Rust work.
            if(!session.IsProjectionProof && Sheet.ProjectionFrame is not null)Sheet.SetProofFrame(null);
            var first=new Position{Row=Sheet.FirstRow,Column=Sheet.FirstColumn};
            var last=new Position{Row=Math.Min(Sheet.TotalVisualRows,Sheet.FirstRow+(uint)Sheet.VisibleRows-1),Column=Sheet.FirstColumn+9};
            var frame = session.IsProjectionProof ? await session.ReadProofAsync(first,last,cancellation)
                : session.IsBindingProof ? await session.ReadAsync(new Position { Row = Sheet.FirstRow, Column = Sheet.FirstColumn },new Position { Row = Math.Min(WorkbookCapacity.MaximumRows, Sheet.FirstRow + (uint)Sheet.VisibleRows - 1), Column = Sheet.FirstColumn + 9 }, cancellation)
                : await session.ReadFilteredAsync(Sheet.FirstRow,(uint)Sheet.VisibleRows,Sheet.FirstColumn,Sheet.FirstColumn+9,cancellation);
            if (shuttingDown || view!=worksheetViewEpoch || documentGeneration!=session.DocumentGeneration || document!=session.DocumentId || worksheet!=session.WorksheetId || (!session.IsProjectionProof && (frame.Worksheet!=worksheet || frame.Cells.Any(c=>c.Worksheet!=worksheet))) || request != displayRequest || frame.Revision != session.Revision || frame.ComputationalState!=session.ComputationalState) return;
            if(!session.IsProjectionProof&&!session.IsBindingProof&&frame.FilterGeneration!=session.FilterGeneration)return;
            if(frame.ProofView is { } proof && (proof.Epoch!=session.ProofEpoch || proof.Presentation!=session.ProofPresentation))return;
            Sheet.SetProofFrame(frame.ProofView);if(request!=displayRequest)return;
            if(!session.IsProjectionProof&&!session.IsBindingProof){
                if(draftCell is not null&&!editing&&frame.FilterVisibleMask.Length==8192&&(frame.FilterVisibleMask.Span[(int)(draftCell.Row-1)/8]&(1<<((int)(draftCell.Row-1)%8)))==0){filterPublicationDeferred=true;filterViewPending=true;FilterStatusLabel.Text="Finish or cancel the editor before filter visibility changes.";return;}
                if(displayedFilterGeneration!=frame.FilterGeneration){++interactionVersion;++clipboardRequest;displayedFilterGeneration=frame.FilterGeneration;}
                Sheet.SetFilterFrame(frame);ShowFilterStatus(frame);
            }
            var dispatch = Stopwatch.StartNew();
            Sheet.SetCells(frame.Cells); Sheet.SetStyles(frame.Styles); frameReady = true; displayedRevision = frame.Revision; displayedGeneration = frame.Generation;
            StatusLabel.Text = $"Ready · {frame.Cells.Count} populated cells · {frame.Cells.Count(c => c.Error.Length > 0)} formula errors";
            if(frame.ProofView?.Error.Length>0)StatusLabel.Text="Publication failed; showing last successful result: "+frame.ProofView.Error;
            UpdateWorksheetTabs();SelectCell(); UpdateDocument();
            lastUiApply = dispatch.Elapsed.TotalMilliseconds;
            TimingLabel.Text = $"Viewport round trip {session.LastRoundTrip.TotalMilliseconds:F2} ms · Rust {session.LastRustExecutionNs / 1e6:F2} ms · encode C# {session.LastClientSerialization.TotalMilliseconds:F2} / Rust {session.LastRustSerializationNs / 1e6:F2} ms · UI apply {dispatch.Elapsed.TotalMilliseconds:F2} ms (paint measured separately)";
            if(frame.FilterStatus==3){_ = RefreshPendingFilterAsync(session,session.WorksheetId,session.DocumentGeneration,request);}
            var painted = await Sheet.PaintCompletion.WaitAsync(TimeSpan.FromSeconds(5), cancellation);
            if (request == displayRequest && !shuttingDown) { lastPaintDelay = painted.TotalMilliseconds; lastVisibleUpdate = visible.Elapsed.TotalMilliseconds; }
        } catch (OperationCanceledException) { }
          catch (Exception e) { if (request == displayRequest && !shuttingDown) StatusLabel.Text = e.Message; }
    }
    private async Task<bool> EditAsync(Edit edit, string? expected = null)
    {
        if (session is null || editing) return false;
        editing = true; SetEditing(false); refresh?.Cancel(); ++displayRequest;
        try {
            if(session.IsBindingProof && edit.ActionCase==Edit.ActionOneofCase.Entry && edit.Entry.TrimStart().StartsWith("=",StringComparison.Ordinal) && edit.Entry.Contains("@{",StringComparison.Ordinal))
                await session.ApplyStructuredFormulaAsync(expected??displayedRevision,edit.At,edit.Entry);
            else await session.ApplyAsync(expected??displayedRevision,[edit]);
            await RefreshAsync();return true;
        }
        catch (WorkbookSessionException e) when (e.IsConflict) { await RefreshAsync(); StatusLabel.Text = "Worksheet changed. Your edit is retained; press Escape to reload before editing again."; return false; }
        catch (Exception e) { StatusLabel.Text = e.Message; return false; }
        finally { editing = false; SetEditing(session.DocumentId.Length != 0); }
    }
    private void CaptureDraft()
    {
        if (draftCell is not null) return;
        if(Sheet.HasRegion) Sheet.Select(Sheet.Selected);
        ++interactionVersion; draftCell = Sheet.Selected.Clone(); draftRevision = displayedRevision; draftOriginal = CellSource(); draftReplace = false;
    }
    private void FormulaChanged()
    {
        if (synchronizing || session is null || session.DocumentId.Length == 0 || editing || fileAction || shuttingDown) return;
        ++interactionVersion; CaptureDraft(); synchronizing = true; CellEditor.Text = FormulaEntry.Text; synchronizing = false; UpdateDocument();
    }
    private void CellEditorChanged()
    {
        if (synchronizing || draftCell is null) return;
        ++interactionVersion; synchronizing = true; FormulaEntry.Text = CellEditor.Text; synchronizing = false; UpdateDocument();
    }
    private void PositionEditor()
    {
        UpdateFillMenu();
        bool visible = draftCell is not null && Sheet.IsVisibleCell(draftCell);
        CellEditor.IsVisible = visible && inlineEditing;
        EditorLayer.IsHitTestVisible = CellEditor.IsVisible;
        if (draftCell is null) return;
        var rect = Sheet.CellBounds(draftCell);
        Canvas.SetLeft(CellEditor, rect.X); Canvas.SetTop(CellEditor, rect.Y);
        CellEditor.Width = rect.Width; CellEditor.Height = rect.Height;
    }
    private bool inlineEditing;
    private void BeginCellEdit(bool replace, string text = "")
    {
        if (Sheet.ProjectedSelection is not null) {StatusLabel.Text="Projected records are read-only.";return;}
        if (session is null || !Sheet.HasVisibleRows || editing || fileAction || shuttingDown || !frameReady || displayedRevision.Length == 0 || session.DocumentId.Length == 0) return;
        Sheet.EndPointerSelection(false); CaptureDraft(); inlineEditing = true;
        synchronizing = true;
        if (replace) { FormulaEntry.Text = text; draftReplace = true; }
        CellEditor.Text = FormulaEntry.Text;
        synchronizing = false; PositionEditor(); UpdateDocument(); CellEditor.Focus();
        CellEditor.CaretIndex = CellEditor.Text?.Length ?? 0;
    }
    private void CancelDraft()
    {
        ++interactionVersion; ++clipboardRequest; draftCell = null; inlineEditing = false; PositionEditor(); SelectCell(); UpdateDocument(); Sheet.Focus();
        if(filterPublicationDeferred){filterPublicationDeferred=false;_ = RefreshAsync();}
    }
    private async Task<bool> CommitAsync()
    {
        if (draftCell is null) return !editing;
        if (editing) return false;
        string source = FormulaEntry.Text ?? "";
        if (source == draftOriginal && !draftReplace) { CancelDraft(); return true; }
        var edit = new Edit { At = draftCell.Clone(), Entry = source };
        if (!await EditAsync(edit, draftRevision)) return false;
        CancelDraft(); UpdateDocument(); return true;
    }
    private async Task<bool> SelectAsync(Position at, bool extend=false, Func<bool>? current=null)
    {
        if (editing || fileAction || shuttingDown || !await CommitAsync()) return false;
        if(!Sheet.RowIsShown(at.Row)){StatusLabel.Text="That address is filtered out. Clear Filter to reveal it.";return false;}
        if(current is not null && !current()) return false;
        Sheet.Select(at,extend);
        uint firstRow = Sheet.FirstRow, firstColumn = Sheet.FirstColumn;
        Sheet.Reveal(at);
        if (firstRow != Sheet.FirstRow || firstColumn != Sheet.FirstColumn || displayedRevision != session?.Revision) await RefreshAsync();
        if(current is not null && !current()) return false;
        Sheet.Focus(); return true;
    }
    private async Task NavigateAsync(int rows, int columns, bool extend=false)
    {
        var at = Sheet.Selected;
        await SelectAsync(new Position { Row=Sheet.StepVisibleRow(at.Row,rows), Column=(uint)Math.Clamp((int)at.Column+columns,1,1024) },extend);
    }
    private async void GridKeyDown(object? sender, KeyEventArgs e)
    {
        if(e.Key==Key.Escape && (Sheet.IsSelecting || Sheet.IsSelectionPending)) {
            ++clipboardRequest; Sheet.EndPointerSelection(true); e.Handled=true; return;
        }
        if (editing || fileAction || shuttingDown) return;
        if(Sheet.ProjectedSelection is not null){if(e.Key==Key.Escape)Sheet.Select(Sheet.Selected);else StatusLabel.Text="Projected records are read-only. Escape returns to the worksheet.";e.Handled=true;return;}
        if (e.Key==Key.Escape) {
            ++clipboardRequest;
            if(Sheet.IsSelecting) Sheet.EndPointerSelection(true); else Sheet.Select(Sheet.Selected);
            e.Handled=true; return;
        }
        Sheet.EndPointerSelection(false);
        if (!frameReady && (e.Key is Key.F2 or Key.Enter or Key.Delete)) return;
        if (e.Key is Key.F2 or Key.Enter) { BeginCellEdit(false); e.Handled=true; }
        else if (e.Key == Key.Delete) { e.Handled=true; await EditAsync(new Edit { At=Sheet.Selected.Clone(), Clear=true }); }
        else if(e.Key is Key.Home or Key.End or Key.PageUp or Key.PageDown) {
            e.Handled=true;
            var at=Sheet.Selected;
            bool whole=e.KeyModifiers.HasFlag(KeyModifiers.Control) || e.KeyModifiers.HasFlag(KeyModifiers.Meta);
            if(e.Key is Key.Home or Key.End) await SelectAsync(new Position {Row=whole ? Sheet.ClosestVisibleRow(e.Key==Key.Home ? 1u : (uint)WorkbookCapacity.MaximumRows,e.Key==Key.Home) : at.Row, Column=e.Key==Key.Home ? 1u : 1024u},e.KeyModifiers.HasFlag(KeyModifiers.Shift));
            else await NavigateAsync(e.Key==Key.PageUp ? -Sheet.VisibleRows : Sheet.VisibleRows,0,e.KeyModifiers.HasFlag(KeyModifiers.Shift));
        }
        else if (e.Key is Key.Up or Key.Down or Key.Left or Key.Right or Key.Tab) {
            e.Handled=true;
            await NavigateAsync(e.Key==Key.Up ? -1 : e.Key==Key.Down ? 1 : 0,
                e.Key==Key.Left || (e.Key==Key.Tab && e.KeyModifiers.HasFlag(KeyModifiers.Shift)) ? -1 : e.Key is Key.Right or Key.Tab ? 1 : 0,
                e.Key!=Key.Tab && e.KeyModifiers.HasFlag(KeyModifiers.Shift));
        }
    }
    private void GridTextInput(object? sender, TextInputEventArgs e)
    {
        if (string.IsNullOrEmpty(e.Text) || IsCommandInput()) return;
        BeginCellEdit(true,e.Text); e.Handled=true;
    }
    private async void EntryKeyDown(object? sender, KeyEventArgs e)
    {
        if (e.Key==Key.Escape) { ++clipboardRequest; e.Handled=true; if(!editing) CancelDraft(); }
        else if (ReferenceEquals(sender,CellEditor) && inlineEditing && draftCell is not null
            && e.KeyModifiers==KeyModifiers.None && e.Key is Key.Up or Key.Down or Key.Left or Key.Right) {
            e.Handled=true;
            if (editing || fileAction || shuttingDown) return;
            if (await CommitAsync()) await NavigateAsync(e.Key==Key.Up ? -1 : e.Key==Key.Down ? 1 : 0,
                e.Key==Key.Left ? -1 : e.Key==Key.Right ? 1 : 0);
        }
        else if (e.Key is Key.Enter or Key.Tab) {
            e.Handled=true;
            if (await CommitAsync()) await NavigateAsync(e.Key==Key.Enter ? (e.KeyModifiers.HasFlag(KeyModifiers.Shift)?-1:1) : 0,
                e.Key==Key.Tab ? (e.KeyModifiers.HasFlag(KeyModifiers.Shift)?-1:1) : 0);
        }
    }
    private async void ClearClick(object? sender, RoutedEventArgs e) {
        if (Sheet.ProjectedSelection is null && await CommitAsync()) await EditAsync(new Edit { At=Sheet.Selected.Clone(), Clear=true });
    }
    private async void MoveClick(object? sender, RoutedEventArgs e)
    {
        try { if (Sheet.ProjectedSelection is null && await CommitAsync()) await EditAsync(new Edit { At=Sheet.Selected.Clone(), MoveTo=Pivot.Avalonia.WorksheetView.ParseAddress(MoveTarget.Text ?? "") }); }
        catch (Exception ex) { StatusLabel.Text=ex.Message; }
    }
    private void UpClick(object? sender, RoutedEventArgs e) => Sheet.Scroll(-Sheet.VisibleRows);
    private void DownClick(object? sender, RoutedEventArgs e) => Sheet.Scroll(Sheet.VisibleRows);
    private void LeftClick(object? sender, RoutedEventArgs e) => Sheet.Scroll(0, -10);
    private void RightClick(object? sender, RoutedEventArgs e) => Sheet.Scroll(0, 10);

    private async Task SmokeAsync(string report)
    {
        await CapacityAutomationAsync(report);
        await SortingAutomationAsync(report);
        await FilteringAutomationAsync(report);
        await VisibleClipboardAutomationAsync(report);
        await StructureAutomationAsync(report);
        await CsvAutomationAsync(report);
        await FillAutomationAsync(report);
        await AggregateAutomationAsync(report);
        await WorksheetAutomationAsync(report);
        await FormattingAutomationAsync(report);
        await RegionAutomationAsync(report);
        await TypedAutomationAsync(report);
        await BindingAutomationAsync(report);
        await ProjectionAutomationAsync(report);
        await TextAutomationAsync(report);
        await ClipboardChecklistAsync(report);
        await ArrowEditAutomationAsync();
        void Check(bool condition, string message) { if (!condition) throw new InvalidOperationException(message); }
        Position P(uint r, uint c) => new() { Row = r, Column = c };
        var timings = new System.Collections.Generic.List<RenderTiming>();
        var mouse=new Pointer(987,PointerType.Mouse,true);
        var held=new PointerPointProperties(global::Avalonia.Input.RawInputModifiers.LeftMouseButton,PointerUpdateKind.LeftButtonPressed);
        var released=new PointerPointProperties(global::Avalonia.Input.RawInputModifiers.None,PointerUpdateKind.LeftButtonReleased);
        async Task Press(Position at,KeyModifiers modifiers=KeyModifiers.None,int clicks=1) {
            Sheet.RaiseEvent(new PointerPressedEventArgs(Sheet,mouse,this,Sheet.TranslatePoint(Sheet.CellBounds(at).Center,this)!.Value,0,held,modifiers,clicks));
            await WaitFor(()=>Sheet.Selected.Row==at.Row && Sheet.Selected.Column==at.Column);
        }
        void MovePointer(Point point) => Sheet.RaiseEvent(new PointerEventArgs(InputElement.PointerMovedEvent,Sheet,mouse,this,Sheet.TranslatePoint(point,this)!.Value,0,held,KeyModifiers.None));
        void ReleasePointer(Point point) => Sheet.RaiseEvent(new PointerReleasedEventArgs(Sheet,mouse,this,Sheet.TranslatePoint(point,this)!.Value,0,released,KeyModifiers.None,MouseButton.Left));
        string selectionRevision=session!.Revision;bool selectionDirty=session.IsDirty;
        await SelectAsync(P(2,2));
        Key(Sheet,global::Avalonia.Input.Key.Right,KeyModifiers.Shift);
        Key(Sheet,global::Avalonia.Input.Key.Down,KeyModifiers.Shift);
        Check(Sheet.Anchor.Row==2 && Sheet.Anchor.Column==2 && Sheet.Selected.Row==3 && Sheet.Selected.Column==3,"Shift arrows preserve anchor and extend active corner");
        await Press(P(2,2));MovePointer(Sheet.CellBounds(P(5,4)).Center);ReleasePointer(Sheet.CellBounds(P(5,4)).Center);
        Check(Sheet.SelectionFirst.Row==2 && Sheet.SelectionLast.Row==5 && Sheet.SelectionLast.Column==4 && !Sheet.IsSelecting,"Routed drag selects B2:D5 and releases capture");
        await Press(P(7,6),KeyModifiers.Shift);ReleasePointer(Sheet.CellBounds(P(7,6)).Center);
        Check(Sheet.Anchor.Row==2 && Sheet.Anchor.Column==2 && Sheet.Selected.Row==7 && Sheet.Selected.Column==6,"Shift click extends from prior anchor");
        await Press(P(3,3));MovePointer(Sheet.CellBounds(P(6,5)).Center);Key(Sheet,global::Avalonia.Input.Key.Escape);
        Check(Sheet.Anchor.Row==2 && Sheet.Anchor.Column==2 && Sheet.Selected.Row==7 && Sheet.Selected.Column==6 && !Sheet.IsSelecting,"Escape restores pre-drag selection");
        await Press(P(2,2));MovePointer(Sheet.CellBounds(P(4,3)).Center);mouse.Capture(null);
        Check(!Sheet.IsSelecting && Sheet.Selected.Row==4 && Sheet.Selected.Column==3,"Lost capture stops drag and retains last selection");
        int initialRows=Sheet.VisibleRows;
        await Press(P((uint)initialRows,9));
        MovePointer(new Point(Sheet.Bounds.Width+20,Sheet.CellBounds(P((uint)initialRows,1)).Bottom+30));
        await WaitFor(()=>Sheet.FirstRow>=3 && Sheet.FirstColumn>=2);
        Check(Sheet.Selected.Row>(uint)initialRows && Sheet.Selected.Column>10,"Captured extension auto-scrolls both axes");
        Key(Sheet,global::Avalonia.Input.Key.Escape);Check(!Sheet.IsSelecting,"Escape stops auto-scroll even with a pending viewport");await RefreshAsync();
        Sheet.Scroll(-5000,-5000);await RefreshAsync();await SelectAsync(P(3,3));
        Key(Sheet,global::Avalonia.Input.Key.End,KeyModifiers.Shift);await WaitFor(()=>Sheet.Selected.Column==1024);await RefreshAsync();
        Check(Sheet.Anchor.Column==3 && Sheet.SelectionLast.Column==1024,"Shift End extends to bounded row end");
        Key(Sheet,global::Avalonia.Input.Key.Home);await WaitFor(()=>Sheet.Selected.Column==1);await RefreshAsync();
        Check(!Sheet.HasRegion && Sheet.Selected.Row==3,"Home collapses selection at column one");
        Key(Sheet,global::Avalonia.Input.Key.End,KeyModifiers.Control);await WaitFor(()=>Sheet.Selected.Row==WorkbookCapacity.MaximumRows && Sheet.Selected.Column==1024);await RefreshAsync();
        Check(Sheet.FirstRow+(uint)Sheet.VisibleRows-1==WorkbookCapacity.MaximumRows && Sheet.FirstColumn+9==1024,"Whole-sheet End reveals bounded bottom-right cell");
        Key(Sheet,global::Avalonia.Input.Key.Home,KeyModifiers.Control);await WaitFor(()=>Sheet.Selected.Row==1 && Sheet.Selected.Column==1);await RefreshAsync();
        Key(Sheet,global::Avalonia.Input.Key.PageDown,KeyModifiers.Shift);await RefreshAsync();
        Check(Sheet.Anchor.Row==1 && Sheet.Selected.Row==(uint)Sheet.VisibleRows+1,"Shift PageDown extends by visible rows");
        Sheet.Scroll(-5000,-5000);await RefreshAsync();await SelectAsync(P(2,2));await SelectAsync(P(5,4),true);
        var anchorBefore=Sheet.Anchor;var activeBefore=Sheet.Selected;
        Height+=40;await Task.Delay(60);await RefreshAsync();Height-=40;await Task.Delay(60);await RefreshAsync();
        MoveTarget.Focus();UpdateFocusAppearance();
        Check(!Sheet.ActiveFocus && Sheet.Anchor.Equals(anchorBefore) && Sheet.Selected.Equals(activeBefore),"Resize, refresh and focus preserve rectangle");
        Sheet.Focus();UpdateFocusAppearance();
        using(var selectionImage=new RenderTargetBitmap(new PixelSize((int)Bounds.Width,(int)Bounds.Height))) {
            selectionImage.Render(this);selectionImage.Save(Path.ChangeExtension(report,"selection.png"),PngBitmapEncoderOptions.Default);
        }
        Check(session.Revision==selectionRevision && session.IsDirty==selectionDirty,"Selection-only mouse/keyboard/viewport actions do not mutate workbook or dirty state");
        var selectionFrame=await session.ReadAsync(P(1,1),P(1,1));
        Check(selectionFrame.SelectionStats.Selections==0 && selectionFrame.SelectionStats.Points==0,"UI rectangles allocate no computational selections or content");
        var selectionRead=new TaskCompletionSource<string?>(TaskCreationOptions.RunContinuationsAsynchronously);
        var selectionPaste=ClipboardAsync(true,()=>selectionRead.Task);Sheet.Select(Sheet.Selected);
        selectionRead.SetResult("77");await selectionPaste;
        Check(session.Revision==selectionRevision && draftCell is null,"Changing only the rectangular extent invalidates a late clipboard read");
        await SelectAsync(P(2,2));await SelectAsync(P(5,4),true);Key(Sheet,global::Avalonia.Input.Key.F2);
        Check(!Sheet.HasRegion && draftCell?.Row==5 && draftCell.Column==4,"Editing collapses rectangle to the active cell without a model mutation");CancelDraft();
        Check(session.Revision==selectionRevision,"Editor transition alone leaves model unchanged");
        BeginCellEdit(true,"3");
        Sheet.RaiseEvent(new PointerPressedEventArgs(Sheet,mouse,this,Sheet.TranslatePoint(Sheet.CellBounds(P(2,2)).Center,this)!.Value,0,held,KeyModifiers.None,1));
        Key(Sheet,global::Avalonia.Input.Key.Escape);
        await WaitFor(()=>!editing && draftCell is null);
        Check(Sheet.Selected.Row==5 && Sheet.Selected.Column==4 && !Sheet.IsSelecting,"Escape invalidates pending pointer selection after asynchronous edit commit");
        Check(WorkbookSession.Decode(Sheet.CellAt(P(5,4))!.Value)==3m,"Cancelling selection does not pretend to undo an already dispatched cell edit");
        BeginCellEdit(true,"4");
        Sheet.RaiseEvent(new PointerPressedEventArgs(Sheet,mouse,this,Sheet.TranslatePoint(Sheet.CellBounds(P(2,2)).Center,this)!.Value,0,held,KeyModifiers.None,1));
        ReleasePointer(Sheet.CellBounds(P(2,2)).Center);
        await WaitFor(()=>!editing && draftCell is null && Sheet.Selected.Row==2 && Sheet.Selected.Column==2);
        Check(!Sheet.IsSelecting && WorkbookSession.Decode(Sheet.CellAt(P(5,4))!.Value)==4m,"Normal release preserves a pending click after committing the previous edit");
        await EditAsync(new Edit{At=P(5,4),Clear=true});
        await SelectAsync(P(1,1));
        var nativeClipboard=Clipboard ?? throw new InvalidOperationException("Native clipboard is unavailable");
        // Preserve the user clipboard where it contains text; non-text formats are not copied by this check.
        string? previousClipboard=await nativeClipboard.TryGetTextAsync();
        await SelectAsync(P(9,4));BeginCellEdit(true,"=1+2");Check(await CommitAsync(),"Clipboard source formula");
        string cellCopyTrace="";
        await ClipboardAsync(false,diagnose:phase=>cellCopyTrace+="; "+phase);
        await AutomationExpectClipboardTextAsync(nativeClipboard,"=1+2","Cell copy preserves original formula source",
            ()=>cellCopyTrace+$"; editor={clipboardEditor?.Name}; draft={draftCell is not null}; editing={editing}; status={StatusLabel.Text}");
        await SelectAsync(P(9,5));await ClipboardAsync(true);
        Check(draftCell is null && !CellEditor.IsVisible && Sheet.Selected.Row==9 && Sheet.Selected.Column==5,"Cell paste commits without entering editor or moving selection");
        Check(Sheet.CellAt(P(9,5))?.Value is not null,"Native paste produces destination value: "+StatusLabel.Text+"; frameReady="+frameReady+"; editor="+(clipboardEditor?.Name ?? "none"));
        Check(WorkbookSession.Decode(Sheet.CellAt(P(9,5))!.Value)==3m && session!.IsDirty,"Pasted source calculates through Rust and marks dirty");
        BeginCellEdit(false);
        // Let native focus/layout work finish before establishing the synthetic
        // selection. Clipboard awaits may otherwise be the first dispatcher yield.
        await global::Avalonia.Threading.Dispatcher.UIThread.InvokeAsync(()=>{},global::Avalonia.Threading.DispatcherPriority.Background);
        await WaitFor(()=>ReferenceEquals(FocusManager?.GetFocusedElement(),CellEditor));
        CellEditor.SelectionStart=1;CellEditor.SelectionEnd=2;
        string EditorClipboardState()=>$"editor={clipboardEditor?.Name}; focus={FocusManager?.GetFocusedElement()?.GetType().Name}; selection={CellEditor.SelectionStart}:{CellEditor.SelectionEnd}; caret={CellEditor.CaretIndex}; expectedSource={CellEditor.Text=="=1+2"}; selectedOne={CellEditor.SelectedText=="1"}";
        string pasteTrace=EditorClipboardState();
        Check(CellEditor.SelectedText=="1"&&ReferenceEquals(clipboardEditor??(inlineEditing?CellEditor:FormulaEntry),CellEditor),"Editor copy selection and routing; "+pasteTrace);
        Task VerifyPublishedText(string expectedText,string phase)=>AutomationExpectClipboardTextAsync(nativeClipboard,
            expectedText,phase,()=>pasteTrace+"; "+EditorClipboardState());
        // Native publication can lag the write. Retry bounded reads only: never
        // repeat a copy, paste or edit to obtain an acceptance pass.
        await ClipboardAsync(false,diagnose:phase=>pasteTrace+="; copy: "+phase);
        await VerifyPublishedText("1","Editor copies selected substring");
        await nativeClipboard.SetTextAsync("5");
        await VerifyPublishedText("5","Editor replacement text is published to native clipboard");
        Check(CellEditor.SelectionStart==1&&CellEditor.SelectionEnd==2,"Editor selection remains intact before paste; "+EditorClipboardState());
        await ClipboardAsync(true,diagnose:phase=>pasteTrace+="; "+phase);
        Check(FormulaEntry.Text=="=5+2","Editor paste replaces selected substring; "+pasteTrace+$"; editorMatches={CellEditor.Text=="=5+2"}; sourceLength={FormulaEntry.Text?.Length}; selectionAfter={CellEditor.SelectionStart}:{CellEditor.SelectionEnd}; status={StatusLabel.Text}");CancelDraft();
        var primary=OperatingSystem.IsMacOS()?KeyModifiers.Meta:KeyModifiers.Control;
        var shortcut=new KeyEventArgs{RoutedEvent=InputElement.KeyDownEvent,Key=global::Avalonia.Input.Key.C,KeyModifiers=primary};
        Sheet.RaiseEvent(shortcut);Check(shortcut.Handled,"Platform primary copy shortcut is intercepted");
        await Task.Delay(100);
        var shortcutText=new TextInputEventArgs{RoutedEvent=InputElement.TextInputEvent,Text="c"};Sheet.RaiseEvent(shortcutText);
        Check(shortcutText.Handled && draftCell is null,"Shortcut text cannot start a cell draft");
        shortcutModifiers=KeyModifiers.None;
        string multilineRevision=session!.Revision;
        await nativeClipboard.SetTextAsync("1\t2");await ClipboardAsync(true);Check(draftCell is null && session.Revision!=multilineRevision && Sheet.HasRegion,"Rectangular TSV commits immediately and selects destination");
        Check(WorkbookSession.Decode(Sheet.CellAt(P(9,5))!.Value)==1m && WorkbookSession.Decode(Sheet.CellAt(P(9,6))!.Value)==2m,"TSV slot correspondence");
        await EditAsync(new Edit {At=P(9,6),Clear=true});
        await SelectAsync(P(12,2));await ClipboardAsync(true,()=>Task.FromResult<string?>("2\t=B12+1\n3\t=SUM($B$12:B13)"));
        await SelectAsync(P(12,2));await SelectAsync(P(13,3),true);await ClipboardAsync(false);
        using(var nativeData=await nativeClipboard.TryGetDataAsync()) Check(nativeData is not null && nativeData.Contains(CellBlockFormat),"Native typed rectangle is on OS clipboard");
        await SelectAsync(P(15,5));await ClipboardAsync(true);
        Check(Sheet.SelectionFirst.Row==15 && Sheet.SelectionLast.Row==16 && Sheet.SelectionLast.Column==6 && draftCell is null,"Native rectangular paste expands selection without editor");
        Check(Sheet.CellAt(P(15,6))!.Formula=="=E15+1" && WorkbookSession.Decode(Sheet.CellAt(P(15,6))!.Value)==3m,"Native scalar copy translates through Rust");
        Check(Sheet.CellAt(P(16,6))!.Formula=="=SUM($B$12:E16)","Native SUM mixed-axis translation");
        using(var clipboardImage=new RenderTargetBitmap(new PixelSize((int)Bounds.Width,(int)Bounds.Height))) {
            clipboardImage.Render(this);clipboardImage.Save(Path.ChangeExtension(report,"clipboard.png"),PngBitmapEncoderOptions.Default);
        }
        await SelectAsync(P(15,5));await SelectAsync(P(15,6),true);string mismatched=session.Revision;
        await ClipboardAsync(true);Check(session.Revision==mismatched && Sheet.SelectionLast.Row==15,"Mismatched native destination rejects atomically");
        var forgedItem=new DataTransferItem();forgedItem.Set(CellBlockFormat,new byte[]{1,2,3});forgedItem.SetText("99");var forged=new DataTransfer();forged.Add(forgedItem);await nativeClipboard.SetDataAsync(forged);
        await SelectAsync(P(17,5));string forgedRevision=session.Revision;await ClipboardAsync(true);
        Check(session.Revision==forgedRevision && draftCell is null,"Malformed native clipboard never falls back to valid TSV");
        await session.ApplyAsync(session.Revision,[new Edit{At=P(12,2),Clear=true},new Edit{At=P(12,3),Clear=true},new Edit{At=P(13,2),Clear=true},new Edit{At=P(13,3),Clear=true},new Edit{At=P(15,5),Clear=true},new Edit{At=P(15,6),Clear=true},new Edit{At=P(16,5),Clear=true},new Edit{At=P(16,6),Clear=true}]);await RefreshAsync();
        // Deterministic delayed clipboard reads exercise the production async path.
        await SelectAsync(P(8,4));
        var delayed=new TaskCompletionSource<string?>(TaskCreationOptions.RunContinuationsAsynchronously);
        string beforeLate=session!.Revision;var late=ClipboardAsync(true,()=>delayed.Task);
        await SelectAsync(P(8,5));await SelectAsync(P(8,4));delayed.SetResult("42");await late;
        Check(session.Revision==beforeLate && draftCell is null,"Away-and-back selection cancels late paste");
        var cancelled=new TaskCompletionSource<string?>(TaskCreationOptions.RunContinuationsAsynchronously);
        var pending=ClipboardAsync(true,()=>cancelled.Task);Key(Sheet,global::Avalonia.Input.Key.Escape);
        cancelled.SetResult("42");await pending;Check(session.Revision==beforeLate,"Escape cancels clipboard read without edit");
        await ClipboardAsync(true,()=>Task.FromResult<string?>("12.50"));
        Check(draftCell is null && WorkbookSession.Decode(Sheet.CellAt(P(8,4))!.Value)==12.50m,"Numeric paste commits exact decimal immediately");
        await nativeClipboard.SetTextAsync("14");string keyPasteRevision=session.Revision;
        var pasteKey=new KeyEventArgs{RoutedEvent=InputElement.KeyDownEvent,Key=global::Avalonia.Input.Key.V,KeyModifiers=primary};
        Sheet.RaiseEvent(pasteKey);await WaitFor(()=>session.Revision!=keyPasteRevision && !editing && draftCell is null);
        shortcutModifiers=KeyModifiers.None;
        Check(pasteKey.Handled && WorkbookSession.Decode(Sheet.CellAt(P(8,4))!.Value)==14m,"Primary paste shortcut commits immediately");
        await EditAsync(new Edit{At=P(8,5),Formula="=D8+1"});
        await ClipboardAsync(true,()=>Task.FromResult<string?>("7"));
        Check(WorkbookSession.Decode(Sheet.CellAt(P(8,5))!.Value)==8m,"Direct paste recalculates dependent");
        string numericRevision=session.Revision;
        await ClipboardAsync(true,()=>Task.FromResult<string?>("0.1234567890123456789012345678901"));
        Check(session.Revision==numericRevision && draftCell is not null && CellEditor.IsVisible,"Rejected exact numeric paste keeps correctable draft");CancelDraft();
        await ClipboardAsync(true,()=>Task.FromResult<string?>("=("));
        Check(draftCell is null && Sheet.CellAt(P(8,4))!.Error.Length>0,"Invalid formula paste commits editable Rust formula draft");
        await ClipboardAsync(true,()=>Task.FromResult<string?>(""));
        Check(draftCell is null && Sheet.CellAt(P(8,4))!.ContentCase==Cell.ContentOneofCase.Blank,"Empty paste clears cell immediately");
        await ClipboardAsync(false);await AutomationExpectClipboardTextAsync(nativeClipboard,"","Copying a blank cell clears stale clipboard text");
        string beforeCancelledRead=session.Revision;
        await ClipboardAsync(true,()=>Task.FromCanceled<string?>(new CancellationToken(true)));
        Check(session.Revision==beforeCancelledRead && draftCell is null,"Cancelled clipboard operation causes no edit");
        var supersededRead=new TaskCompletionSource<string?>(TaskCreationOptions.RunContinuationsAsynchronously);
        var firstPaste=ClipboardAsync(true,()=>supersededRead.Task);
        await ClipboardAsync(true,()=>Task.FromResult<string?>("13"));string lastPasteRevision=session.Revision;
        supersededRead.SetResult("999");await firstPaste;
        Check(session.Revision==lastPasteRevision && WorkbookSession.Decode(Sheet.CellAt(P(8,4))!.Value)==13m,"New paste supersedes old read without duplicate commit");
        var newerRead=new TaskCompletionSource<string?>(TaskCreationOptions.RunContinuationsAsynchronously);
        var obsolete=ClipboardAsync(true,()=>newerRead.Task);
        await session.ApplyAsync(session.Revision,[new Edit{At=P(8,4),Number=WorkbookSession.Encode(99m)}]);
        string newerRevision=session.Revision;newerRead.SetResult("42");await obsolete;
        Check(session.Revision==newerRevision,"Changed revision rejects late clipboard data");
        // Session is newer than the displayed frame: do not silently overwrite it.
        await ClipboardAsync(true,()=>Task.FromResult<string?>("44"));
        Check(session.Revision==newerRevision && draftCell is not null && FormulaEntry.Text=="44","Stale displayed revision conflicts without replay and retains source");CancelDraft();
        await SelectAsync(P(8,4));BeginCellEdit(false);CellEditor.SelectAll();string editorRevision=session.Revision;
        Sheet.Scroll(20);await RefreshAsync();
        Check(!CellEditor.IsVisible && draftCell is not null,"Scrolled-away editor retains draft context");
        // Native focus changes on hiding may collapse selection before paste starts.
        string hiddenText=CellEditor.Text ?? "";
        int hiddenStart=Math.Min(CellEditor.SelectionStart,CellEditor.SelectionEnd), hiddenEnd=Math.Max(CellEditor.SelectionStart,CellEditor.SelectionEnd);
        await ClipboardAsync(true,()=>Task.FromResult<string?>("55"));
        Check(session.Revision==editorRevision && draftCell is not null && FormulaEntry.Text==hiddenText[..hiddenStart]+"55"+hiddenText[hiddenEnd..],"Hidden active editor paste respects its current caret without committing");
        Sheet.Scroll(-20);await RefreshAsync();CancelDraft();
        FormulaEntry.Focus();FormulaEntry.SelectAll();
        await ClipboardAsync(true,()=>Task.FromResult<string?>("66"));
        Check(session.Revision==editorRevision && FormulaEntry.Text=="66","Formula bar paste remains uncommitted text");CancelDraft();
        await EditAsync(new Edit{At=P(8,4),Clear=true});await EditAsync(new Edit{At=P(8,5),Clear=true});
        BlinkCaret.IsChecked=false;Check(CellEditor.CaretBlinkInterval==TimeSpan.Zero && FormulaEntry.CaretBlinkInterval==TimeSpan.Zero,"Steady caret option");
        BlinkCaret.IsChecked=true;
        MoveTarget.Focus();UpdateFocusAppearance();Check(!Sheet.ActiveFocus,"Unfocused active cell has muted outline");
        await EditAsync(new Edit{At=P(9,4),Clear=true});await EditAsync(new Edit{At=P(9,5),Clear=true});
        if(previousClipboard is not null)await nativeClipboard.SetTextAsync(previousClipboard);

        async Task WaitFor(Func<bool> predicate) {
            var timeout=Stopwatch.StartNew();
            while (!predicate()) { if(timeout.Elapsed > TimeSpan.FromSeconds(10)) throw new InvalidOperationException("Input action timed out: "+StatusLabel.Text); await Task.Delay(10); }
        }
        void Key(Control target, global::Avalonia.Input.Key key, KeyModifiers modifiers=KeyModifiers.None) =>
            target.RaiseEvent(new KeyEventArgs { RoutedEvent=InputElement.KeyDownEvent, Key=key, KeyModifiers=modifiers });
        void Type(string text) => Sheet.RaiseEvent(new TextInputEventArgs { RoutedEvent=InputElement.TextInputEvent, Text=text });
        await SelectAsync(P(10,4)); Type("1");
        Check(CellEditor.IsVisible && CellEditor.Text=="1" && FormulaEntry.Text=="1","Grid typing starts synchronized inline editor");
        CellEditor.Text="12.50"; await RefreshAsync();
        Check(CellEditor.Text=="12.50" && FormulaEntry.Text=="12.50","Refresh preserves uncommitted draft");
        var otherCellPoint=Sheet.TranslatePoint(Sheet.CellBounds(P(10,5)).Center,this)!.Value;
        Check(this.InputHitTest(otherCellPoint)==Sheet,"Inline overlay leaves other cells clickable");
        using (var draftImage = new RenderTargetBitmap(new PixelSize((int)Bounds.Width,(int)Bounds.Height))) {
            draftImage.Render(this); draftImage.Save(Path.ChangeExtension(report,"editing.png"),PngBitmapEncoderOptions.Default);
        }
        Key(CellEditor,global::Avalonia.Input.Key.Enter); await WaitFor(()=>draftCell is null && !editing && Sheet.Selected.Row==11);
        Check(WorkbookSession.Decode(Sheet.CellAt(P(10,4))!.Value)==12.50m,"Inline Enter commits and moves down");
        await SelectAsync(P(10,4)); string untouchedRevision=session!.Revision;
        Key(Sheet,global::Avalonia.Input.Key.F2); Check(CellEditor.Text=="12.50","F2 loads original exact decimal");
        CellEditor.Text="99";Key(CellEditor,global::Avalonia.Input.Key.Left,KeyModifiers.Shift);
        Check(Sheet.Selected.Row==10 && Sheet.Selected.Column==4,"Modified editing arrow stays in text editor");
        Key(CellEditor,global::Avalonia.Input.Key.Escape);
        Check(session.Revision==untouchedRevision && !CellEditor.IsVisible,"Escape cancels without mutation");
        Key(Sheet,global::Avalonia.Input.Key.F2);Key(CellEditor,global::Avalonia.Input.Key.Enter);
        await WaitFor(()=>draftCell is null && Sheet.Selected.Row==11);
        Check(session.Revision==untouchedRevision,"Unchanged F2 does not resubmit");
        await SelectAsync(P(10,5));Type("=D10+1");Key(CellEditor,global::Avalonia.Input.Key.Tab);
        await WaitFor(()=>draftCell is null && !editing && Sheet.Selected.Column==6);
        Check(WorkbookSession.Decode(Sheet.CellAt(P(10,5))!.Value)==13.50m,"Tab commits formula and moves right");
        Key(Sheet,global::Avalonia.Input.Key.Left); await WaitFor(()=>Sheet.Selected.Column==5);
        Key(Sheet,global::Avalonia.Input.Key.F2);Check(CellEditor.Text=="=D10+1","F2 loads original formula, not result");
        CancelDraft();Type("79228162514264337593543950336");string rejectedRevision=session.Revision;
        Check(!await SelectAsync(P(10,6)) && Sheet.Selected.Column==5 && FormulaEntry.Text=="79228162514264337593543950336" && session.Revision==rejectedRevision,"Rejected edit blocks selection and retains draft");
        CancelDraft(); await SelectAsync(P(10,4));Type("7");
        Check(await SelectAsync(P(10,5)),"Click-selection path commits before selection");
        Check(WorkbookSession.Decode(Sheet.CellAt(P(10,5))!.Value)==8m,"Click-committed input recalculates dependent");
        FormulaEntry.Text="=D10+2";Check(CellEditor.Text=="=D10+2","Formula bar synchronizes shared draft");
        Check(await CommitAsync(),"Formula bar commits shared draft");
        await SelectAsync(P(10,4));Key(Sheet,global::Avalonia.Input.Key.Delete);await WaitFor(()=>!editing && Sheet.CellAt(P(10,4))?.ContentCase==Cell.ContentOneofCase.Blank);
        Check(WorkbookSession.Decode(Sheet.CellAt(P(10,5))!.Value)==2m,"Delete clears selected cell");
        await EditAsync(new Edit{At=P(10,5),Clear=true});
        await SelectAsync(P(WorkbookCapacity.MaximumRows,1024));Key(Sheet,global::Avalonia.Input.Key.Right);
        Check(Sheet.Selected.Column==1024 && Sheet.FirstColumn+9==1024,"Keyboard navigation reveals and bounds edge");
        await SelectAsync(P(1,1)); Sheet.Scroll(-5000,-5000);await RefreshAsync();

        async Task Enter(uint r, uint c, string value) {
            CancelDraft(); Sheet.Select(P(r,c)); SelectCell(); BeginCellEdit(true,value);
            var clock = Stopwatch.StartNew(); await CommitAsync();
            Check(displayedRevision == session!.Revision, "Displayed frame must match current exact revision.");
            Check(displayedGeneration.Length > 0, "Displayed frame must identify its calculation generation.");
            timings.Add(new RenderTiming { edit_to_render_ms = clock.Elapsed.TotalMilliseconds, ui_apply_ms = lastUiApply, paint_delay_ms = lastPaintDelay, refresh_to_render_ms = lastVisibleUpdate });
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
        await SelectAsync(P(1,3));BeginCellEdit(false);string unchangedFormula=session.Revision;
        Check(await CommitAsync() && session.Revision==unchangedFormula && Value(1,3)==7m,"Unchanged formula edit preserves historical stable binding");
        await Enter(1,3,"=A1+B1"); Check(Value(1,3)==102m,"UI explicit rebind");
        await EditAsync(new Edit { At=P(1,2), Clear=true }); Check(Value(1,3)==100m,"UI clear");
        // The editor must submit the displayed revision, even if a background
        // operation has advanced the session before the new frame is installed.
        await session!.ApplyAsync(session.Revision, [new Edit { At=P(2,2), Number=WorkbookSession.Encode(4m) }]);
        Sheet.Select(P(1,1)); FormulaEntry.Text="999"; await CommitAsync();
        Check(Value(1,1)==100m && Value(2,2)==4m && draftCell is not null && FormulaEntry.Text=="999","UI stale edit must conflict, retain draft and refresh without overwriting.");
        // Superseded viewport loads must never install after a newer request.
        CancelDraft();
        var superseded = RefreshAsync(); Sheet.Scroll(20); await RefreshAsync(); await superseded;
        Check(Sheet.FirstRow==21 && Sheet.CellAt(P(1,3)) is null,"UI scroll bounded frame");
        Sheet.Scroll(-20); await RefreshAsync(); Check(Value(1,3)==100m,"UI scroll back");
        Sheet.Scroll(100000,5000); await RefreshAsync();
        Check(Sheet.FirstRow+(uint)Sheet.VisibleRows-1==WorkbookCapacity.MaximumRows && Sheet.FirstColumn+9==1024,"UI scroll respects worksheet bounds");
        Sheet.Scroll(-100000,-5000); await RefreshAsync(); Check(Value(1,3)==100m,"UI return from worksheet edge");
        await Enter(2,8,"1500");await Enter(3,8,"2350");await Enter(4,8,"1275");await Enter(5,8,"900");
        await Enter(6,9,"=SUM(H2:H5)");Check(Value(6,9)==6025m,"UI SUM");
        await Enter(3,8,"2400");Check(Value(6,9)==6075m,"UI SUM member edit");
        await EditAsync(new Edit{At=P(3,8),MoveTo=P(8,8)});Check(Value(6,9)==3675m,"UI SUM boundary move");
        await Enter(3,8,"10");Check(Value(6,9)==3685m,"UI SUM coordinate reuse");
        string testRoot=Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT") ?? Path.GetDirectoryName(report)!;
        Directory.CreateDirectory(testRoot);
        string savedPath=Path.Combine(testRoot,"ui-"+Guid.NewGuid()+".pivot");
        await SelectAsync(P(3,8));BeginCellEdit(true,"11");
        Check(await SaveDocumentAsync(true,savedPath),"Save As flushes active inline draft");
        Check(Value(6,9)==3686m && draftCell is null,"Save includes unfinished cell edit exactly once");
        string savedDocument=session!.DocumentId;
        await session.NewAsync(session.Revision);await RefreshAsync();Check(Sheet.CellAt(P(1,3)) is null,"UI New");
        await OpenDocumentAsync(savedPath);Check(Value(1,3)==100m && session.DocumentId==savedDocument && !session.IsDirty,"UI Open saved workbook");
        Check(Value(6,9)==3686m,"UI WB3 SUM reopen");
        await Enter(2,1,"6");Check(session.IsDirty,"Reopened editing marks dirty");
        Check(await SaveDocumentAsync(false),"UI Save existing workbook: "+StatusLabel.Text);
        File.Copy(savedPath,Path.ChangeExtension(report,".pivot"),overwrite:true);
        using (var image = new RenderTargetBitmap(new PixelSize((int)Bounds.Width, (int)Bounds.Height))) {
            image.Render(this); image.Save(Path.ChangeExtension(report,".png"), PngBitmapEncoderOptions.Default);
        }
        await session!.DisposeAsync(); Check(session.FinalCharge==0,"UI session cleanup");
        await File.WriteAllTextAsync(report, JsonSerializer.Serialize(new DesktopReport { graphical_backend=PlatformImpl?.GetType().FullName, status="passed", visible_clipboard="passed: CB6 visible copy, actual destination row review, one-use visible paste, hidden-row integrity, geometric formula origins, stale clipboard refusal, preferred-native no fallback and native-only exact Strings", filtering="passed: typed exact criteria, confirmed expansion, coherent original-row viewport, navigation, hidden-row operation tickets, changed clipboard cancellation, automatic refresh, structural review, zero visible rows and WB12 reopen", sorting="passed: actual Sort dialog, explicit adjacent-data expansion, stable multikey record permutation, header exclusion, both formatting policies, exact leading-zero Strings, scalar bindings and WB11 reopen", structure="passed: row/column menus, stable identities, local lazy reference faults, unchanged editor provenance, operand repair, CB5/Fill, clipping notice, focused band selection and WB11 persistence", region_formatting="passed: selection-wide property overwrite, full-selection Mixed, row/column headers, worksheet defaults, explicit reset/inherit and WB10 reopen", capacity="passed: 65,536-row address, select, scroll, edit, Fill, native clipboard, WB10 reopen and capacity diagnostics", csv="passed: quoted inference controls, all-quoted choices, leading zeros, Blank/empty String, calculated export and native lifecycle", basic_formatting="passed: exact Rust display, Format pane/mixed patches, graph-free styled blanks, editor exactness, native clipboard and WB10 reopen", typed_values="passed: typed literal/results, Unicode LEN, lazy IF, editor provenance, native clipboard translation and WB8 Save/reopen", multiple_worksheets="passed: Add/Rename/tabs, commit on switch, quoted effective names, stale frame rejection, per-sheet view state and WB7 Save/reopen", rectangular_fill="passed: routed four-direction menu/shortcuts, typed edge translation, identity/no-op, editor exclusion, unchanged clipboard and WB5 save/reopen", spreadsheet_aggregates="passed: actual inline MIN/MAX/AVERAGE/COUNT, blank/zero transitions, strict SUM, typed no-values, native copy and WB5 save/reopen", worksheet_binding="passed: real inline structured formulas, publication without definition changes, typed faults, rename/no-op editing, mixed A1 duplication, unchanged OS clipboard and non-savable close lifecycle", structured_projection="passed: bounded read-only bands, stable ordinary geometry, keyed stale-target rejection, detached total invocation and explicit non-savable discard/cancel lifecycle", worksheet_text="passed: inline text entry/escape, no-op editing, strict scalar/SUM errors, mixed native/TSV paste and WB5 save/reopen", startup_to_ready_ms=startupReadyMs, backend="real Rust", viewport_rows=Sheet.VisibleRows, viewport_columns=10, maximum_viewport_cells=200, remaining_charge=session.FinalCharge, arrow_edit="passed: four directions commit numeric/formula drafts, unchanged F2 preserves binding, bounds clamp, invalid numeric stays put, modified arrows/formula bar retain text editing", clipboard_checklist="passed: deterministic five-group coverage with separate OS clipboard process and actual close Save/Discard/Cancel/clean flows; physical input and native pickers excluded", rectangular_clipboard="passed: native typed rectangle and external TSV, Rust scalar/SUM transformation, destination highlighting, shape rollback and malformed-native no-fallback", rectangular_selection="passed: routed drag/shift extension, anchor/active bounds, capture cancellation, two-axis auto-scroll, Home/End/Page navigation, resize/refresh/focus preservation, no workbook mutation or computational selections, editor transition and late clipboard protection", direct_paste="passed: immediate numeric/formula/empty paste, dependent recalculation, rejected numeric correction, invalid formula draft, active-editor insertion, delayed selection/revision cancellation and conflict preservation", clipboard_focus="passed: native clipboard cell/source and substring round trips, platform shortcut interception/text suppression, native rectangular transfer and editor table rejection, inactive focus and caret option", in_place_editing="passed: routed keyboard/type entry, draft/refresh, F2/no-op versus explicit rebind, Enter/Tab/Escape/Delete, navigation, selection commit, stale/rejected retention, active-draft Save/reopen", timings=timings, note="Rendered-frame timing includes coalescing and UI scheduling, not physical display presentation." }, ReportJsonContext.Default.DesktopReport));
        allowClose = true; Close();
    }
}
