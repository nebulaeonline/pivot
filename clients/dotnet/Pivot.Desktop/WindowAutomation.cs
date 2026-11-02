using System;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Input;
using Avalonia.Input.Platform;
using Avalonia.VisualTree;
using Pivot.Client;
namespace Pivot.Desktop;
public partial class MainWindow {
 private async Task WindowAutomationAsync(string report){
  var s=session!;await s.NewAsync(s.Revision,discard:true);await RefreshAsync();
  string sheet=s.WorksheetId;
  WindowNameEditor.Text="Line Items";WindowRows.Value=100;WindowColumns.Value=10;
  CreateWindowButton.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
  await AutomationWaitAsync(()=>s.IsWindowScope&&s.Windows.Count==1&&frameReady&&!fileAction);
  string id=s.WorksheetId;
  AutomationCheck(Sheet.RowLimit==100&&Sheet.ColumnLimit==10&&WindowTabs.Children.Count==1,"independent Grid navigation and dimensions");
  async Task Enter(uint row,string text){
   await AutomationWaitAsync(()=>frameReady&&!editing&&!fileAction&&displayedRevision==s.Revision);
   AutomationCheck(await SelectAsync(AutomationPosition(row,1)),"Window editor selects cell");
   BeginCellEdit(true,text);CellEditor.RaiseEvent(new KeyEventArgs{RoutedEvent=KeyDownEvent,Key=Key.Enter});
   await AutomationWaitAsync(()=>!editing&&draftCell is null);await RefreshAsync();
  }
  await Enter(1,"3");await Enter(2,"1");await Enter(3,"=ALLOCATEWEIGHTEDRANGE(10,1,2,A1:A2)");await Enter(4,"'001234");
  AutomationCheck(WorkbookSession.Decode(Sheet.CellAt(AutomationPosition(3,1))!.Value)==7.5m,"Window uses canonical range calculation");
  Sheet.Focus();Sheet.RaiseEvent(new KeyEventArgs{RoutedEvent=KeyDownEvent,Key=Key.End,KeyModifiers=KeyModifiers.Control});
  await AutomationWaitAsync(()=>Sheet.Selected.Row==100&&Sheet.Selected.Column==10&&frameReady);
  AutomationCheck(Sheet.FirstRow+(uint)Sheet.VisibleRows-1<=100,"Window keyboard navigation stays within declared dimensions");
  await SelectAsync(AutomationPosition(1,1));await RefreshAsync();
  var native=Clipboard!;await native.SetTextAsync("window-copy-sentinel");
  await SelectAsync(AutomationPosition(1,1));await ClipboardAsync(false);
  await AutomationExpectClipboardTextAsync(native,"window-copy-sentinel","native Window Copy refusal leaves OS clipboard untouched");
  WindowNameEditor.Text="O'Brien Items";RenameWindowButton.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
  await AutomationWaitAsync(()=>s.Windows.Single().Name=="O'Brien Items"&&!fileAction&&frameReady);
  await SwitchWorksheetAsync(sheet);await Enter(1,"='O''Brien Items'|A3");
  AutomationCheck(WorkbookSession.Decode(Sheet.CellAt(AutomationPosition(1,1))!.Value)==7.5m,"worksheet resolves Window identity");
  string path=Path.Combine(Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetDirectoryName(report)!,"windows-ui-"+Guid.NewGuid()+".pivot");
  AutomationCheck(await SaveDocumentAsync(true,path),"WB15 saves Window directory");
  await s.NewAsync(s.Revision,discard:true);await RefreshAsync();await OpenDocumentAsync(path);
  await AutomationWaitAsync(()=>frameReady&&displayedRevision==s.Revision&&Sheet.CellAt(AutomationPosition(1,1)) is not null);
  AutomationCheck(s.Windows.Single().Id==id&&WorkbookSession.Decode(Sheet.CellAt(AutomationPosition(1,1))!.Value)==7.5m,"WB15 restores Window binding");
  await SwitchWorksheetAsync(id);
  DeleteWindowButton.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
  await AutomationWaitAsync(()=>OwnedWindows.Any(w=>w.Title=="Delete Window"));
  var dialog=OwnedWindows.Single(w=>w.Title=="Delete Window");
  AutomationCheck(dialog.GetVisualDescendants().OfType<TextBlock>().Any(t=>t.Text?.Contains("1 inbound reference(s)")==true),"deletion displays exact inbound count");
  dialog.GetVisualDescendants().OfType<Button>().Single(b=>Equals(b.Content,"Delete")).RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
  await AutomationWaitAsync(()=>s.Windows.Count==0&&!fileAction&&frameReady);
  AutomationCheck(!s.IsWindowScope&&Sheet.CellAt(AutomationPosition(1,1))!.ErrorCode=="ReferenceDeleted","Window deletion faults dependent and restores worksheet view");
  await s.NewAsync(s.Revision,discard:true);await RefreshAsync();Sheet.Focus();
 }
}
