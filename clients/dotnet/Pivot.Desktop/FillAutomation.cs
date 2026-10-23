using System;
using System.IO;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Input.Platform;
using Avalonia.Interactivity;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
namespace Pivot.Desktop;
public partial class MainWindow {
 private async Task FillAutomationAsync(string report){
  var s=session!;var clipboard=Clipboard??throw new InvalidOperationException("Clipboard unavailable");string? text=await clipboard.TryGetTextAsync();
  var primary=OperatingSystem.IsMacOS()?KeyModifiers.Meta:KeyModifiers.Control;
  foreach(var direction in new[]{Request.Types.FillDirection.Down,Request.Types.FillDirection.Up,Request.Types.FillDirection.Right,Request.Types.FillDirection.Left}){
   await s.NewAsync(s.Revision,discard:true);
   uint row=direction==Request.Types.FillDirection.Up?4u:2u,col=direction==Request.Types.FillDirection.Left?4u:2u;
   await s.ApplyAsync(s.Revision,[new Edit{At=AutomationPosition(row,col),Formula="=SUM($J$1:$J$2)+H"+row},new Edit{At=AutomationPosition(1,10),Number=WorkbookSession.Encode(2)}]);await RefreshAsync();
   await SelectAsync(AutomationPosition(4,4));await SelectAsync(AutomationPosition(2,2),true);Sheet.Focus();
   var active=Sheet.Selected.Clone();uint firstRow=Sheet.FirstRow,firstColumn=Sheet.FirstColumn;UpdateFillMenu();
   var item=direction switch{Request.Types.FillDirection.Down=>FillDownItem,Request.Types.FillDirection.Up=>FillUpItem,Request.Types.FillDirection.Right=>FillRightItem,_=>FillLeftItem};
   AutomationCheck(item.IsEnabled,"Fill menu enabled for ordinary grid");item.RaiseEvent(new RoutedEventArgs(MenuItem.ClickEvent));await AutomationWaitAsync(()=>!editing&&displayedRevision==s.Revision);await RefreshAsync();
   AutomationCheck(Sheet.Selected.Equals(active)&&Sheet.FirstRow==firstRow&&Sheet.FirstColumn==firstColumn&&Sheet.SelectionFirst.Equals(AutomationPosition(2,2))&&Sheet.SelectionLast.Equals(AutomationPosition(4,4)),"Fill preserves reversed selection, active cell and scroll");
   uint targetRow=direction==Request.Types.FillDirection.Up?2u:4u,targetCol=direction==Request.Types.FillDirection.Left?2u:4u;
   // Source edges include blank members: check the lane containing the formula.
   if(direction is Request.Types.FillDirection.Up or Request.Types.FillDirection.Down)targetCol=col;else targetRow=row;
   AutomationCheck(Sheet.CellAt(AutomationPosition(targetRow,targetCol))?.Value is not null,"Menu Fill installs numeric formula");
   string before=s.Revision;bool reverse=direction is Request.Types.FillDirection.Up or Request.Types.FillDirection.Left;
   Sheet.RaiseEvent(new KeyEventArgs{RoutedEvent=InputElement.KeyDownEvent,Key=direction is Request.Types.FillDirection.Up or Request.Types.FillDirection.Down?Key.D:Key.R,KeyModifiers=primary|(reverse?KeyModifiers.Shift:KeyModifiers.None)});
   await AutomationWaitAsync(()=>!editing);AutomationCheck(before==s.Revision&&draftCell is null,"Repeated Fill shortcut is identity-preserving and does not type a letter");shortcutModifiers=KeyModifiers.None;
  }
  await SelectAsync(AutomationPosition(2,2));BeginCellEdit(false);string editorRevision=s.Revision;UpdateFillMenu();AutomationCheck(!FillDownItem.IsEnabled,"Fill disabled during inline edit");
  await FillAsync(Request.Types.FillDirection.Down);AutomationCheck(draftCell is not null&&s.Revision==editorRevision,"Fill does not commit/discard draft");CancelDraft();
  await SelectAsync(AutomationPosition(2,2));await SelectAsync(AutomationPosition(4,4),true);AutomationImage(Path.ChangeExtension(report,"fill.png"));
  AutomationCheck(text==await clipboard.TryGetTextAsync(),"Fill leaves OS clipboard unchanged");
  string root=Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetTempPath(),path=Path.Combine(root,"pivot-fill-ui-"+Guid.NewGuid()+".pivot");
  AutomationCheck(await SaveDocumentAsync(true,path),"Fill saves WB5");await s.NewAsync(s.Revision,discard:true);await s.OpenAsync(s.Revision,path,discard:true);await RefreshAsync();AutomationCheck(!s.IsDirty&&Sheet.CellAt(AutomationPosition(2,2))?.Value is not null,"Fill WB5 reopens");
  await s.NewAsync(s.Revision,discard:true);CancelDraft();shortcutModifiers=KeyModifiers.None;await RefreshAsync();Sheet.Focus();
 }
}
