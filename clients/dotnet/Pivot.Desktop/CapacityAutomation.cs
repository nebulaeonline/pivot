using System;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Controls;
using Pivot.Avalonia;
using Pivot.Client;
using Pivot.Protocol.Application.V1;

namespace Pivot.Desktop;
public partial class MainWindow
{
    private async Task CapacityAutomationAsync(string report)
    {
        var s=session!;
        Position P(uint row,uint column=1)=>new(){Row=row,Column=column};
        await s.NewAsync(s.Revision,discard:true);await RefreshAsync();
        AutomationCheck(s.Worksheets.Single().Rows==WorkbookCapacity.MaximumRows,"large row metadata");
        AutomationCheck(WorksheetView.ParseAddress("AMJ65536").Row==65536,"large address parser");
        async Task Enter(uint row,uint column,string text) {
            AutomationCheck(await SelectAsync(P(row,column)),"large-row selection");
            BeginCellEdit(true,text);AutomationCheck(await CommitAsync(),"large-row edit committed");await RefreshAsync();
        }
        await Enter(65535,1,"5");await Enter(65536,1,"7");
        await Enter(65535,2,"=A65535+1");await Enter(65536,3,"'001234");
        await SelectAsync(P(65535,2));await SelectAsync(P(65536,2),true);
        FillDownItem.RaiseEvent(new RoutedEventArgs(MenuItem.ClickEvent));
        await AutomationWaitAsync(()=>!editing&&frameReady&&Sheet.CellAt(P(65536,2))?.Result?.Decimal is not null);
        AutomationCheck(WorkbookSession.Decode(Sheet.CellAt(P(65536,2))!.Result.Decimal)==8,"high-row Fill translates references");
        AutomationCheck(Sheet.FirstRow+(uint)Sheet.VisibleRows-1<=65536&&Sheet.IsVisibleCell(P(65536,2)),"high-row scroll bounds");
        await SelectAsync(P(65536,3));Sheet.Focus();await ClipboardAsync(false);
        await SelectAsync(P(65535,3));Sheet.Focus();await ClipboardAsync(true);await RefreshAsync();
        AutomationCheck(Sheet.CellAt(P(65535,3))?.Result.Text=="001234","native high-row String clipboard");
        Sheet.RaiseEvent(new KeyEventArgs{RoutedEvent=KeyDownEvent,Key=Key.Home,KeyModifiers=KeyModifiers.Control});
        await AutomationWaitAsync(()=>Sheet.Selected.Row==1&&Sheet.Selected.Column==1);
        Sheet.RaiseEvent(new KeyEventArgs{RoutedEvent=KeyDownEvent,Key=Key.End,KeyModifiers=KeyModifiers.Control});
        await AutomationWaitAsync(()=>Sheet.Selected.Row==65536&&Sheet.Selected.Column==1024);
        await RefreshAsync();
        AutomationCheck(Sheet.FirstRow+(uint)Sheet.VisibleRows-1==65536,"Ctrl End reaches final row");
        var path=Path.ChangeExtension(report,"capacity.pivot");
        AutomationCheck(await SaveDocumentAsync(true,path),"large-row native save");
        await OpenDocumentAsync(path);await SelectAsync(P(65536,2));await RefreshAsync();
        AutomationCheck(WorkbookSession.Decode(Sheet.CellAt(P(65536,2))!.Result.Decimal)==8,"large-row formula reopen");
        AutomationCheck(s.PositionedCells==6&&s.AllocatedCellIdentities==6,"position and identity diagnostics");
        AutomationCheck(FormatFields.Children.OfType<TextBlock>().Any(t=>t.Text?.Contains("property regions")==true),"format-region diagnostics visible");
        AutomationImage(Path.ChangeExtension(report,"capacity.png"));
        await s.NewAsync(s.Revision,discard:true);CancelDraft();clipboardEditor=null;shortcutModifiers=KeyModifiers.None;await RefreshAsync();Sheet.Focus();
    }
}
