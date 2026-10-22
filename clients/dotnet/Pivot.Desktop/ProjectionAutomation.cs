using System;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.VisualTree;
using Pivot.Avalonia;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
namespace Pivot.Desktop;
public partial class MainWindow
{
    private async Task ProjectionAutomationAsync(string report)
    {
        var s=session!;
        await s.NewAsync(s.Revision,discard:true);
        await s.ApplyAsync(s.Revision,[new Edit{At=AutomationPosition(8,1),Number=WorkbookSession.Encode(8)},new Edit{At=AutomationPosition(9,1),Number=WorkbookSession.Encode(9)},new Edit{At=AutomationPosition(9,2),Formula="=SUM(A8:A9)"}]);
        string stable=(await s.ReadAsync(AutomationPosition(9,1),AutomationPosition(9,1))).Cells.Single().CellId;
        await s.StartProofAsync();await RefreshAsync();UpdateDocument();
        await AutomationWaitAsync(()=>frameReady && !editing);
        AutomationCheck(DocumentLabel.Text!.Contains("Cannot save"),"Explicit experimental non-savable footer");
        AutomationCheck(Sheet.VisualRow(9)==19 && (await s.ReadAsync(AutomationPosition(9,1),AutomationPosition(9,1))).Cells.Single().CellId==stable,"Bands preserve ordinary address and identity");
        await SelectAsync(AutomationPosition(8,1));await SelectAsync(AutomationPosition(9,1),true);
        await ClipboardAsync(false);
        var copied=await s.CopyBlockAsync(s.Revision,AutomationPosition(8,1),AutomationPosition(9,1));
        AutomationCheck(copied.ClipboardText.Replace("\r", "").TrimEnd('\n')=="8\n9","Logical rectangle crossing bands excludes projected records");
        await SelectAsync(AutomationPosition(8,1));
        Sheet.RaiseEvent(new global::Avalonia.Input.KeyEventArgs{RoutedEvent=global::Avalonia.Input.InputElement.KeyDownEvent,Key=global::Avalonia.Input.Key.Down});
        await AutomationWaitAsync(()=>Sheet.Selected.Equals(AutomationPosition(9,1)));
        Sheet.Scroll(1-(int)Sheet.FirstRow,1-(int)Sheet.FirstColumn);await RefreshAsync();
        var frame=Sheet.ProjectionFrame!;
        var row=frame.Rows.Single(r=>r.Projection==frame.Placements[0].Id && r.Key=="L2");
        var target=new ProjectionTarget(row.Projection,frame.Producer,frame.Generation,row.Key,frame.FieldIds[0],frame.Epoch,frame.Presentation);
        AutomationCheck(Sheet.ApplyProjectionTarget(target),"Select keyed L2 record");
        string revision=s.Revision;BeginCellEdit(true,"999");await ClipboardAsync(true,()=>Task.FromResult<string?>("999"));
        AutomationCheck(draftCell is null && s.Revision==revision,"Projected data rejects editor and paste");
        // Hold an already-read target across publication: the same visual ordinal now belongs to L3.
        var release=new TaskCompletionSource<bool>();
        async Task<bool> LateTarget(){await release.Task;return Sheet.ApplyProjectionTarget(target);}
        var late=LateTarget();await s.PublishProofAsync(3,"L2");await RefreshAsync();release.SetResult(true);
        AutomationCheck(!await late && Sheet.MissingProjectedRecord && Sheet.ProjectedSelection?.Key=="","Disappearing L2 selects header; delayed target never selects L3");
        await s.PublishProofAsync(5);await RefreshAsync();
        AutomationCheck(Sheet.VisualRow(9)==23,"Five-record flow repositions presentation only");
        var cap=await s.CaptureAsync();var total=(await s.EvaluateProofAsync(cap.Capture)).ProofValue;
        AutomationCheck(WorkbookSession.Decode(total.Total)==150 && WorkbookSession.Decode(total.Taxed)==165,"Checked total feeds existing numeric invocation");await s.ReleaseAsync(cap.Capture);
        frame=Sheet.ProjectionFrame!;await s.LayoutProofAsync(frame.Placements[0].Id,4,true);await RefreshAsync();
        AutomationCheck(Sheet.ProjectionFrame!.Placements[0].Reverse && Sheet.VisualRow(9)==23,"Independent projection layout preserves ordinary axes");
        string path=Path.Combine(Path.GetTempPath(),"pivot-proof-ui-"+Guid.NewGuid()+".pivot");
        AutomationCheck(!await SaveDocumentAsync(true,path) && !File.Exists(path),"Save refuses experimental document without creating a file");
        AutomationImage(Path.ChangeExtension(report,"projections.png"));
        async Task Choice(string choice){
            await AutomationWaitAsync(()=>OwnedWindows.OfType<Window>().Any(w=>w.Title=="Discard experimental document"));
            var dialog=OwnedWindows.OfType<Window>().Single(w=>w.Title=="Discard experimental document");
            var buttons=dialog.GetVisualDescendants().OfType<Button>().ToArray();
            AutomationCheck(buttons.Length==2 && buttons.All(b=>!Equals(b.Content,"Save")),"Experimental dialog offers only discard/cancel");
            buttons.Single(b=>Equals(b.Content,choice)).RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
        }
        Close();await Choice("Cancel");await AutomationWaitAsync(()=>!shuttingDown);
        AutomationCheck(s.IsProjectionProof,"Cancel close retains proof");
        CloseDocumentClick(null,new RoutedEventArgs());await Choice("Discard");
        await AutomationWaitAsync(()=>!fileAction && s.DocumentId.Length==0);
        AutomationCheck(!s.IsProjectionProof,"Discard releases experimental document");
        await s.NewAsync(s.Revision,discard:true);CancelDraft();clipboardEditor=null;Sheet.Scroll(1-(int)Sheet.FirstRow,1-(int)Sheet.FirstColumn);await RefreshAsync();Sheet.Focus();
    }
}
