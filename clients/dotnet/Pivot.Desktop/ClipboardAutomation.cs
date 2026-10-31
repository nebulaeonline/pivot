using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Input.Platform;
using Avalonia.Interactivity;
using Avalonia.Media.Imaging;
using Avalonia.VisualTree;
using Pivot.Client;
using Pivot.Protocol.Application.V1;

namespace Pivot.Desktop;

public partial class MainWindow
{
    private static Position AutomationPosition(uint row,uint column)=>new(){Row=row,Column=column};
    private static void AutomationCheck(bool condition,string message)
    {
        if(!condition) throw new InvalidOperationException(message);
    }
    private static async Task AutomationWaitAsync(Func<bool> predicate)
    {
        var clock=Stopwatch.StartNew();
        while(!predicate()) {
            if(clock.Elapsed>TimeSpan.FromSeconds(10)) throw new TimeoutException("Automated UI action timed out.");
            await Task.Delay(10);
        }
    }
    private static async Task AutomationExpectClipboardTextAsync(IClipboard clipboard,string? expected,
        string message,Func<string>? state=null)
    {
        var clock=Stopwatch.StartNew();string? observed;
        do {
            observed=await clipboard.TryGetTextAsync();
            if(observed==expected)return;
            await Task.Delay(20);
        }while(clock.Elapsed<TimeSpan.FromSeconds(1));
        throw new InvalidOperationException(message+$"; expectedLength={expected?.Length}; observedLength={observed?.Length}; "+state?.Invoke());
    }
    private void AutomationImage(string path)
    {
        using var image=new RenderTargetBitmap(new PixelSize((int)Bounds.Width,(int)Bounds.Height));
        image.Render(this);image.Save(path,PngBitmapEncoderOptions.Default);
    }

    // Deterministic UI-event tests. These do not claim OS-generated keyboard input.
    private async Task ClipboardChecklistAsync(string report)
    {
        var clipboard=Clipboard ?? throw new InvalidOperationException("Clipboard unavailable.");
        // Diagnostic metadata only: never write clipboard payloads, formulas or cell values.
        string checkpoint="initial clipboard read";
        void Trace(string phase)
        {
            File.AppendAllText(Path.ChangeExtension(report,"clipboard.log"),
                $"{checkpoint}: {phase}; session={session is not null}; editing={editing}; draft={draftCell is not null}; editor={clipboardEditor is not null}; frame={frameReady}; fileAction={fileAction}; shuttingDown={shuttingDown}\n");
        }
        async Task ClipboardStepAsync(bool paste,string name)
        {
            checkpoint=name;Trace("begin");
            await ClipboardAsync(paste,diagnose:Trace);
            Trace("completed");
        }
        Cell RequiredCell(uint row,uint column)
        {
            var cell=Sheet.CellAt(AutomationPosition(row,column));
            if(cell is null) {
                Trace($"missing required cell r{row}c{column}");
                throw new InvalidOperationException($"Clipboard checklist missing required cell r{row}c{column} after {checkpoint}; see clipboard checkpoint log.");
            }
            return cell;
        }
        Trace("begin");
        string? previous=await clipboard.TryGetTextAsync();
        Trace("completed");
        try {
            await SelectAsync(AutomationPosition(2,2));
            await clipboard.SetTextAsync("2\t=B2+1\n3\t=SUM($B$2:B3)");await ClipboardStepAsync(true,"01-paste");
            await SelectAsync(AutomationPosition(2,2));await SelectAsync(AutomationPosition(3,3),true);
            await ClipboardStepAsync(false,"02-copy");await SelectAsync(AutomationPosition(5,5));await ClipboardStepAsync(true,"03-paste");
            AutomationCheck(RequiredCell(5,6).Formula=="=E5+1" && WorkbookSession.Decode(RequiredCell(6,6).Value)==18m,"Checklist native scalar/SUM translation");
            // Preserve reversed anchor orientation when the destination has exact dimensions.
            await SelectAsync(AutomationPosition(9,6));await SelectAsync(AutomationPosition(8,5),true);
            await ClipboardStepAsync(true,"04-paste");
            AutomationCheck(Sheet.Anchor.Equals(AutomationPosition(9,6)) && Sheet.Selected.Equals(AutomationPosition(8,5)),"Exact destination preserves anchor orientation");
            await SelectAsync(AutomationPosition(8,5));await SelectAsync(AutomationPosition(8,6),true);
            string before=session!.Revision;await ClipboardStepAsync(true,"05-paste");
            AutomationCheck(session.Revision==before,"Mismatched rectangle is atomic");
            await SelectAsync(AutomationPosition(12,1));await clipboard.SetTextAsync("1\n2");await ClipboardStepAsync(true,"06-paste");
            await SelectAsync(AutomationPosition(12,1));await SelectAsync(AutomationPosition(13,1),true);await ClipboardStepAsync(false,"07-copy");
            await SelectAsync(AutomationPosition(13,1));await ClipboardStepAsync(true,"08-paste");
            AutomationCheck(WorkbookSession.Decode(RequiredCell(13,1).Value)==1m && WorkbookSession.Decode(RequiredCell(14,1).Value)==2m,"UI overlap reads source snapshot");
            await SelectAsync(AutomationPosition(12,3));await clipboard.SetTextAsync("7\t\n8\t9");await ClipboardStepAsync(true,"09-paste");
            await SelectAsync(AutomationPosition(12,3));await SelectAsync(AutomationPosition(13,4),true);await ClipboardStepAsync(false,"10-copy");
            await SelectAsync(AutomationPosition(12,1));await ClipboardStepAsync(true,"11-paste");
            AutomationCheck(Sheet.CellAt(AutomationPosition(12,2)) is null || RequiredCell(12,2).ContentCase==Cell.ContentOneofCase.Blank,"Blank slot clears destination");
            // Occupied destination is required to prove clearing, rather than sparse no-op.
            await session.ApplyAsync(session.Revision,[new Edit{At=AutomationPosition(12,2),Number=WorkbookSession.Encode(99m)}]);await RefreshAsync();
            await SelectAsync(AutomationPosition(12,1));await ClipboardStepAsync(true,"12-paste");
            AutomationCheck(RequiredCell(12,2).ContentCase==Cell.ContentOneofCase.Blank,"Blank overwrites occupied destination");
            var obsoleteItem=new DataTransferItem();obsoleteItem.Set(OldCellBlock4Format,new byte[]{1});obsoleteItem.SetText("001234");
            var obsoleteTransfer=new DataTransfer();obsoleteTransfer.Add(obsoleteItem);await clipboard.SetDataAsync(obsoleteTransfer);
            string obsoleteRevision=session.Revision;await ClipboardStepAsync(true,"obsolete-cb4");
            AutomationCheck(session.Revision==obsoleteRevision && StatusLabel.Text?.Contains("Unsupported Pivot clipboard version")==true,"Obsolete CB4 is rejected without interpreting its text fallback");
            AutomationImage(Path.ChangeExtension(report,"checklist.png"));
            await session.NewAsync(session.Revision,discard:true);await RefreshAsync();
            await session.ApplyAsync(session.Revision,[new Edit{At=AutomationPosition(1,1),Number=WorkbookSession.Encode(10m)},new Edit{At=AutomationPosition(1,2),Formula="=A1+1"}]);await RefreshAsync();
            await SelectAsync(AutomationPosition(1,1));MoveTarget.Text="A5";MoveClick(null,new RoutedEventArgs());
            await AutomationWaitAsync(()=>!editing && Sheet.CellAt(AutomationPosition(5,1)) is not null);
            await SelectAsync(AutomationPosition(1,2));await ClipboardStepAsync(false,"13-copy");await SelectAsync(AutomationPosition(2,2));await ClipboardStepAsync(true,"14-paste");
            AutomationCheck(RequiredCell(2,2).Formula=="=A6+1" && RequiredCell(1,2).Formula=="=A5+1" && RequiredCell(1,2).AuthoredFormula=="=A1+1","UI move then copy uses effective binding and preserves authored provenance");
            await session.NewAsync(session.Revision,discard:true);await RefreshAsync();
            await session.ApplyAsync(session.Revision,[new Edit{At=AutomationPosition(2,2),Formula="= A2 + $A2 + A$2 + $A$2"}]);await RefreshAsync();
            await SelectAsync(AutomationPosition(2,2));await ClipboardStepAsync(false,"15-copy");await SelectAsync(AutomationPosition(5,4));await ClipboardStepAsync(true,"16-paste");
            AutomationCheck(RequiredCell(5,4).Formula=="= C5 + $A5 + C$2 + $A$2","UI all four copy-axis modes");

            // Publish bytes from a source engine that is then closed. A distinct GUI process
            // must retrieve them through the OS clipboard and bind into its own Rust child.
            string binary=RustExecutablePath();
            await using(var source=await WorkbookSession.StartAsync(binary)) {
                await source.PasteBlockAsync(source.Revision,AutomationPosition(2,2),AutomationPosition(2,2),Encoding.UTF8.GetBytes("2\t=B2+1\n3\t=SUM($B$2:B3)"),false);
                await source.ApplyAsync(source.Revision,[new Edit{At=AutomationPosition(2,4),Text="001.20"},new Edit{At=AutomationPosition(3,4),Text="Invoice"}]);
                await source.ApplyAsync(source.Revision,[new Edit{At=AutomationPosition(3,2),MoveTo=AutomationPosition(5,2)},new Edit{At=AutomationPosition(3,3),MoveTo=AutomationPosition(5,3)},new Edit{At=AutomationPosition(3,4),MoveTo=AutomationPosition(5,4)}]);
                await source.SetFilterAsync(source.Revision,new FilterDefinition{First=AutomationPosition(2,2),Last=AutomationPosition(5,4),Conditions={new FilterCondition{Column=2,Predicates={new FilterPredicate{Operator=10}}}}});
                for(int attempt=0;attempt<200;attempt++){if((await source.ReadFilteredAsync(1,10,1,5)).FilterStatus!=3)break;await Task.Delay(10);}
                var copied=await source.CopyVisibleAsync(source.Revision,AutomationPosition(2,2),AutomationPosition(5,4));
                AutomationCheck(copied.VisibleRows.SequenceEqual(new uint[]{2,5}),"cross-process image retains gapped source origins");
                var item=new DataTransferItem();item.Set(VisibleBlockFormat,copied.Clipboard.ToByteArray());item.SetText(copied.ClipboardText);
                var transfer=new DataTransfer();transfer.Add(item);await clipboard.SetDataAsync(transfer);
                await source.DisposeAsync();AutomationCheck(source.FinalCharge==0,"Clipboard retains no source engine");
            }
            checkpoint="cross-process peers";Trace("begin");
            foreach(string mode in new[]{"clipboard-save","discard","clean"}) {
                string peerReport=Path.ChangeExtension(report,mode+".json");
                string executable=Environment.ProcessPath ?? throw new IOException("Current executable path unavailable.");
                var start=new ProcessStartInfo(executable){UseShellExecute=false,CreateNoWindow=true};
                if(string.Equals(Path.GetFileNameWithoutExtension(executable),"dotnet",StringComparison.OrdinalIgnoreCase))
                    start.ArgumentList.Add(Path.Combine(AppContext.BaseDirectory,"Pivot.Desktop.dll"));
                start.Environment.Remove("PIVOT_UI_SMOKE_REPORT");
                start.Environment["PIVOT_UI_PEER_REPORT"]=peerReport;start.Environment["PIVOT_UI_PEER_MODE"]=mode;
                using var peer=Process.Start(start) ?? throw new IOException("Could not start UI peer.");
                try {await peer.WaitForExitAsync().WaitAsync(TimeSpan.FromSeconds(30));}
                catch {if(!peer.HasExited) peer.Kill(entireProcessTree:true);throw;}
                AutomationCheck(peer.ExitCode==0 && File.Exists(peerReport),"UI peer exits with evidence: "+mode);
                using var result=JsonDocument.Parse(await File.ReadAllTextAsync(peerReport));var data=result.RootElement;
                AutomationCheck(data.GetProperty("status").GetString()=="passed" && data.GetProperty("remaining_charge").GetUInt64()==0,"UI peer passes: "+data);
                AutomationCheck(data.GetProperty("process_id").GetInt32()!=Environment.ProcessId,"Clipboard peer is a separate application process");
                if(mode!="clean") {
                    await using var reader=await WorkbookSession.StartAsync(binary);
                    await reader.OpenAsync(reader.Revision,data.GetProperty("path").GetString()!,discard:true);
                    var frame=await reader.ReadAsync(AutomationPosition(25,5),AutomationPosition(25,5));
                    AutomationCheck(WorkbookSession.Decode(frame.Cells.Single().Number)==(mode=="clipboard-save"?9m:2m),"Close Save/Discard persisted the expected generation");
                    var textFrame=await reader.ReadAsync(AutomationPosition(25,7),AutomationPosition(25,7));
                    AutomationCheck(textFrame.Cells.Single().Text==(mode=="clipboard-save"?"Saved label":"Original label"),"Dirty-close text draft Save/Discard installs correct generation");
                    await reader.DisposeAsync();AutomationCheck(reader.FinalCharge==0,"Close verification releases engine");
                }
            }
            await session.NewAsync(session.Revision,discard:true);Sheet.Select(AutomationPosition(1,1));Sheet.Scroll(-5000,-5000);CancelDraft();clipboardEditor=null;shortcutModifiers=KeyModifiers.None;await RefreshAsync();
        } finally {
            checkpoint="restore clipboard";Trace("begin");
            if(previous is not null) await clipboard.SetTextAsync(previous);else await clipboard.ClearAsync();
        }
    }

    private async Task ArrowEditAutomationAsync()
    {
        void SendKey(Control target,Key key,KeyModifiers modifiers=KeyModifiers.None)=>target.RaiseEvent(new KeyEventArgs {RoutedEvent=InputElement.KeyDownEvent,Key=key,KeyModifiers=modifiers});
        foreach(var (key,row,column) in new[]{(Key.Up,5u,6u),(Key.Down,7u,6u),(Key.Left,6u,5u),(Key.Right,6u,7u)}) {
            await SelectAsync(AutomationPosition(6,6));BeginCellEdit(true,"12.50");
            SendKey(CellEditor,key);await AutomationWaitAsync(()=>!editing && draftCell is null && Sheet.Selected.Equals(AutomationPosition(row,column)));
            AutomationCheck(WorkbookSession.Decode(Sheet.CellAt(AutomationPosition(6,6))!.Number)==12.50m,"Arrow commits exact input: "+key);
            await SelectAsync(AutomationPosition(6,6));BeginCellEdit(true,"=1+2");
            SendKey(CellEditor,key);await AutomationWaitAsync(()=>!editing && draftCell is null && Sheet.Selected.Equals(AutomationPosition(row,column)));
            AutomationCheck(WorkbookSession.Decode(Sheet.CellAt(AutomationPosition(6,6))!.Value)==3m,"Arrow commits formula: "+key);
        }
        await SelectAsync(AutomationPosition(1,1));BeginCellEdit(true,"7");SendKey(CellEditor,Key.Left);
        await AutomationWaitAsync(()=>!editing && draftCell is null);
        AutomationCheck(Sheet.Selected.Equals(AutomationPosition(1,1)),"Arrow commits at worksheet boundary without leaving bounds");
        await SelectAsync(AutomationPosition(6,6));BeginCellEdit(true,"0.1234567890123456789012345678901");
        string before=session!.Revision;SendKey(CellEditor,Key.Right);await AutomationWaitAsync(()=>!editing);
        AutomationCheck(session.Revision==before && draftCell is not null && Sheet.Selected.Equals(AutomationPosition(6,6)),"Rejected numeric arrow commit retains draft and selection");CancelDraft();
        BeginCellEdit(false);before=session.Revision;SendKey(CellEditor,Key.Left,KeyModifiers.Shift);await Task.Yield();
        AutomationCheck(session.Revision==before && draftCell is not null,"Modified arrows stay in text editor");CancelDraft();
        FormulaEntry.Focus();FormulaEntry.Text="19";before=session.Revision;SendKey(FormulaEntry,Key.Right);await Task.Yield();
        AutomationCheck(session.Revision==before && draftCell is not null,"Formula bar arrows retain normal text editing");CancelDraft();
        await session.NewAsync(session.Revision,discard:true);await RefreshAsync();
        await session.ApplyAsync(session.Revision,[new Edit {At=AutomationPosition(1,2),Formula="=A1+1"},new Edit{At=AutomationPosition(1,1),MoveTo=AutomationPosition(5,1)}]);await RefreshAsync();
        await SelectAsync(AutomationPosition(1,2));BeginCellEdit(false);before=session.Revision;SendKey(CellEditor,Key.Down);
        await AutomationWaitAsync(()=>draftCell is null && Sheet.Selected.Equals(AutomationPosition(2,2)));
        AutomationCheck(session.Revision==before,"Unchanged F2 arrow exit does not rebind historical formula source");
        await session.NewAsync(session.Revision,discard:true);Sheet.Select(AutomationPosition(1,1));Sheet.Scroll(-5000,-5000);CancelDraft();clipboardEditor=null;shortcutModifiers=KeyModifiers.None;await RefreshAsync();
    }

    private async Task ClipboardPeerAsync(string report,string mode)
    {
        AutomationCheck(mode is "clipboard-save" or "discard" or "clean","Known peer scenario required");
        string root=Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT") ?? Path.GetDirectoryName(report)!;
        Directory.CreateDirectory(root);string path=Path.Combine(root,"close-"+Guid.NewGuid()+".pivot");
        bool cancelled=false,nativeReceived=false;
        if(mode!="clean") {
            if(mode=="clipboard-save") {
                using(var data=await Clipboard!.TryGetDataAsync()) nativeReceived=data is not null && data.Contains(VisibleBlockFormat);
                AutomationCheck(nativeReceived,"Separate process receives private OS format");
                await SelectAsync(AutomationPosition(25,5));await ClipboardAsync(true);
                AutomationCheck(Sheet.CellAt(AutomationPosition(25,6))!.Formula=="=E25+1" && Sheet.CellAt(AutomationPosition(26,6))!.Formula=="=SUM($B$2:E24)","Cross-process effective scalar/SUM bindings");
            } else {
                await EditAsync(new Edit{At=AutomationPosition(25,5),Number=WorkbookSession.Encode(2m)});
            }
            if(mode=="clipboard-save") AutomationCheck(Sheet.CellAt(AutomationPosition(25,7))!.Text=="001.20" && Sheet.CellAt(AutomationPosition(26,7))!.Text=="Invoice","Native text crosses actual process clipboard");
            else await EditAsync(new Edit{At=AutomationPosition(25,7),Text="Original label"});
            AutomationCheck(await SaveDocumentAsync(true,path),"Peer native baseline Save As");
            await EditAsync(new Edit{At=AutomationPosition(25,5),Number=WorkbookSession.Encode(9m)});
            await SelectAsync(AutomationPosition(25,7));BeginCellEdit(true,mode=="clipboard-save"?"Saved label":"Discarded label");
            if(mode=="clipboard-save") {
                AutomationCheck(WorkbookSession.Decode(Sheet.CellAt(AutomationPosition(25,6))!.Value)==10m,"Cross-process binding follows destination edits");
                Close();await ClickCloseChoiceAsync("Cancel");await AutomationWaitAsync(()=>!shuttingDown);
                AutomationCheck(IsVisible && session!.IsDirty,"Actual close Cancel retains dirty window");cancelled=true;
            }
        }
        AutomationImage(Path.ChangeExtension(report,"png"));
        string? peerGraphicalBackend=PlatformImpl?.GetType().FullName;
        Closed+=(_,_)=>{
            bool pass=session?.FinalCharge==0 && OwnedWindows.Count==0;
            File.WriteAllText(report,JsonSerializer.Serialize(new PeerReport {graphical_backend=peerGraphicalBackend,status=pass?"passed":"failed",mode=mode,process_id=Environment.ProcessId,rust_process_id=session?.ProcessId,remaining_charge=session?.FinalCharge,path=path,cancel_checked=cancelled,native_received=nativeReceived,native_format=nativeReceived?"CB6":null,dialog_input="programmatic Button.Click routed event",os_keyboard_input=false},ReportJsonContext.Default.PeerReport));
        };
        Close();
        if(mode!="clean") await ClickCloseChoiceAsync(mode=="clipboard-save"?"Save":"Discard");
    }
    private async Task ClickCloseChoiceAsync(string choice)
    {
        await AutomationWaitAsync(()=>OwnedWindows.OfType<Window>().Any(w=>w.Title=="Unsaved changes"));
        var dialog=OwnedWindows.OfType<Window>().Single(w=>w.Title=="Unsaved changes");
        var button=dialog.GetVisualDescendants().OfType<Button>().Single(b=>Equals(b.Content,choice));
        button.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
    }
}
