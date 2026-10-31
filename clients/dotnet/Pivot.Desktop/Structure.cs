using System;
using System.Threading.Tasks;
using Avalonia.Interactivity;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
namespace Pivot.Desktop;
public partial class MainWindow {
 private void StructureOpened(object? sender,RoutedEventArgs args) {
  var first=Sheet.SelectionFirst;var last=Sheet.SelectionLast;
  uint rows=last.Row-first.Row+1,columns=last.Column-first.Column+1;
  bool allowed=session is not null&&session.DocumentId.Length!=0&&!session.IsBindingProof&&!session.IsProjectionProof&&!editing&&!fileAction&&!shuttingDown&&!Sheet.IsSelecting;
  InsertRowsItem.Header=$"Insert {rows:N0} row(s) above";DeleteRowsItem.Header=$"Delete {rows:N0} entire row(s)";
  InsertColumnsItem.Header=$"Insert {columns:N0} column(s) left";DeleteColumnsItem.Header=$"Delete {columns:N0} entire column(s)";
  InsertRowsItem.IsEnabled=DeleteRowsItem.IsEnabled=InsertColumnsItem.IsEnabled=DeleteColumnsItem.IsEnabled=allowed;
 }
 private async void InsertRowsClick(object? s,RoutedEventArgs e)=>await StructureAsync(false,false);
 private async void DeleteRowsClick(object? s,RoutedEventArgs e)=>await StructureAsync(false,true);
 private async void InsertColumnsClick(object? s,RoutedEventArgs e)=>await StructureAsync(true,false);
 private async void DeleteColumnsClick(object? s,RoutedEventArgs e)=>await StructureAsync(true,true);
 private async Task StructureAsync(bool columns,bool delete) {
  if(session is null||session.IsBindingProof||session.IsProjectionProof||editing||fileAction||shuttingDown||Sheet.IsSelecting||Sheet.ProjectedSelection is not null||!await CommitAsync())return;
  var s=session;var first=Sheet.SelectionFirst;var last=Sheet.SelectionLast;var focus=Sheet.Selected;
  uint start=columns?first.Column:first.Row,count=columns?last.Column-first.Column+1:last.Row-first.Row+1;
  editing=true;SetEditing(false);++clipboardRequest;refresh?.Cancel();++displayRequest;
  try {
   var outcome=await s.StructureAsync(s.Revision,columns,delete,start,count);
   if(outcome.StructureChanged) {
    var begin=columns?new Position{Row=1,Column=start}:new Position{Row=start,Column=1};
    var end=columns?new Position{Row=WorkbookCapacity.MaximumRows,Column=start+(delete?0:count-1)}:new Position{Row=start+(delete?0:count-1),Column=WorkbookCapacity.MaximumColumns};
    var selected=columns?new Position{Row=focus.Row,Column=start}:new Position{Row=start,Column=focus.Column};
    Sheet.SelectStructuralBand(begin,end,selected);Sheet.Reveal(selected);
   }
   await RefreshAsync();if(FormatPane.IsExpanded)await LoadFormatting();
   StatusLabel.Text=outcome.StructureChanged?$"{(delete?"Deleted":"Inserted")} {count:N0} {(columns?"column(s)":"row(s)")}. {outcome.StructureMoved:N0} cells moved; {outcome.StructureRetired:N0} retired.":"No workbook change.";
   if(outcome.FormattingClipped)StatusLabel.Text+=" Explicit formatting beyond the worksheet edge was discarded.";
  }catch(Exception e){StatusLabel.Text=e.Message;}
  finally{editing=false;SetEditing(s.DocumentId.Length!=0);clipboardEditor=null;Sheet.Focus();}
 }
}
