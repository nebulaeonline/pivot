using System;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Input;
using Avalonia.Interactivity;
using Pivot.Client;
using Pivot.Protocol.Application.V1;

namespace Pivot.Desktop;
public partial class MainWindow {
    private async Task WorksheetAutomationAsync(string report) {
        var s=session!;
        async Task Ready()=>await AutomationWaitAsync(()=>frameReady&&!editing&&!fileAction&&displayedRevision==s.Revision);
        async Task Enter(uint row,uint col,string text) {
            await Ready();AutomationCheck(await SelectAsync(AutomationPosition(row,col)),"worksheet editor selects cell");
            BeginCellEdit(true,text);CellEditor.RaiseEvent(new KeyEventArgs{RoutedEvent=KeyDownEvent,Key=Key.Enter});
            await AutomationWaitAsync(()=>!editing&&draftCell is null);await RefreshAsync();
        }
        async Task Tab(string id) {
            await AutomationWaitAsync(()=>frameReady&&!fileAction&&displayedRevision==s.Revision);
            var button=WorksheetTabs.Children.OfType<ToggleButton>().Single(b=>Equals(b.Tag,id));
            button.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
            await AutomationWaitAsync(()=>s.WorksheetId==id&&!fileAction&&frameReady);await RefreshAsync();
        }
        string summary=s.WorksheetId;AutomationCheck(WorksheetTabs.Children.Count==1&&!s.IsDirty,"one clean initial tab");
        AddWorksheetButton.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
        await AutomationWaitAsync(()=>s.Worksheets.Count==2&&s.WorksheetId!=summary&&!fileAction&&frameReady);
        string revenue=s.WorksheetId;
        WorksheetNameEditor.Text="Revenue";RenameWorksheetButton.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
        await AutomationWaitAsync(()=>s.Worksheets.Any(w=>w.Id==revenue&&w.Name=="Revenue")&&!fileAction);
        await Enter(1,1,"10");await Enter(2,1,"20");
        await SelectAsync(AutomationPosition(3,1));BeginCellEdit(true,"5");
        await Tab(summary);AutomationCheck(draftCell is null,"switch commits old-sheet edit");
        await Enter(1,1,"=SUM(Revenue!A1:A3,1)");await SelectAsync(AutomationPosition(1,1));
        AutomationCheck(WorkbookSession.Decode(Sheet.SelectedCell!.Value)==36,"cross-sheet aggregate in grid");
        await Tab(revenue);WorksheetNameEditor.Text="O'Brien Sales";
        RenameWorksheetButton.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
        await AutomationWaitAsync(()=>s.Worksheets.Any(w=>w.Id==revenue&&w.Name=="O'Brien Sales")&&!fileAction);
        await Tab(summary);await SelectAsync(AutomationPosition(1,1));
        AutomationCheck(FormulaEntry.Text=="=SUM('O''Brien Sales'!A1:A3,1)"&&Sheet.SelectedCell!.AuthoredFormula=="=SUM(Revenue!A1:A3,1)","quoted effective name and original provenance");
        await SelectAsync(AutomationPosition(24,4));await SelectAsync(AutomationPosition(25,6),true);Sheet.Scroll(3,1);await RefreshAsync();
        var anchor=Sheet.Anchor;var active=Sheet.Selected;uint row=Sheet.FirstRow,col=Sheet.FirstColumn;
        var older=RefreshAsync();await Tab(revenue);await older;
        AutomationCheck(Sheet.CellAt(AutomationPosition(1,1))?.Worksheet==revenue,"old sheet frame does not paint new tab");
        await Tab(summary);AutomationCheck(Sheet.Anchor.Equals(anchor)&&Sheet.Selected.Equals(active)&&Sheet.FirstRow==row&&Sheet.FirstColumn==col,"per-sheet selection and scroll restored");
        await SelectAsync(AutomationPosition(1,1));
        string path=Path.Combine(Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetDirectoryName(report)!,"worksheets-ui-"+Guid.NewGuid()+".pivot");
        AutomationCheck(await SaveDocumentAsync(true,path),"multi-sheet UI save");string savedRevision=s.Revision;
        await Tab(revenue);await Tab(summary);AutomationCheck(!s.IsDirty&&s.Revision==savedRevision,"tab switches leave saved workbook clean");
        await s.NewAsync(s.Revision,discard:true);await RefreshAsync();await OpenDocumentAsync(path);
        AutomationCheck(s.WorksheetId==summary&&s.Worksheets.Count==2&&!s.IsDirty&&WorkbookSession.Decode(Sheet.CellAt(AutomationPosition(1,1))!.Value)==36,"all sheets reopened through desktop");
        AutomationImage(Path.ChangeExtension(report,"worksheets.png"));
        await s.NewAsync(s.Revision,discard:true);CancelDraft();clipboardEditor=null;shortcutModifiers=KeyModifiers.None;await RefreshAsync();Sheet.Focus();
    }
}
