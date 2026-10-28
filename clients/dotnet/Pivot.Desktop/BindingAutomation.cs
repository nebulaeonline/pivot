using System;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Input.Platform;
using Avalonia.Interactivity;
using Avalonia.VisualTree;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
namespace Pivot.Desktop;
public partial class MainWindow {
    private async Task BindingAutomationAsync(string report){
        var s=session!;await s.NewAsync(s.Revision,discard:true);await s.StartBindingsAsync();await RefreshAsync();SetEditing(true);
        await AutomationWaitAsync(()=>frameReady&&!editing&&displayedRevision==s.Revision);
        AutomationCheck(await SelectAsync(AutomationPosition(1,1)),"Binding editor selects A1");
        Sheet.RaiseEvent(new TextInputEventArgs{RoutedEvent=InputElement.TextInputEvent,Text="=@{InvoiceLines.TotalAmount}*1.1+B1"});
        AutomationCheck(draftCell is not null&&inlineEditing,"Binding editor starts after layout refresh");
        CellEditor.RaiseEvent(new KeyEventArgs{RoutedEvent=InputElement.KeyDownEvent,Key=Key.Enter});
        await AutomationWaitAsync(()=>!editing&&draftCell is null&&Sheet.Selected.Equals(AutomationPosition(2,1)));
        var first=(await s.ReadAsync(AutomationPosition(1,1),AutomationPosition(1,1))).Cells.Single();
        AutomationCheck(WorkbookSession.Decode(first.Value)==66 && first.SourceProducer.Length>0,"Inline editor creates real stable structured operand");
        string definition=s.Revision;await s.PublishProofAsync(5);await RefreshAsync();
        var current=(await s.ReadAsync(AutomationPosition(1,1),AutomationPosition(1,1))).Cells.Single();
        AutomationCheck(s.Revision==definition&&current.CellId==first.CellId&&WorkbookSession.Decode(current.Value)==165,"Publication changes value without rewriting ordinary formula or identity");
        await s.RenameProducerAsync("RenamedLines");await RefreshAsync();await SelectAsync(AutomationPosition(1,1));
        string unchanged=s.Revision;BeginCellEdit(false);AutomationCheck(await CommitAsync()&&s.Revision==unchanged,"Unchanged F2 exit after rename preserves authoritative binding");
        var clipboard=Clipboard ?? throw new InvalidOperationException("Clipboard unavailable");
        string? previousClipboard=await clipboard.TryGetTextAsync();
        try {await clipboard.SetTextAsync("binding-copy-sentinel");await ClipboardAsync(false);
            AutomationCheck(await clipboard.TryGetTextAsync()=="binding-copy-sentinel","Unsupported OS Copy leaves clipboard unchanged");
        }finally{if(previousClipboard is null)await clipboard.ClearAsync();else await clipboard.SetTextAsync(previousClipboard);}
        await s.DuplicateBlockAsync(AutomationPosition(1,1),AutomationPosition(1,1),AutomationPosition(12,2),AutomationPosition(12,2));
        await s.ApplyAsync(s.Revision,[new Edit{At=AutomationPosition(12,3),Number=WorkbookSession.Encode(7)}]);
        var copy=(await s.ReadAsync(AutomationPosition(12,2),AutomationPosition(12,2))).Cells.Single();
        AutomationCheck(WorkbookSession.Decode(copy.Value)==172&&copy.SourceProducer==first.SourceProducer,"Duplicate preserves renamed producer identity and translates B1 to C12");
        await s.PublishUnavailableAsync();await RefreshAsync();
        var unavailable=(await s.ReadAsync(AutomationPosition(1,1),AutomationPosition(1,1))).Cells.Single();
        AutomationCheck(unavailable.SourceFaultCode=="UnavailableResult"&&unavailable.SourceFaultId.Length>0,"Unavailable head propagates typed document fault with stable core identity");
        await s.PublishProofAsync(0);await RefreshAsync();AutomationCheck(WorkbookSession.Decode((await s.ReadAsync(AutomationPosition(1,1),AutomationPosition(1,1))).Cells.Single().Value)==0,"Valid empty result differs from unavailable");
        await s.PublishProofAsync(5);await RefreshAsync();await SelectAsync(AutomationPosition(1,1));UpdateDocument();
        AutomationImage(Path.ChangeExtension(report,"bindings.png"));
        string path=Path.Combine(Path.GetTempPath(),"pivot-binding-ui-"+Guid.NewGuid()+".pivot");
        AutomationCheck(!await SaveDocumentAsync(true,path)&&!File.Exists(path),"Bound document cannot silently lose bindings through WB4");
        async Task Choice(string choice){
            await AutomationWaitAsync(()=>OwnedWindows.OfType<Window>().Any(w=>w.Title=="Discard experimental document"));
            var dialog=OwnedWindows.OfType<Window>().Single(w=>w.Title=="Discard experimental document");
            var buttons=dialog.GetVisualDescendants().OfType<Button>().ToArray();
            AutomationCheck(buttons.Length==2&&buttons.All(b=>!Equals(b.Content,"Save")),"Binding lifecycle offers discard/cancel only");
            buttons.Single(b=>Equals(b.Content,choice)).RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
        }
        Close();await Choice("Cancel");await AutomationWaitAsync(()=>!shuttingDown);AutomationCheck(s.IsBindingProof,"Cancel retains binding document");
        CloseDocumentClick(null,new RoutedEventArgs());await Choice("Discard");await AutomationWaitAsync(()=>!fileAction&&s.DocumentId.Length==0);
        await s.NewAsync(s.Revision,discard:true);CancelDraft();clipboardEditor=null;Sheet.Scroll(1-(int)Sheet.FirstRow,1-(int)Sheet.FirstColumn);await RefreshAsync();Sheet.Focus();
    }
}
