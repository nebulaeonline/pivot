using System;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.VisualTree;
using Pivot.Client;

namespace Pivot.Desktop;
public partial class MainWindow {
    private async Task BoundAutomationAsync(string report) {
        var s=session!;
        await s.NewAsync(s.Revision,discard:true);await RefreshAsync();
        string source=s.WorksheetId;
        await s.ApplyAsync(s.Revision,new[]{
            new Pivot.Protocol.Application.V1.Edit{At=AutomationPosition(1,1),Entry="2"},
            new Pivot.Protocol.Application.V1.Edit{At=AutomationPosition(3,1),Entry="5"},
        });
        await RefreshAsync();
        BindWindowButton.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
        await AutomationWaitAsync(()=>OwnedWindows.OfType<BoundWindowDialog>().Any());
        var dialog=OwnedWindows.OfType<BoundWindowDialog>().Single();
        dialog.NameEditor.Text="Live Items";dialog.First.Text="A1";dialog.Last.Text="A3";
        await AutomationWaitAsync(()=>dialog.Create.IsEnabled&&dialog.Preview.Text!.Contains("3 × 1"));
        AutomationCheck(dialog.Preview.Text!.Contains("3 × 1")&&dialog.Create.IsEnabled,"bound dialog previews physical geometry");
        dialog.Create.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
        await AutomationWaitAsync(()=>s.IsReadOnly&&frameReady&&!fileAction);
        string window=s.WorksheetId;
        AutomationCheck(Sheet.RowLimit==3&&Sheet.CellAt(AutomationPosition(2,1))!.Result.KindCase==Pivot.Protocol.Application.V1.TypedValue.KindOneofCase.Blank,"bound view keeps source Blank positions");
        AutomationCheck(!FormulaEntry.IsEnabled&&!ClearButton.IsEnabled&&BoundStatusLabel.Text!.Contains("Live worksheet"),"bound view exposes read-only source provenance");
        AutomationCheck(Sheet.CellAt(AutomationPosition(1,1))!.CellId.Length==0,"projected frame does not invent cell identity");
        await SwitchWorksheetAsync(source);
        await s.ApplyAsync(s.Revision,new[]{new Pivot.Protocol.Application.V1.Edit{At=AutomationPosition(1,1),Entry="9"}});await RefreshAsync();
        await SwitchWorksheetAsync(window);
        await AutomationWaitAsync(()=>frameReady&&displayedRevision==s.Revision&&Sheet.CellAt(AutomationPosition(1,1)) is not null);
        AutomationCheck(WorkbookSession.Decode(Sheet.CellAt(AutomationPosition(1,1))!.Value)==9m,"worksheet source updates without Refresh");
        await SwitchWorksheetAsync(source);
        BindWindowButton.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
        await AutomationWaitAsync(()=>OwnedWindows.OfType<BoundWindowDialog>().Any());
        dialog=OwnedWindows.OfType<BoundWindowDialog>().Single();dialog.NameEditor.Text="Snapshot Items";dialog.Mode.SelectedIndex=1;
        dialog.Create.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
        await AutomationWaitAsync(()=>s.IsReadOnly&&s.WorksheetId!=window&&frameReady&&!fileAction);
        AutomationCheck(SelectedProducer?.SelectedGeneration.Length==0,"fixture remains unavailable until explicit Refresh");
        RefreshProducerButton.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
        await AutomationWaitAsync(()=>SelectedProducer?.SelectedGeneration.Length>0&&frameReady&&!fileAction);
        AutomationCheck(WorkbookSession.Decode(Sheet.CellAt(AutomationPosition(1,1))!.Value)==1m&&Sheet.CellAt(AutomationPosition(1,2))!.Result.Text=="0012","fixture refresh retains exact identifier text");
        await SelectAsync(AutomationPosition(1,2));Sheet.Focus();
        CopyValuesButton.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
        await AutomationExpectClipboardTextAsync(Clipboard!,"'0012","explicit bound Copy Values preserves identifier text");
        string selected=SelectedProducer!.SelectedGeneration;
        string snapshotWindow=s.WorksheetId;
        BoundSourceButton.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
        await AutomationWaitAsync(()=>OwnedWindows.Any(w=>w.Title=="Bound source"));
        var properties=OwnedWindows.Single(w=>w.Title=="Bound source");
        properties.GetVisualDescendants().OfType<ComboBox>().Single().SelectedIndex=3;
        properties.GetVisualDescendants().OfType<Button>().Single(b=>Equals(b.Content,"Refresh selected fixture")).RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
        await AutomationWaitAsync(()=>!fileAction&&frameReady&&SelectedProducer?.LastOutcome=="Failed; retained snapshot");
        AutomationCheck(SelectedProducer!.SelectedGeneration==selected&&Sheet.CellAt(AutomationPosition(1,2))!.Result.Text=="0012","failed explicit refresh retains selected values and provenance");
        string path=Path.Combine(Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetDirectoryName(report)!,"bound-ui-"+Guid.NewGuid()+".pivot");
        AutomationCheck(await SaveDocumentAsync(true,path),"WB15 saves bound source snapshot");
        await s.NewAsync(s.Revision,discard:true);await RefreshAsync();await OpenDocumentAsync(path);
        await SwitchWorksheetAsync(snapshotWindow);
        await AutomationWaitAsync(()=>frameReady&&displayedRevision==s.Revision&&SelectedProducer?.RefreshStatus=="Disconnected");
        AutomationCheck(SelectedProducer!.SelectedGeneration==selected&&Sheet.CellAt(AutomationPosition(1,2))!.Result.Text=="0012","reopen preserves exact snapshot without automatic provider execution");
        await SwitchWorksheetAsync(source);
        await s.StructureAsync(s.Revision,false,true,1,3);await RefreshAsync();
        await SwitchWorksheetAsync(window);
        await AutomationWaitAsync(()=>frameReady&&SelectedProducer?.RefreshStatus=="Source unavailable");
        AutomationCheck(Sheet.CellAt(AutomationPosition(1,1)) is null,"whole-source retirement leaves no replacement projected values");
        await s.NewAsync(s.Revision,discard:true);await RefreshAsync();Sheet.Focus();
    }
}
