using System;
using System.IO;
using System.Text;
using System.Threading.Tasks;
using Avalonia.Input;
using Avalonia.Input.Platform;
using Pivot.Client;
namespace Pivot.Desktop;
public partial class MainWindow {
 private async Task AggregateAutomationAsync(string report){
  var s=session!;await s.NewAsync(s.Revision,discard:true);await RefreshAsync();
  async Task Enter(uint row,uint col,string text){
   await AutomationWaitAsync(()=>frameReady&&displayedRevision==s.Revision&&!editing&&!fileAction);
   AutomationCheck(await SelectAsync(AutomationPosition(row,col)),"Aggregate editor selects target");
   Sheet.RaiseEvent(new TextInputEventArgs{RoutedEvent=InputElement.TextInputEvent,Text=text});
   AutomationCheck(draftCell is not null&&inlineEditing,"Aggregate inline draft begins");
   CellEditor.RaiseEvent(new KeyEventArgs{RoutedEvent=InputElement.KeyDownEvent,Key=Key.Enter});
   await AutomationWaitAsync(()=>!editing&&draftCell is null&&Sheet.Selected.Equals(AutomationPosition(row+1,col)));
  }
  await Enter(1,1,"1");await Enter(2,1,"2");await Enter(3,1,"2");await Enter(4,1,"Label");
  await Enter(1,2,"=MIN(A1:A5)");await Enter(2,2,"=MAX(A1:A5)");await Enter(3,2,"=AVERAGE(A1:A5)");await Enter(4,2,"=COUNT(A1:A5)");await Enter(5,2,"=SUM(A1:A5)");
  AutomationCheck(WorkbookSession.Decode(Sheet.CellAt(AutomationPosition(1,2))!.Value)==1&&WorkbookSession.Decode(Sheet.CellAt(AutomationPosition(2,2))!.Value)==2&&WorkbookSession.Decode(Sheet.CellAt(AutomationPosition(3,2))!.Value)==1.6666666666666666666666666667m&&WorkbookSession.Decode(Sheet.CellAt(AutomationPosition(4,2))!.Value)==3,"Actual worksheet MIN/MAX/AVERAGE/COUNT");
  AutomationCheck(Sheet.CellAt(AutomationPosition(5,2))!.ErrorCode=="TypeMismatch","SUM unchanged strict text policy");
  await Enter(5,1,"0");AutomationCheck(WorkbookSession.Decode(Sheet.CellAt(AutomationPosition(4,2))!.Value)==4,"Explicit zero participates");
  await SelectAsync(AutomationPosition(5,1));string before=s.Revision;ClearClick(null,new global::Avalonia.Interactivity.RoutedEventArgs());await AutomationWaitAsync(()=>!editing&&s.Revision!=before);await RefreshAsync();
  AutomationCheck(WorkbookSession.Decode(Sheet.CellAt(AutomationPosition(4,2))!.Value)==3,"Clear restores blank classification");
  await Enter(1,5,"=MIN(F1:F5)");AutomationCheck(Sheet.CellAt(AutomationPosition(1,5))!.Error.Contains("No numeric values"),"No numeric values visible");
  await Enter(1,3,"10");await Enter(2,3,"20");
  var clipboard=Clipboard??throw new InvalidOperationException("Clipboard unavailable");string? previous=await clipboard.TryGetTextAsync();
  try{await SelectAsync(AutomationPosition(1,2));await SelectAsync(AutomationPosition(4,2),true);await ClipboardAsync(false);await SelectAsync(AutomationPosition(1,4));await ClipboardAsync(true);
   AutomationCheck(Sheet.CellAt(AutomationPosition(3,4))?.Formula=="=AVERAGE(C1:C5)"&&WorkbookSession.Decode(Sheet.CellAt(AutomationPosition(3,4))!.Value)==15,"Native range-function copy translates references");
  }finally{if(previous is null)await clipboard.ClearAsync();else await clipboard.SetTextAsync(previous);}
  await SelectAsync(AutomationPosition(3,2));AutomationImage(Path.ChangeExtension(report,"aggregates.png"));
  string root=Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetTempPath();string path=Path.Combine(root,"pivot-aggregate-ui-"+Guid.NewGuid()+".pivot");
  AutomationCheck(await SaveDocumentAsync(true,path),"Aggregate document saves");AutomationCheck(Encoding.ASCII.GetString((await File.ReadAllBytesAsync(path))[..9])=="PIVOTWB12","UI writes WB11");
  await s.NewAsync(s.Revision,discard:true);await s.OpenAsync(s.Revision,path,discard:true);await RefreshAsync();AutomationCheck(!s.IsDirty&&WorkbookSession.Decode(Sheet.CellAt(AutomationPosition(4,2))!.Value)==3,"WB5 UI reopen preserves blank semantics");
  await s.NewAsync(s.Revision,discard:true);CancelDraft();clipboardEditor=null;shortcutModifiers=KeyModifiers.None;Sheet.Scroll(1-(int)Sheet.FirstRow,1-(int)Sheet.FirstColumn);await RefreshAsync();Sheet.Focus();
 }
}
