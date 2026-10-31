using System;
using System.IO;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
namespace Pivot.Desktop;
public partial class MainWindow {
 private async Task StructureAutomationAsync(string report) {
  var s=session!;await s.NewAsync(s.Revision,discard:true);await RefreshAsync();
  Position P(uint r,uint c=1)=>new(){Row=r,Column=c};
  async Task Enter(uint r,uint c,string text){AutomationCheck(await SelectAsync(P(r,c)),"structure editor selection");BeginCellEdit(true,text);AutomationCheck(await CommitAsync(),"structure editor commit");await RefreshAsync();}
  async Task Command(MenuItem item){StructureOpened(null,new RoutedEventArgs());AutomationCheck(item.IsEnabled,"structural menu enabled");item.RaiseEvent(new RoutedEventArgs(MenuItem.ClickEvent));await AutomationWaitAsync(()=>!editing&&displayedRevision==s.Revision);await RefreshAsync();}
  await Enter(2,1,"7");await Enter(5,2,"=$A$2*2");await Enter(5,3,"=IF(FALSE,A2,1)");await Enter(1,4,"'001234");
  string target=(await s.ReadAsync(P(2),P(2))).Cells[0].CellId;
  await SelectAsync(P(2));await Command(InsertRowsItem);
  AutomationCheck(Sheet.Selected.Row==2&&Sheet.SelectionLast.Column==1024,"inserted row selected with first-row focus");
  AutomationCheck((await s.ReadAsync(P(3),P(3))).Cells[0].CellId==target,"menu insertion preserves cell identity");
  await SelectAsync(P(3));await Command(DeleteRowsItem);
  await SelectAsync(P(5,2));await RefreshAsync();AutomationCheck(Sheet.CellAt(P(5,2))?.ErrorCode=="ReferenceDeleted"&&Sheet.CellAt(P(5,2))?.Formula=="=#REF!*2","deleted reference displayed in formula bar source");
  BeginCellEdit(false);AutomationCheck(CellEditor.Text=="=#REF!*2","fault operand editable");string revision=s.Revision;AutomationCheck(await CommitAsync()&&s.Revision==revision,"unchanged fault edit retains provenance and revision");
  Sheet.Focus();await ClipboardAsync(false);await SelectAsync(P(8,2));Sheet.Focus();await ClipboardAsync(true);await RefreshAsync();AutomationCheck(Sheet.CellAt(P(8,2))?.ErrorCode=="ReferenceDeleted","OS CB5 paste preserves fault");
  await SelectAsync(P(8,2));await SelectAsync(P(9,2),true);FillDownItem.RaiseEvent(new RoutedEventArgs(MenuItem.ClickEvent));await AutomationWaitAsync(()=>!editing&&displayedRevision==s.Revision);await RefreshAsync();AutomationCheck(Sheet.CellAt(P(9,2))?.ErrorCode=="ReferenceDeleted","fault-bearing Fill");
  await SelectAsync(P(5,2));BeginCellEdit(false);CellEditor.Text="=A3*2";AutomationCheck(await CommitAsync(),"repair only broken operand without clearing cell");await Enter(3,1,"9");await RefreshAsync();AutomationCheck(WorkbookSession.Decode(Sheet.CellAt(P(5,2))!.Result.Decimal)==18,"repaired formula calculates normally");AutomationCheck(WorkbookSession.Decode(Sheet.CellAt(P(5,3))!.Result.Decimal)==1,"lazy IF ignores deleted reference");
  await s.FormatAsync(s.Revision,P(65536,20),P(65536,20),new CellStyle(Bold:true),32);await RefreshAsync();await SelectAsync(P(1));
  StructureOpened(null,new RoutedEventArgs());InsertRowsItem.RaiseEvent(new RoutedEventArgs(MenuItem.ClickEvent));await AutomationWaitAsync(()=>!editing&&displayedRevision==s.Revision);AutomationCheck(StatusLabel.Text?.Contains("discarded")==true,"actual presentation clipping notice");
  await SelectAsync(P(6,2));await Command(InsertColumnsItem);AutomationCheck(Sheet.Selected.Row==6&&Sheet.Selected.Column==2&&Sheet.SelectionLast.Row==65536&&Sheet.FirstRow<20,"whole-column band retains focused row and viewport");
  await SelectAsync(P(6,2));await Command(DeleteColumnsItem);await RefreshAsync();AutomationCheck(WorkbookSession.Decode(Sheet.CellAt(P(6,2))!.Result.Decimal)==18,"column insert/delete survives scalar bindings");
  string path=Path.Combine(Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetDirectoryName(report)!,"structure-ui-"+Guid.NewGuid()+".pivot");AutomationCheck(await SaveDocumentAsync(true,path),"structural WB11 save");await s.NewAsync(s.Revision,discard:true);await RefreshAsync();await OpenDocumentAsync(path);
  var fault=(await s.ReadAsync(P(10,2),P(10,2))).Cells[0];AutomationCheck(fault.ErrorCode=="ReferenceDeleted","fault remains after WB11 reopen");AutomationCheck((await s.ReadAsync(P(2,4),P(2,4))).Cells[0].Result.Text=="001234","structural String exactness");
  AutomationImage(Path.ChangeExtension(report,"structure.png"));await s.NewAsync(s.Revision,discard:true);CancelDraft();clipboardEditor=null;await RefreshAsync();Sheet.Focus();
 }
}
