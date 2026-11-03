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
public partial class MainWindow
{
    private async Task<System.Collections.Generic.List<RenderTiming>> InputAutomationAsync(string report)
    {
        void Check(bool condition, string message) { if (!condition) throw new InvalidOperationException(message); }
        Position P(uint r, uint c) => new() { Row = r, Column = c };
        async Task Ready(){await WaitFor(()=>frameReady && displayedRevision==session!.Revision && Sheet.GeometryInputReady);await Sheet.EnsureSectionSelectionAsync();await global::Avalonia.Threading.Dispatcher.UIThread.InvokeAsync(()=>{},global::Avalonia.Threading.DispatcherPriority.Background);await WaitFor(()=>frameReady && Sheet.GeometryInputReady);}

        var timings = new System.Collections.Generic.List<RenderTiming>();
        var mouse=new Pointer(987,PointerType.Mouse,true);
        var held=new PointerPointProperties(global::Avalonia.Input.RawInputModifiers.LeftMouseButton,PointerUpdateKind.LeftButtonPressed);
        var released=new PointerPointProperties(global::Avalonia.Input.RawInputModifiers.None,PointerUpdateKind.LeftButtonReleased);
        async Task Press(Position at,KeyModifiers modifiers=KeyModifiers.None,int clicks=1) {
            Sheet.RaiseEvent(new PointerPressedEventArgs(Sheet,mouse,this,Sheet.TranslatePoint(Sheet.CellBounds(at).Center,this)!.Value,0,held,modifiers,clicks));
            await WaitFor(()=>Sheet.Selected.Row==at.Row && Sheet.Selected.Column==at.Column);
            await Sheet.EnsureSectionSelectionAsync();
            await global::Avalonia.Threading.Dispatcher.UIThread.InvokeAsync(()=>{},global::Avalonia.Threading.DispatcherPriority.Background);
        }
        void MovePointer(Point point) => Sheet.RaiseEvent(new PointerEventArgs(InputElement.PointerMovedEvent,Sheet,mouse,this,Sheet.TranslatePoint(point,this)!.Value,0,held,KeyModifiers.None));
        void ReleasePointer(Point point) => Sheet.RaiseEvent(new PointerReleasedEventArgs(Sheet,mouse,this,Sheet.TranslatePoint(point,this)!.Value,0,released,KeyModifiers.None,MouseButton.Left));
        string selectionRevision=session!.Revision;bool selectionDirty=session.IsDirty;
        await SelectAsync(P(2,2));
        Key(Sheet,global::Avalonia.Input.Key.Right,KeyModifiers.Shift);
        Key(Sheet,global::Avalonia.Input.Key.Down,KeyModifiers.Shift);
        Check(Sheet.Anchor.Row==2 && Sheet.Anchor.Column==2 && Sheet.Selected.Row==3 && Sheet.Selected.Column==3,"Shift arrows preserve anchor and extend active corner");
        await Press(P(2,2));MovePointer(Sheet.CellBounds(P(5,4)).Center);ReleasePointer(Sheet.CellBounds(P(5,4)).Center);
        await WaitFor(()=>Sheet.Selected.Row==5&&Sheet.Selected.Column==4&&!Sheet.IsSelecting);
        await Sheet.EnsureSectionSelectionAsync();
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
        Key(Sheet,global::Avalonia.Input.Key.Escape);Check(!Sheet.IsSelecting,"Escape stops auto-scroll even with a pending viewport");await RefreshAsync();await Ready();
        Sheet.Scroll(-5000,-5000);await RefreshAsync();await Ready();await SelectAsync(P(3,3));
        Key(Sheet,global::Avalonia.Input.Key.End,KeyModifiers.Shift);await WaitFor(()=>Sheet.Selected.Column==1024);await RefreshAsync();await Ready();
        Check(Sheet.Anchor.Column==3 && Sheet.SelectionLast.Column==1024,"Shift End extends to bounded row end");
        Key(Sheet,global::Avalonia.Input.Key.Home);await WaitFor(()=>Sheet.Selected.Column==1);await RefreshAsync();await Ready();
        Check(!Sheet.HasRegion && Sheet.Selected.Row==3,"Home collapses selection at column one");
        Key(Sheet,global::Avalonia.Input.Key.End,KeyModifiers.Control);await WaitFor(()=>Sheet.Selected.Row==WorkbookCapacity.MaximumRows && Sheet.Selected.Column==1024);await RefreshAsync();await Ready();
        Check(Sheet.FirstRow+(uint)Sheet.VisibleRows-1==WorkbookCapacity.MaximumRows && Sheet.FirstColumn+(uint)Sheet.VisibleColumns-1==1024,$"Whole-sheet End reveals bounded bottom-right cell; first={Sheet.FirstRow},{Sheet.FirstColumn}; visible={Sheet.VisibleRows},{Sheet.VisibleColumns}; selected={Sheet.Selected}; ready={frameReady}; status={StatusLabel.Text}");
        Key(Sheet,global::Avalonia.Input.Key.Home,KeyModifiers.Control);await WaitFor(()=>Sheet.Selected.Row==1 && Sheet.Selected.Column==1);await RefreshAsync();await Ready();
        Key(Sheet,global::Avalonia.Input.Key.PageDown,KeyModifiers.Shift);await RefreshAsync();await Ready();
        Check(Sheet.Anchor.Row==1 && Sheet.Selected.Row==(uint)Sheet.VisibleRows+1,"Shift PageDown extends by visible rows");
        Sheet.Scroll(-5000,-5000);await RefreshAsync();await Ready();await SelectAsync(P(2,2));await SelectAsync(P(5,4),true);
        var anchorBefore=Sheet.Anchor;var activeBefore=Sheet.Selected;
        Height+=40;await Task.Delay(60);await RefreshAsync();await Ready();Height-=40;await Task.Delay(60);await RefreshAsync();await Ready();
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
        await Sheet.EnsureSectionSelectionAsync();
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
        await SelectAsync(P(12,2));await nativeClipboard.SetTextAsync("2\t=B12+1\n3\t=SUM($B$12:B13)");await ClipboardAsync(true);
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
        await session.ApplyAsync(session.Revision,[new Edit{At=P(12,2),Clear=true},new Edit{At=P(12,3),Clear=true},new Edit{At=P(13,2),Clear=true},new Edit{At=P(13,3),Clear=true},new Edit{At=P(15,5),Clear=true},new Edit{At=P(15,6),Clear=true},new Edit{At=P(16,5),Clear=true},new Edit{At=P(16,6),Clear=true}]);await RefreshAsync();await Ready();
        // Deterministic delayed clipboard reads exercise the production async path.
        await SelectAsync(P(8,4));
        var delayed=new TaskCompletionSource<string?>(TaskCreationOptions.RunContinuationsAsynchronously);
        string beforeLate=session!.Revision;var late=ClipboardAsync(true,()=>delayed.Task);
        await SelectAsync(P(8,5));await SelectAsync(P(8,4));delayed.SetResult("42");await late;
        Check(session.Revision==beforeLate && draftCell is null,"Away-and-back selection cancels late paste");
        var cancelled=new TaskCompletionSource<string?>(TaskCreationOptions.RunContinuationsAsynchronously);
        var pending=ClipboardAsync(true,()=>cancelled.Task);Key(Sheet,global::Avalonia.Input.Key.Escape);
        cancelled.SetResult("42");await pending;Check(session.Revision==beforeLate,"Escape cancels clipboard read without edit");
        await nativeClipboard.SetTextAsync("12.50");await ClipboardAsync(true);
        Check(draftCell is null && WorkbookSession.Decode(Sheet.CellAt(P(8,4))!.Value)==12.50m,"Numeric paste commits exact decimal immediately");
        await nativeClipboard.SetTextAsync("14");string keyPasteRevision=session.Revision;
        var pasteKey=new KeyEventArgs{RoutedEvent=InputElement.KeyDownEvent,Key=global::Avalonia.Input.Key.V,KeyModifiers=primary};
        Sheet.RaiseEvent(pasteKey);await WaitFor(()=>session.Revision!=keyPasteRevision && !editing && draftCell is null);
        shortcutModifiers=KeyModifiers.None;
        Check(pasteKey.Handled && WorkbookSession.Decode(Sheet.CellAt(P(8,4))!.Value)==14m,"Primary paste shortcut commits immediately");
        await EditAsync(new Edit{At=P(8,5),Formula="=D8+1"});
        await nativeClipboard.SetTextAsync("7");await ClipboardAsync(true);
        Check(WorkbookSession.Decode(Sheet.CellAt(P(8,5))!.Value)==8m,"Direct paste recalculates dependent");
        string numericRevision=session.Revision;
        await nativeClipboard.SetTextAsync("0.1234567890123456789012345678901");await ClipboardAsync(true);
        Check(session.Revision==numericRevision && draftCell is not null && CellEditor.IsVisible,"Rejected exact numeric paste keeps correctable draft");CancelDraft();
        await nativeClipboard.SetTextAsync("=(");await ClipboardAsync(true);
        Check(draftCell is null && Sheet.CellAt(P(8,4))!.Error.Length>0,"Invalid formula paste commits editable Rust formula draft");
        await nativeClipboard.SetTextAsync("");await ClipboardAsync(true);
        Check(draftCell is null && Sheet.CellAt(P(8,4))!.ContentCase==Cell.ContentOneofCase.Blank,"Empty paste clears cell immediately");
        await ClipboardAsync(false);await AutomationExpectClipboardTextAsync(nativeClipboard,"","Copying a blank cell clears stale clipboard text");
        string beforeCancelledRead=session.Revision;
        await ClipboardAsync(true,()=>Task.FromCanceled<string?>(new CancellationToken(true)));
        Check(session.Revision==beforeCancelledRead && draftCell is null,"Cancelled clipboard operation causes no edit");
        var supersededRead=new TaskCompletionSource<string?>(TaskCreationOptions.RunContinuationsAsynchronously);
        var firstPaste=ClipboardAsync(true,()=>supersededRead.Task);
        await nativeClipboard.SetTextAsync("13");await ClipboardAsync(true);string lastPasteRevision=session.Revision;
        supersededRead.SetResult("999");await firstPaste;
        Check(session.Revision==lastPasteRevision && WorkbookSession.Decode(Sheet.CellAt(P(8,4))!.Value)==13m,"New paste supersedes old read without duplicate commit");
        // Publish and verify the source before making the displayed frame stale.
        // Native clipboard awaits must not give a pending viewport refresh time
        // to turn this into an ordinary, current-frame paste.
        await nativeClipboard.SetTextAsync("44");
        await AutomationExpectClipboardTextAsync(nativeClipboard,"44","Stale-view clipboard source is published");
        await AutomationWaitForGeometryFrameAsync();
        var newerRead=new TaskCompletionSource<string?>(TaskCreationOptions.RunContinuationsAsynchronously);
        var obsolete=ClipboardAsync(true,()=>newerRead.Task);
        await session.ApplyAsync(session.Revision,[new Edit{At=P(8,4),Number=WorkbookSession.Encode(99m)}]);
        string newerRevision=session.Revision;newerRead.SetResult("42");await obsolete;
        Check(session.Revision==newerRevision,"Changed revision rejects late clipboard data");
        // Session is newer than the displayed frame: do not silently overwrite it.
        Check(displayedRevision!=newerRevision,"Stale-view fixture retains an older displayed frame before paste");
        await ClipboardAsync(true);
        Check(session.Revision==newerRevision && draftCell is null,$"Stale displayed revision refuses before dispatch without replay; revisionUnchanged={session.Revision==newerRevision}; draft={draftCell is not null}; frame={frameReady}; displayedCurrent={displayedRevision==session.Revision}; status={StatusLabel.Text}");
        await AutomationExpectClipboardTextAsync(nativeClipboard,"44","Refused stale-view paste retains clipboard source");CancelDraft();
        await SelectAsync(P(8,4));BeginCellEdit(false);CellEditor.SelectAll();string editorRevision=session.Revision;
        Sheet.Scroll(20);await RefreshAsync();await Ready();
        Check(!CellEditor.IsVisible && draftCell is not null,"Scrolled-away editor retains draft context");
        // Native focus changes on hiding may collapse selection before paste starts.
        string hiddenText=CellEditor.Text ?? "";
        int hiddenStart=Math.Min(CellEditor.SelectionStart,CellEditor.SelectionEnd), hiddenEnd=Math.Max(CellEditor.SelectionStart,CellEditor.SelectionEnd);
        await nativeClipboard.SetTextAsync("55");await ClipboardAsync(true);
        Check(session.Revision==editorRevision && draftCell is not null && FormulaEntry.Text==hiddenText[..hiddenStart]+"55"+hiddenText[hiddenEnd..],"Hidden active editor paste respects its current caret without committing");
        Sheet.Scroll(-20);await RefreshAsync();await Ready();CancelDraft();
        FormulaEntry.Focus();FormulaEntry.SelectAll();
        await nativeClipboard.SetTextAsync("66");await ClipboardAsync(true);
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
        CellEditor.Text="12.50"; await RefreshAsync();await Ready();
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
        Check(Sheet.Selected.Column==1024 && Sheet.FirstColumn+(uint)Sheet.VisibleColumns-1==1024,"Keyboard navigation reveals and bounds edge");
        await SelectAsync(P(1,1)); Sheet.Scroll(-5000,-5000);await RefreshAsync();await Ready();

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
        var superseded = RefreshAsync(); Sheet.Scroll(20); await RefreshAsync();await Ready(); await superseded;
        Check(Sheet.FirstRow==21 && Sheet.CellAt(P(1,3)) is null,"UI scroll bounded frame");
        Sheet.Scroll(-20); await RefreshAsync();await Ready(); Check(Value(1,3)==100m,"UI scroll back");
        Sheet.Scroll(100000,5000); await RefreshAsync();await Ready();
        Check(Sheet.FirstRow+(uint)Sheet.VisibleRows-1==WorkbookCapacity.MaximumRows && Sheet.FirstColumn+(uint)Sheet.VisibleColumns-1==1024,"UI scroll respects worksheet bounds");
        Sheet.Scroll(-100000,-5000); await RefreshAsync();await Ready(); Check(Value(1,3)==100m,"UI return from worksheet edge");
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
        await session.NewAsync(session.Revision);await RefreshAsync();await Ready();Check(Sheet.CellAt(P(1,3)) is null,"UI New");
        await OpenDocumentAsync(savedPath);Check(Value(1,3)==100m && session.DocumentId==savedDocument && !session.IsDirty,"UI Open saved workbook");
        Check(Value(6,9)==3686m,"UI WB3 SUM reopen");
        await Enter(2,1,"6");Check(session.IsDirty,"Reopened editing marks dirty");
        Check(await SaveDocumentAsync(false),"UI Save existing workbook: "+StatusLabel.Text);
        File.Copy(savedPath,Path.ChangeExtension(report,".pivot"),overwrite:true);
        using (var image = new RenderTargetBitmap(new PixelSize((int)Bounds.Width, (int)Bounds.Height))) {
            image.Render(this); image.Save(Path.ChangeExtension(report,".png"), PngBitmapEncoderOptions.Default);
        }
        return timings;
    }
}
