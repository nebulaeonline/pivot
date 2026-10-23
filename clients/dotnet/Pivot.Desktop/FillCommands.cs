using System;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Pivot.Protocol.Application.V1;
namespace Pivot.Desktop;
public partial class MainWindow {
 private bool CanFill => session is not null && session.DocumentId.Length!=0 && frameReady && displayedRevision==session.Revision && draftCell is null && !editing && !fileAction && !shuttingDown && !Sheet.IsSelecting && Sheet.ProjectedSelection is null;
 private void UpdateFillMenu(){
  if(FillDownItem is null)return;
  string reason=draftCell is not null ? "Finish or cancel the cell edit before Fill." : !CanFill ? "Fill needs a ready ordinary worksheet selection; finish pending operations or dragging." : "Fill selected rectangle (up to 256 positions); source edge is copied, not a series.";
  foreach(var item in new[]{FillDownItem,FillUpItem,FillRightItem,FillLeftItem}){item.IsEnabled=CanFill;ToolTip.SetTip(item,reason);}
  ToolTip.SetTip(FillRoot,reason);
 }
 private void FillOpened(object? sender,RoutedEventArgs e)=>UpdateFillMenu();
 private async void FillClick(object? sender,RoutedEventArgs e){if(sender is MenuItem item && Enum.TryParse<Request.Types.FillDirection>(item.Tag?.ToString(),out var direction))await FillAsync(direction);}
 private async Task FillAsync(Request.Types.FillDirection direction){
  if(!CanFill){StatusLabel.Text="Finish or cancel editing, dragging or pending work before Fill on ordinary cells.";return;}
  var s=session!;var first=Sheet.SelectionFirst.Clone();var last=Sheet.SelectionLast.Clone();string expected=displayedRevision;
  editing=true;SetEditing(false);++clipboardRequest;refresh?.Cancel();++displayRequest;
  try{
   await s.FillAsync(expected,first,last,direction);await RefreshAsync();StatusLabel.Text="Fill "+direction+" complete.";
  }catch(Exception error){await RefreshAsync();StatusLabel.Text=error.Message;}
  finally{editing=false;SetEditing(s.DocumentId.Length!=0);clipboardEditor=null;Sheet.Focus();}
 }
}
