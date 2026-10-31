using System;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Layout;
using Avalonia.Threading;
using Pivot.Protocol.Application.V1;
namespace Pivot.Desktop;
public partial class MainWindow {
 private Window? activeVisibleEditDialog;
 private async void ClearVisibleClick(object? sender,RoutedEventArgs args){if(sender is MenuItem item&&Enum.TryParse<Request.Types.ClearMode>(item.Tag?.ToString(),out var mode))await EditVisibleAsync(mode,Request.Types.FillDirection.FillUnspecified);}
 private async void FillVisibleClick(object? sender,RoutedEventArgs args){if(sender is MenuItem item&&Enum.TryParse<Request.Types.FillDirection>(item.Tag?.ToString(),out var direction))await EditVisibleAsync(Request.Types.ClearMode.ClearUnspecified,direction);}
 private async Task EditVisibleAsync(Request.Types.ClearMode mode,Request.Types.FillDirection direction){
  if(!CanFill||session!.FilterStatus is not (0 or 1)){StatusLabel.Text="Visible Clear/Fill needs a qualified ordinary selection with no active edit or pending operation.";return;}
  var s=session;var first=Sheet.SelectionFirst.Clone();var last=Sheet.SelectionLast.Clone();string expected=displayedRevision;
  editing=true;SetEditing(false);++clipboardRequest;refresh?.Cancel();++displayRequest;
  try{
   if(mode!=Request.Types.ClearMode.ClearUnspecified)await s.ClearVisibleAsync(expected,first,last,mode);else await s.FillVisibleAsync(expected,first,last,direction);
   await RefreshAsync();StatusLabel.Text="Visible "+(mode!=Request.Types.ClearMode.ClearUnspecified?"Clear "+mode:"Fill "+direction)+" complete.";
  }catch(Exception error){await RefreshAsync();StatusLabel.Text=error.Message;}
  finally{editing=false;SetEditing(s.DocumentId.Length!=0);clipboardEditor=null;Sheet.Focus();}
 }
 private Task<bool> ConfirmVisibleEditAsync(Request operation,Response preview,CancellationToken cancellation)=>OnUiAsync(async()=>{
  cancellation.ThrowIfCancellationRequested();bool clear=operation.Operation==Request.Types.Operation.ClearVisible;uint n=(uint)preview.VisibleRows.Count,w=preview.VisibleWidth;
  uint limit=clear?4096u:256u;if(n==0||w==0||(ulong)n*w>limit)throw new InvalidOperationException("Invalid visible edit plan.");
  string Col(uint c)=>Pivot.Avalonia.WorksheetView.Address(new Position{Row=1,Column=c}).TrimEnd('1');
  string columns=Col(preview.VisibleFirstColumn)+":"+Col(preview.VisibleFirstColumn+w-1);
  string action=clear?"Clear "+(operation.ClearMode==Request.Types.ClearMode.Formatting?"Formatting (Built-in Defaults)":operation.ClearMode==Request.Types.ClearMode.Both?"Contents and Formatting":"Contents"):"Fill "+operation.FillDirection;
  string sources="none",destinations=VisibleRowRuns(preview.VisibleRows)+", columns "+columns;uint sourceSlots=0,destinationSlots=n*w;
  if(!clear){
   bool vertical=operation.FillDirection is Request.Types.FillDirection.Down or Request.Types.FillDirection.Up;
   sourceSlots=vertical?w:n;destinationSlots=n*w-sourceSlots;
   if(vertical){uint source=operation.FillDirection==Request.Types.FillDirection.Down?preview.VisibleRows[0]:preview.VisibleRows[^1];sources=$"row {source}, columns {columns}";destinations=VisibleRowRuns(preview.VisibleRows.Where(r=>r!=source))+", columns "+columns;}
   else{uint source=operation.FillDirection==Request.Types.FillDirection.Right?preview.VisibleFirstColumn:preview.VisibleFirstColumn+w-1;uint a=operation.FillDirection==Request.Types.FillDirection.Right?source+1:preview.VisibleFirstColumn,b=operation.FillDirection==Request.Types.FillDirection.Right?preview.VisibleFirstColumn+w-1:source-1;sources=$"rows {VisibleRowRuns(preview.VisibleRows)}, column {Col(source)}";destinations=$"rows {VisibleRowRuns(preview.VisibleRows)}, columns "+(destinationSlots==0?"none":Col(a)+":"+Col(b));}
  }
  var panel=new StackPanel{Margin=new global::Avalonia.Thickness(20),Spacing=12};var window=new Window{Title="Review Visible "+(clear?"Clear":"Fill"),Width=680,MaxHeight=650,SizeToContent=SizeToContent.Height,WindowStartupLocation=WindowStartupLocation.CenterOwner};activeVisibleEditDialog=window;
  panel.Children.Add(new TextBlock{Text=$"Worksheet: {session!.Worksheets.Single(s=>s.Id==operation.Worksheet).Name}\n{action}\nSelection: {Pivot.Avalonia.WorksheetView.Address(operation.First)}:{Pivot.Avalonia.WorksheetView.Address(operation.Last)}\nVisible rows: {VisibleRowRuns(preview.VisibleRows)}; columns: {columns}\nSources: {sources} ({sourceSlots} slots)\nDestinations: {destinations} ({destinationSlots} slots)\nAddressed slots including sources: {n*w} / {limit}\nSkipped hidden rows: {preview.VisibleSkippedRows}",TextWrapping=global::Avalonia.Media.TextWrapping.Wrap});
  panel.Children.Add(new TextBlock{Text=destinationSlots==0?"No destinations: this is a reviewed no-op.":clear?(operation.ClearMode==Request.Types.ClearMode.Contents?"Contents become Blank; existing cell identities and all formatting are preserved.":"Every addressed visible cell receives all ten explicit built-in format defaults. Worksheet defaults and hidden cells are unchanged. "+(operation.ClearMode==Request.Types.ClearMode.Both?"Contents also become Blank in the same transaction.":"Contents and formula bindings are preserved.")):"Copy original visible source-edge content and complete styles. Formulas translate by actual worksheet offsets, not visible row ordinals. Hidden reference targets and ordinary Blank anchors keep their meaning.",TextWrapping=global::Avalonia.Media.TextWrapping.Wrap});
  var buttons=new StackPanel{Orientation=Orientation.Horizontal,Spacing=12};var cancel=new Button{Content="Cancel",IsCancel=true};var apply=new Button{Content=destinationSlots==0?"Confirm no-op":"Apply to these cells"};cancel.Click+=(_,_)=>window.Close(false);apply.Click+=(_,_)=>window.Close(true);buttons.Children.Add(cancel);buttons.Children.Add(apply);panel.Children.Add(buttons);window.Content=new ScrollViewer{Content=panel};
  try{var result=window.ShowDialog<bool>(this);using var registration=cancellation.Register(()=>Dispatcher.UIThread.Post(()=>{if(window.IsVisible)window.Close(false);}));return await result;}finally{activeVisibleEditDialog=null;}
 });
}
