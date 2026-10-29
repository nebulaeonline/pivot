using System;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Pivot.Client;
namespace Pivot.Desktop;
public partial class MainWindow {
 private async Task FormattingAutomationAsync(string report) {
  var s=session!;
  async Task Pane(bool expanded) {
   if(FormatPane.IsExpanded!=expanded) {
    var completed=new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
    void Done(object? sender,RoutedEventArgs e)=>completed.TrySetResult();
    if(expanded)FormatPane.Expanded+=Done;else FormatPane.Collapsed+=Done;
    try {FormatPane.IsExpanded=expanded;await completed.Task.WaitAsync(TimeSpan.FromSeconds(10));}
    finally {if(expanded)FormatPane.Expanded-=Done;else FormatPane.Collapsed-=Done;}
   }
   UpdateLayout();await RefreshAsync();
   await AutomationWaitAsync(()=>frameReady&&!editing&&displayedRevision==s.Revision);
   if(expanded)await LoadFormatting();
  }
  void Trace(string phase)=>File.AppendAllText(Path.ChangeExtension(report,"formatting-clipboard.log"),
      $"{phase}; frame={frameReady}; editing={editing}; draft={draftCell is not null}; editor={clipboardEditor is not null}; pane={FormatPane.IsExpanded}; firstRow={Sheet.FirstRow}; rows={Sheet.VisibleRows}\n");
  try {
  await s.NewAsync(s.Revision,discard:true);await RefreshAsync();
  await SelectAsync(AutomationPosition(1,1));BeginCellEdit(true,"1234.5678");AutomationCheck(await CommitAsync(),"format input committed");await RefreshAsync();
  await Pane(true);
  formatCombos[0].SelectedIndex=3;formatCombos[4].SelectedIndex=2;formatCombos[8].SelectedIndex=7;
  await ApplyFormatting(false);await RefreshAsync();
  AutomationCheck(Sheet.CellAt(AutomationPosition(1,1))!.FormattedNumber=="$1,234.57","Rust formatted currency reaches grid");
  AutomationCheck(Sheet.StyleAt(AutomationPosition(1,1)).Bold&&Sheet.StyleAt(AutomationPosition(1,1)).Background==0xffeb9c,"grid visual metadata applied");
  await Pane(false);
  BeginCellEdit(false);AutomationCheck(CellEditor.Text=="1234.5678","editor retains exact decimal input");CancelDraft();
  await SelectAsync(AutomationPosition(1,2));await Pane(true);formatCombos[5].SelectedIndex=2;await ApplyFormatting(false);
  AutomationCheck(Sheet.CellAt(AutomationPosition(1,2)) is null&&Sheet.StyleAt(AutomationPosition(1,2)).Italic,"style-only blank has no cell");
  await SelectAsync(AutomationPosition(1,1));await SelectAsync(AutomationPosition(1,2),true);await LoadFormatting();
  AutomationCheck(formatCombos[4].SelectedIndex==0&&formatCombos[5].SelectedIndex==0,"mixed emphasis shown");
  formatCombos[6].SelectedIndex=3;await ApplyFormatting(false);
  AutomationCheck(Sheet.StyleAt(AutomationPosition(1,1)).Bold&&!Sheet.StyleAt(AutomationPosition(1,1)).Italic&&Sheet.StyleAt(AutomationPosition(1,2)).Italic,"mixed patch retains untouched fields");
  await Pane(false);
  AutomationCheck(await SelectAsync(AutomationPosition(1,1)),"format copy anchor selected");
  AutomationCheck(await SelectAsync(AutomationPosition(1,2),true),"format copy rectangle selected");
  await RefreshAsync();await AutomationWaitAsync(()=>frameReady&&displayedRevision==s.Revision);
  bool published=false;
  await ClipboardAsync(false,diagnose:phase=>{Trace("copy: "+phase);if(phase=="native copy published")published=true;});
  AutomationCheck(published,"format native clipboard copy completed; see formatting-clipboard.log");
  AutomationCheck(await SelectAsync(AutomationPosition(3,1)),"format paste target selected");
  await RefreshAsync();await AutomationWaitAsync(()=>frameReady&&displayedRevision==s.Revision);
  bool pasted=false;
  await ClipboardAsync(true,diagnose:phase=>{Trace("paste: "+phase);if(phase=="engine paste completed")pasted=true;});
  AutomationCheck(pasted,"format native clipboard paste completed; see formatting-clipboard.log");
  await RefreshAsync();
  await AutomationWaitAsync(()=>frameReady&&displayedRevision==s.Revision&&Sheet.CellAt(AutomationPosition(3,1)) is not null);
  AutomationCheck(Sheet.CellAt(AutomationPosition(3,1))?.FormattedNumber=="$1,234.57"&&Sheet.CellAt(AutomationPosition(3,2)) is null&&Sheet.StyleAt(AutomationPosition(3,2)).Italic,"native OS paste includes styled blanks");
  string path=Path.Combine(Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetDirectoryName(report)!,"format-ui-"+Guid.NewGuid()+".pivot");
  AutomationCheck(await SaveDocumentAsync(true,path),"formatted workbook saves");await s.NewAsync(s.Revision,discard:true);await RefreshAsync();await OpenDocumentAsync(path);
  // Reopen resets the viewport; expander layout can also replace a pending frame.
  // Verify persistence independently, then reveal the row and await its current frame.
  var reopened=await s.ReadAsync(AutomationPosition(3,1),AutomationPosition(3,2));
  AutomationCheck(reopened.Cells.SingleOrDefault(c=>c.At.Row==3&&c.At.Column==1)?.FormattedNumber=="$1,234.57"
      &&CellStyle.Decode(reopened.Styles.Single(p=>p.At.Column==2).Style).Italic,"WB9 backend restores styles and exact values");
  AutomationCheck(await SelectAsync(AutomationPosition(3,1)),"reopened formatting row selected");
  await RefreshAsync();
  await AutomationWaitAsync(()=>frameReady&&!editing&&displayedRevision==s.Revision&&Sheet.IsVisibleCell(AutomationPosition(3,1))&&Sheet.CellAt(AutomationPosition(3,1)) is not null);
  AutomationCheck(Sheet.StyleAt(AutomationPosition(3,2)).Italic&&Sheet.CellAt(AutomationPosition(3,1))?.FormattedNumber=="$1,234.57","WB9 styles and exact values reach reopened grid");
  AutomationImage(Path.ChangeExtension(report,"formatting.png"));
  await s.NewAsync(s.Revision,discard:true);CancelDraft();clipboardEditor=null;shortcutModifiers=KeyModifiers.None;await RefreshAsync();Sheet.Focus();
  } catch {
   Trace("formatting automation failed");
   AutomationImage(Path.ChangeExtension(report,"formatting-failed.png"));
   throw;
  }
 }
}
