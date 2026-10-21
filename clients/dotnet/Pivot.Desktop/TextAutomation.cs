using System;
using System.IO;
using System.Threading.Tasks;
using Avalonia.Input;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
namespace Pivot.Desktop;
public partial class MainWindow
{
    private async Task TextAutomationAsync(string report)
    {
        var s=session!;
        await s.NewAsync(s.Revision,discard:true);await RefreshAsync();
        async Task Enter(uint r,uint c,string text) {
            await AutomationWaitAsync(()=>frameReady && displayedRevision==s.Revision && !editing && !fileAction);
            AutomationCheck(await SelectAsync(AutomationPosition(r,c)),"Text fixture selection commits previous edit");
            await AutomationWaitAsync(()=>frameReady && displayedRevision==s.Revision && !editing && !fileAction);
            Sheet.RaiseEvent(new TextInputEventArgs { RoutedEvent=InputElement.TextInputEvent, Text=text });
            AutomationCheck(draftCell is not null && inlineEditing,"Text input begins inline draft at "+r+","+c+"; ready="+frameReady+"; editing="+editing+"; status="+StatusLabel.Text);
            CellEditor.RaiseEvent(new KeyEventArgs { RoutedEvent=InputElement.KeyDownEvent,Key=Key.Enter });
            await AutomationWaitAsync(()=>!editing && draftCell is null && Sheet.Selected.Equals(AutomationPosition(r+1,c)));
        }
        await Enter(1,1,"Invoice 🧾");await Enter(1,2,"'001.20");await Enter(1,3,"''Hello");
        await Enter(2,1,"2");await Enter(2,2,"=A2+1");await Enter(2,3,"=SUM(A2:A3)");
        AutomationCheck(Sheet.CellAt(AutomationPosition(1,1))!.Text=="Invoice 🧾" && Sheet.CellAt(AutomationPosition(1,1))!.Error=="","Labels render as valid text content");
        await SelectAsync(AutomationPosition(1,2));BeginCellEdit(false);
        AutomationCheck(CellEditor.Text=="'001.20","Text edit reconstructs escape");
        string unchanged=s.Revision;AutomationCheck(await CommitAsync() && s.Revision==unchanged,"No-op text edit stays unchanged");
        await Enter(2,1,"Label");
        AutomationCheck(Sheet.CellAt(AutomationPosition(2,2))!.Error.Contains("Nonnumeric") && Sheet.CellAt(AutomationPosition(2,3))!.Error.Contains("Nonnumeric"),"Scalar and SUM text type errors");
        await SelectAsync(AutomationPosition(1,1));await SelectAsync(AutomationPosition(2,3),true);await ClipboardAsync(false);
        await SelectAsync(AutomationPosition(5,1));await ClipboardAsync(true);
        AutomationCheck(Sheet.CellAt(AutomationPosition(5,2))!.Text=="001.20" && Sheet.CellAt(AutomationPosition(5,3))!.Text=="'Hello","Mixed native rectangle preserves text");
        await SelectAsync(AutomationPosition(8,1));await ClipboardAsync(true,()=>Task.FromResult<string?>("'001.20\t''Hello\t'=A1"));
        AutomationCheck(Sheet.CellAt(AutomationPosition(8,3))!.Text=="=A1","External TSV explicit text escapes");
        await SelectAsync(AutomationPosition(8,1));BeginCellEdit(false);CellEditor.Text="'Updated label";
        AutomationCheck(await CommitAsync(),"Edit existing text");
        string root=Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT") ?? Path.GetTempPath();Directory.CreateDirectory(root);
        string path=Path.Combine(root,"pivot-text-ui-"+Guid.NewGuid()+".pivot");
        await s.SaveAsync(s.Revision,path);await s.OpenAsync(s.Revision,path,discard:true);await RefreshAsync();
        AutomationCheck(Sheet.CellAt(AutomationPosition(8,1))!.Text=="Updated label" && !s.IsDirty,"Text UI save/reopen");
        await SelectAsync(AutomationPosition(1,1));AutomationImage(Path.ChangeExtension(report,"text.png"));
        await s.NewAsync(s.Revision,discard:true);CancelDraft();clipboardEditor=null;shortcutModifiers=KeyModifiers.None;Sheet.Focus();await RefreshAsync();
    }
}
