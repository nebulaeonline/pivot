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
        Sheet.SelectionChanged += () => { ++interactionVersion; SelectCell();if(Sheet.NeedsGeometryHeaders&&!editing&&!fileAction&&!shuttingDown)_=RefreshGeometryAsync(true); };
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
            await ReleaseGeometryViewAsync();
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
    private async Task ClipboardAsync(bool paste, Func<Task<string?>>? readText = null, Action<string>? diagnose = null, bool visible = false, bool textOnly = false)
    {
        if (session is null || editing || fileAction || shuttingDown || Clipboard is null) {diagnose?.Invoke("blocked at entry");return;}
        if(Sheet.ProjectedSelection is not null){StatusLabel.Text="Projected clipboard operations are not supported in this proof.";return;}
        try {
            var input=clipboardEditor ?? (draftCell is null ? null : inlineEditing ? CellEditor : FormulaEntry);
            if (input==CellEditor && draftCell is null) input=null;
            using var section=input is null?(await BeginSectionCommandAsync())?.Invoke():null;if(input is null&&section is null)return;
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
                        return await ReadWorksheetClipboardAsync(data,input is null && !textOnly);

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
            ConfigureGeometrySession(session);
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
                await File.WriteAllTextAsync(report, JsonSerializer.Serialize(new DesktopReport { status = "failed", error = e.ToString() }, ReportJsonContext.Default.DesktopReport)); await ReleaseGeometryViewAsync();if(session is not null) await session.DisposeAsync(); allowClose=true; Close();
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
        refresh?.Cancel(); ++displayRequest; await ReleaseGeometryViewAsync(); await session!.NewAsync(session.Revision,discard:true); Sheet.Select(new Position{Row=1,Column=1}); await RefreshAsync();
    });
    private async void OpenClick(object? sender,RoutedEventArgs e) => await FileActionAsync(async()=>{
        var files=await StorageProvider.OpenFilePickerAsync(new FilePickerOpenOptions { Title="Open Pivot workbook", AllowMultiple=false });
        var path=files.Count==1 ? files[0].TryGetLocalPath() : null; if(path is null || !await ConfirmDiscardAsync()) return;
        refresh?.Cancel(); ++displayRequest; await OpenDocumentAsync(path);
    });
    private async Task OpenDocumentAsync(string path) {
        await ReleaseGeometryViewAsync();Sheet.EnterGeometryMode();
        try{await session!.OpenAsync(session.Revision,path,discard:true);Sheet.Select(new Position{Row=1,Column=1});await RefreshAsync();}
        catch{await RefreshAsync();throw;}
    }
    private async void CloseDocumentClick(object? sender,RoutedEventArgs e) => await FileActionAsync(async()=>{
        if (!await ConfirmDiscardAsync()) return;
        refresh?.Cancel(); ++displayRequest; await ReleaseGeometryViewAsync(); await session!.CloseDocumentAsync(session.Revision,discard:true);
        displayedRevision="";displayedGeneration="";Sheet.SetProofFrame(null);Sheet.SetCells([]);FormulaEntry.Text="";StatusLabel.Text="Workbook closed. Choose New or Open.";
    });
    private void SetEditing(bool enabled) {
        UpdateFillMenu();
        bool bulkReady=FilterBulkReady;
        CopyCellsButton.IsEnabled=PasteCellsButton.IsEnabled=enabled&&Sheet.HasVisibleRows&&(bulkReady||draftCell is not null);
        FilterBar.IsEnabled=enabled&&!shuttingDown&&!fileAction&&session?.IsReadOnly!=true;
        FormatFields.IsEnabled=enabled&&Sheet.HasVisibleRows&&bulkReady&&session?.IsReadOnly!=true;MoveButton.IsEnabled=enabled&&Sheet.HasVisibleRows&&bulkReady&&session?.IsReadOnly!=true;
        ClearButton.IsEnabled = FormulaEntry.IsEnabled = CellEditor.IsEnabled = enabled && Sheet.HasVisibleRows && !shuttingDown && !fileAction && Sheet.ProjectedSelection is null && session?.IsReadOnly!=true;
        if(Sheet.GeometryMode&&!Sheet.RowIsShown(Sheet.Selected.Row)){FormulaEntry.IsEnabled=false;CellEditor.IsEnabled=false;}
        WorksheetBar.IsVisible=session is not null && !session.IsBindingProof && !session.IsProjectionProof;
        WorksheetBar.IsEnabled=enabled&&!shuttingDown&&!fileAction;
        WindowBar.IsEnabled=enabled&&!shuttingDown&&!fileAction;
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
        SelectionLabel.Text=$"Selection {Pivot.Avalonia.WorksheetView.Address(Sheet.SelectionFirst)}:{Pivot.Avalonia.WorksheetView.Address(Sheet.SelectionLast)} · Active {AddressLabel.Text} · Copy/Paste target selection · Edit targets active cell; Clear/Move target the complete selection";
        if(Sheet.GeometryMode&&geometryView is {IsDisposed:false} geometry){
            var columnRuler=geometry.Frame.Geometry.Headers.FirstOrDefault(h=>h.Column);var rowRuler=geometry.Frame.Geometry.Headers.FirstOrDefault(h=>!h.Column);
            if(columnRuler is not null)SelectionLabel.Text+=$" · Column ruler: row section {columnRuler.SectionFirst}–{columnRuler.SectionLast}";
            if(rowRuler is not null)SelectionLabel.Text+=$" · Row ruler: column section {rowRuler.SectionFirst}–{rowRuler.SectionLast}";
        }
        if(Sheet.SectionSelectionEnlarged)SelectionLabel.Text+=$" · Expanded from {SortDialog.RangeText(Sheet.RequestedFirst,Sheet.RequestedLast)}; changing operations require confirmation";
        if(Sheet.SelectionNeedsReview)SelectionLabel.Text+=" · Reselect after section changes";
        uint hidden=Sheet.HiddenRows(Sheet.SelectionFirst.Row,Sheet.SelectionLast.Row);if(hidden!=0)SelectionLabel.Text+=$" · Includes {hidden:N0} filtered-out rows";
        if(Sheet.SelectedCell?.Projected is { } projected)SelectionLabel.Text+=$" · Read-only producer {projected.Producer} · Item {projected.ItemKey} · Field {projected.FieldId}";
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
        if(!session.IsProjectionProof&&!session.IsBindingProof){await RefreshGeometryAsync();return;}
        await ReleaseGeometryViewAsync();
        if(displayedRevision!=session.Revision)frameReady=false;
        refresh?.Cancel(); refresh?.Dispose(); refresh = new CancellationTokenSource();
        var cancellation = refresh.Token; long request = ++displayRequest;
        EnsureWorksheetViews();
        var scopeInfo=session.GridScopes.FirstOrDefault(s=>s.Id==session.WorksheetId);
        if(scopeInfo is not null)Sheet.SetGridDimensions(scopeInfo.Rows,scopeInfo.Columns);
        bool windowScope=session.IsWindowScope;
        string worksheet=session.WorksheetId,document=session.DocumentId;ulong documentGeneration=session.DocumentGeneration;long view=worksheetViewEpoch;

        var visible = Stopwatch.StartNew();
        try {
            await Task.Delay(35, cancellation); // Coalesce scrolling before queuing Rust work.
            if(!session.IsProjectionProof && Sheet.ProjectionFrame is not null)Sheet.SetProofFrame(null);
            var first=new Position{Row=Sheet.FirstRow,Column=Sheet.FirstColumn};
            var last=new Position{Row=Math.Min(Sheet.TotalVisualRows,Sheet.FirstRow+(uint)Sheet.VisibleRows-1),Column=Math.Min((uint)Sheet.ColumnLimit,Sheet.FirstColumn+9)};
            var frame = session.IsProjectionProof ? await session.ReadProofAsync(first,last,cancellation)
                : session.IsBindingProof ? await session.ReadAsync(new Position { Row = Sheet.FirstRow, Column = Sheet.FirstColumn },new Position { Row = Math.Min(WorkbookCapacity.MaximumRows, Sheet.FirstRow + (uint)Sheet.VisibleRows - 1), Column = Sheet.FirstColumn + 9 }, cancellation)
                : await session.ReadFilteredAsync(Sheet.FirstRow,(uint)Sheet.VisibleRows,Sheet.FirstColumn,Math.Min((uint)Sheet.ColumnLimit,Sheet.FirstColumn+9),cancellation);
            if (shuttingDown || view!=worksheetViewEpoch || documentGeneration!=session.DocumentGeneration || document!=session.DocumentId || worksheet!=session.WorksheetId || windowScope!=session.IsWindowScope || frame.WindowScope!=windowScope || (!session.IsProjectionProof && (frame.Worksheet!=worksheet || frame.Cells.Any(c=>c.Worksheet!=worksheet))) || request != displayRequest || frame.Revision != session.Revision || frame.ComputationalState!=session.ComputationalState) return;
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
            if(frame.BoundProjection){UpdateBoundControls();StatusLabel.Text=BoundStatusLabel.Text;SetEditing(!editing&&!fileAction&&session.DocumentId.Length!=0);}
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
        if(session.IsReadOnly){StatusLabel.Text="Bound content is read-only. Edit the source worksheet instead.";return false;}
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
        if (synchronizing || session is null || session.IsReadOnly || session.DocumentId.Length == 0 || editing || fileAction || shuttingDown) return;
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
        CellEditor.Width = rect.Width; CellEditor.Height = rect.Height;CellEditor.FontSize=Sheet.GeometryMode?13*Sheet.PaintedGeometryZoom/100.0:13;
    }
    private bool inlineEditing;
    private void BeginCellEdit(bool replace, string text = "")
    {
        if (Sheet.ProjectedSelection is not null || session?.IsReadOnly==true) {StatusLabel.Text="Projected records are read-only. Edit the source worksheet instead.";return;}
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
        if(Sheet.GeometryMode&&!await Sheet.EnsureSectionSelectionAsync())return false;
        if(current is not null && !current()) return false;
        Sheet.Focus(); return true;
    }
    private async Task NavigateAsync(int rows, int columns, bool extend=false)
    {
        var at = Sheet.Selected;
        await SelectAsync(new Position { Row=Sheet.StepVisibleRow(at.Row,rows), Column=(uint)Math.Clamp((int)at.Column+columns,1,Sheet.ColumnLimit) },extend);
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
        else if (e.Key == Key.Delete) { e.Handled=true; await ClearSelectionAsync(); }
        else if(e.Key is Key.Home or Key.End or Key.PageUp or Key.PageDown) {
            e.Handled=true;
            var at=Sheet.Selected;
            bool whole=e.KeyModifiers.HasFlag(KeyModifiers.Control) || e.KeyModifiers.HasFlag(KeyModifiers.Meta);
            if(e.Key is Key.Home or Key.End) await SelectAsync(new Position {Row=whole ? Sheet.ClosestVisibleRow(e.Key==Key.Home ? 1u : (uint)Sheet.RowLimit,e.Key==Key.Home) : at.Row, Column=e.Key==Key.Home ? 1u : (uint)Sheet.ColumnLimit},e.KeyModifiers.HasFlag(KeyModifiers.Shift));
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
        await ClearSelectionAsync();
    }
    private async void MoveClick(object? sender, RoutedEventArgs e)
    {
        try { if (Sheet.ProjectedSelection is null && await CommitAsync()) {
            using var context=(await BeginSectionCommandAsync())?.Invoke();if(context is null)return;
            var destination=Pivot.Avalonia.WorksheetView.ParseAddress(MoveTarget.Text??"");
            if(Sheet.GeometryMode){editing=true;SetEditing(false);try{await session!.MoveRegionAsync(displayedRevision,Sheet.SelectionFirst,Sheet.SelectionLast,destination);await RefreshAsync();}finally{editing=false;SetEditing(session!.DocumentId.Length!=0);}}
            else await EditAsync(new Edit{At=Sheet.Selected.Clone(),MoveTo=destination});
        } }
        catch (Exception ex) { StatusLabel.Text=ex.Message; }
    }
    private void UpClick(object? sender, RoutedEventArgs e) => Sheet.Scroll(-Sheet.VisibleRows);
    private void DownClick(object? sender, RoutedEventArgs e) => Sheet.Scroll(Sheet.VisibleRows);
    private void LeftClick(object? sender, RoutedEventArgs e) => Sheet.Scroll(0, -Sheet.VisibleColumns);
    private void RightClick(object? sender, RoutedEventArgs e) => Sheet.Scroll(0, Sheet.VisibleColumns);

    private async Task SmokeAsync(string report)
    {
        await GeometryAutomationAsync(Path.ChangeExtension(report,".geometry"));
        await CapacityAutomationAsync(report);
        await SortingAutomationAsync(report);
        await FilteringAutomationAsync(report);
        await VisibleClipboardAutomationAsync(report);
        await VisibleEditAutomationAsync(report);
        await StructureAutomationAsync(report);
        await CsvAutomationAsync(report);
        await FillAutomationAsync(report);
        await AggregateAutomationAsync(report);
        await WorksheetAutomationAsync(report);
        await WindowAutomationAsync(report);
        await BoundAutomationAsync(report);
        await FormattingAutomationAsync(report);
        await RegionAutomationAsync(report);
        await TypedAutomationAsync(report);
        await BindingAutomationAsync(report);
        await ProjectionAutomationAsync(report);
        await TextAutomationAsync(report);
        await ClipboardChecklistAsync(report);
        await ArrowEditAutomationAsync();
        var timings=await InputAutomationAsync(report);
        await ReleaseGeometryViewAsync();await session!.DisposeAsync(); AutomationCheck(session.FinalCharge==0,"UI session cleanup");
        await File.WriteAllTextAsync(report, JsonSerializer.Serialize(new DesktopReport { graphical_backend=PlatformImpl?.GetType().FullName, status="passed", section_geometry="passed: complete invoice editor sequences in both construction orders, balanced/shared sizing, expanded operation confirmations and positional layout review/reset", bound="passed: actual binding dialog, physical Blank positions, read-only provenance, live source updates, explicit fixture refresh and exact leading-zero text", windows="passed: independent Window grid, dimensions, canonical formulas, native clipboard refusal, WB16 reopen and confirmed identity retirement", visible_edit="passed: three visible Clear modes, explicit built-in defaults, four immutable visible-edge Fill directions, geometric formulas, hidden integrity, exact review/cancel and degenerate no-op", visible_clipboard="passed: CB6 visible copy, actual destination row review, one-use visible paste, hidden-row integrity, geometric formula origins, stale clipboard refusal, preferred-native no fallback and native-only exact Strings", filtering="passed: typed exact criteria, confirmed expansion, coherent original-row viewport, navigation, hidden-row operation tickets, changed clipboard cancellation, automatic refresh, structural review, zero visible rows and WB16 reopen", sorting="passed: actual Sort dialog, explicit adjacent-data expansion, stable multikey record permutation, header exclusion, both formatting policies, exact leading-zero Strings, scalar bindings and WB16 reopen", structure="passed: row/column menus, stable identities, local lazy reference faults, unchanged editor provenance, operand repair, CB5/Fill, clipping notice, focused band selection and WB16 persistence", region_formatting="passed: selection-wide property overwrite, full-selection Mixed, row/column headers, worksheet defaults, explicit reset/inherit and WB16 reopen", capacity="passed: 65,536-row address, select, scroll, edit, Fill, native clipboard, WB16 reopen and capacity diagnostics", csv="passed: quoted inference controls, all-quoted choices, leading zeros, Blank/empty String, calculated export and native lifecycle", basic_formatting="passed: exact Rust display, Format pane/mixed patches, graph-free styled blanks, editor exactness, native clipboard and WB16 reopen", typed_values="passed: typed literal/results, Unicode LEN, lazy IF, editor provenance, native clipboard translation and WB16 Save/reopen", multiple_worksheets="passed: Add/Rename/tabs, commit on switch, quoted effective names, stale frame rejection, per-sheet view state and WB7 Save/reopen", rectangular_fill="passed: routed four-direction menu/shortcuts, typed edge translation, identity/no-op, editor exclusion, unchanged clipboard and WB5 save/reopen", spreadsheet_aggregates="passed: actual inline MIN/MAX/AVERAGE/COUNT, blank/zero transitions, strict SUM, typed no-values, native copy and WB5 save/reopen", worksheet_binding="passed: real inline structured formulas, publication without definition changes, typed faults, rename/no-op editing, mixed A1 duplication, unchanged OS clipboard and non-savable close lifecycle", structured_projection="passed: bounded read-only bands, stable ordinary geometry, keyed stale-target rejection, detached total invocation and explicit non-savable discard/cancel lifecycle", worksheet_text="passed: inline text entry/escape, no-op editing, strict scalar/SUM errors, mixed native/TSV paste and WB5 save/reopen", startup_to_ready_ms=startupReadyMs, backend="real Rust", viewport_rows=Sheet.VisibleRows, viewport_columns=Sheet.VisibleColumns, maximum_viewport_cells=4096, remaining_charge=session.FinalCharge, arrow_edit="passed: four directions commit numeric/formula drafts, unchanged F2 preserves binding, bounds clamp, invalid numeric stays put, modified arrows/formula bar retain text editing", clipboard_checklist="passed: deterministic five-group coverage with separate OS clipboard process and actual close Save/Discard/Cancel/clean flows; physical input and native pickers excluded", rectangular_clipboard="passed: native typed rectangle and external TSV, Rust scalar/SUM transformation, destination highlighting, shape rollback and malformed-native no-fallback", rectangular_selection="passed: routed drag/shift extension, anchor/active bounds, capture cancellation, two-axis auto-scroll, Home/End/Page navigation, resize/refresh/focus preservation, no workbook mutation or computational selections, editor transition and late clipboard protection", direct_paste="passed: immediate numeric/formula/empty paste, dependent recalculation, rejected numeric correction, invalid formula draft, active-editor insertion, delayed selection/revision cancellation and conflict preservation", clipboard_focus="passed: native clipboard cell/source and substring round trips, platform shortcut interception/text suppression, native rectangular transfer and editor table rejection, inactive focus and caret option", in_place_editing="passed: routed keyboard/type entry, draft/refresh, F2/no-op versus explicit rebind, Enter/Tab/Escape/Delete, navigation, selection commit, stale/rejected retention, active-draft Save/reopen", timings=timings, note="Rendered-frame timing includes coalescing and UI scheduling, not physical display presentation." }, ReportJsonContext.Default.DesktopReport));
        allowClose = true; Close();
    }
}
