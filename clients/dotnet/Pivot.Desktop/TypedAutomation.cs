using System;
using System.IO;
using System.Threading.Tasks;
using Avalonia.Input;
using Pivot.Client;
namespace Pivot.Desktop;
public partial class MainWindow {
 private async Task TypedAutomationAsync(string report){
  var s=session!;await s.NewAsync(s.Revision,discard:true);await RefreshAsync();
  async Task Enter(uint row,string text){
   await AutomationWaitAsync(()=>frameReady&&!editing&&!fileAction&&displayedRevision==s.Revision);
   AutomationCheck(await SelectAsync(AutomationPosition(row,1)),"typed cell selected");BeginCellEdit(true,text);
   AutomationCheck(draftCell is not null,"typed editor starts");CellEditor.RaiseEvent(new KeyEventArgs{RoutedEvent=KeyDownEvent,Key=Key.Enter});
   await AutomationWaitAsync(()=>!editing&&draftCell is null);await RefreshAsync();
  }
  await Enter(1,"TRUE");await Enter(2,"hello");await Enter(3,"=IF(A1,A2&\"!\",1/0)");await Enter(4,"=LEN(\"😀a\")");
  AutomationCheck(WorkbookSession.DisplayValue(Sheet.CellAt(AutomationPosition(1,1))!)=="TRUE"&&WorkbookSession.DisplayValue(Sheet.CellAt(AutomationPosition(3,1))!)=="hello!","typed results in grid");
  await SelectAsync(AutomationPosition(3,1));BeginCellEdit(false);AutomationCheck(CellEditor.Text=="=IF(A1,A2&\"!\",1/0)","F2 edits expression, not result");CancelDraft();
  await SelectAsync(AutomationPosition(1,1));await SelectAsync(AutomationPosition(4,1),true);await ClipboardAsync(false);
  await SelectAsync(AutomationPosition(1,3));await ClipboardAsync(true);await RefreshAsync();
  AutomationCheck(WorkbookSession.DisplayValue(Sheet.CellAt(AutomationPosition(3,3))!)=="hello!","OS native clipboard translates typed formula references");
  string path=Path.Combine(Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetDirectoryName(report)!,"typed-ui-"+Guid.NewGuid()+".pivot");
  AutomationCheck(await SaveDocumentAsync(true,path),"typed workbook saves");await s.NewAsync(s.Revision,discard:true);await RefreshAsync();await OpenDocumentAsync(path);
  AutomationCheck(WorkbookSession.DisplayValue(Sheet.CellAt(AutomationPosition(3,3))!)=="hello!","typed workbook reopens");AutomationImage(Path.ChangeExtension(report,"typed.png"));
  await s.NewAsync(s.Revision,discard:true);await RefreshAsync();
  await Enter(1,"3");await Enter(2,"1");await Enter(3,"=ALLOCATEWEIGHTEDRANGE(10,1,2,$A$1:$A$2)");
  AutomationCheck(WorkbookSession.Decode(Sheet.CellAt(AutomationPosition(3,1))!.Result.Decimal)==7.5m,"range allocation exact grid result");
  await SelectAsync(AutomationPosition(3,1));await ClipboardAsync(false);await SelectAsync(AutomationPosition(5,1));await ClipboardAsync(true);await RefreshAsync();
  AutomationCheck(WorkbookSession.Decode(Sheet.CellAt(AutomationPosition(5,1))!.Result.Decimal)==7.5m,"range allocation native clipboard");
  await Enter(4,"=ALLOCATEWEIGHTEDRANGE(10,1,2,B1:B1)");
  AutomationCheck(Sheet.CellAt(AutomationPosition(4,1))!.ErrorCode=="TypeMismatch","range allocation missing weight stays strict");
  AutomationCheck(await SaveDocumentAsync(true,path),"range workbook saves");await s.NewAsync(s.Revision,discard:true);await RefreshAsync();await OpenDocumentAsync(path);
  AutomationCheck(WorkbookSession.Decode(Sheet.CellAt(AutomationPosition(5,1))!.Result.Decimal)==7.5m,"WB15 range workbook reopens");
  await s.NewAsync(s.Revision,discard:true);CancelDraft();clipboardEditor=null;shortcutModifiers=KeyModifiers.None;await RefreshAsync();Sheet.Focus();
 }
}
