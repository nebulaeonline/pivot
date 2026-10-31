using System;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Threading;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
namespace Pivot.Desktop;
public partial class MainWindow {
 private async Task VisibleEditAutomationAsync(string report){
  var s=session!;await s.NewAsync(s.Revision,discard:true);CancelDraft();clipboardEditor=null;await RefreshAsync();Sheet.Focus();
  Position P(uint r,uint c=1)=>new(){Row=r,Column=c};
  async Task Ready(){var clock=global::System.Diagnostics.Stopwatch.StartNew();while(true){if(clock.Elapsed.TotalSeconds>10)throw new TimeoutException("Visible edit test needs stable painted view");await AutomationWaitAsync(()=>s.FilterStatus is 0 or 1&&!editing&&frameReady&&displayedRevision==s.Revision);long interaction=interactionVersion;try{await Sheet.PaintCompletion.WaitAsync(TimeSpan.FromSeconds(5));}catch(OperationCanceledException){continue;}await Dispatcher.UIThread.InvokeAsync(()=>{},DispatcherPriority.Background);if(frameReady&&!editing&&displayedRevision==s.Revision&&interaction==interactionVersion)return;}}
  async Task Select(){await SelectAsync(P(2,2));await SelectAsync(P(12,3),true);await Ready();}
  async Task<Window> Open(bool clear,string tag){await Ready();StructureOpened(null,new RoutedEventArgs());var root=clear?ClearVisibleRoot:FillVisibleRoot;AutomationCheck(root.IsEnabled,"visible edit menu enabled");root.Items.OfType<MenuItem>().Single(i=>i.Tag?.ToString()==tag).RaiseEvent(new RoutedEventArgs(MenuItem.ClickEvent));await AutomationWaitAsync(()=>activeVisibleEditDialog?.IsVisible==true||!editing);return activeVisibleEditDialog is {IsVisible:true} window?window:throw new InvalidOperationException("Visible edit did not open review: "+StatusLabel.Text);}
  StackPanel Panel(Window w)=>(StackPanel)((ScrollViewer)w.Content!).Content!;
  async Task Choose(Window w,bool yes){((Button)((StackPanel)Panel(w).Children.Last()).Children[yes?1:0]).RaiseEvent(new RoutedEventArgs(Button.ClickEvent));await AutomationWaitAsync(()=>!editing);await Ready();}
  await s.ApplyAsync(s.Revision,[new Edit{At=P(2),Text="shown"},new Edit{At=P(5),Text="shown"},new Edit{At=P(12),Text="shown"},new Edit{At=P(2,2),Text="001234"},new Edit{At=P(2,3),Formula="=B2"},new Edit{At=P(5,2),Text="middle"},new Edit{At=P(12,2),Text="last"},new Edit{At=P(8,2),Text="hidden sentinel"}]);
  await s.SetFilterAsync(s.Revision,new FilterDefinition{First=P(1),Last=P(20,5),Conditions={new FilterCondition{Column=1,Predicates={new FilterPredicate{Operator=0,Text="shown"}}}}});await RefreshAsync();await Ready();
  var hidden=(await s.ReadAsync(P(8,2),P(8,2))).Cells.Single();await Select();string revision=s.Revision;var window=await Open(false,"Down");string summary=((TextBlock)Panel(window).Children[0]).Text!;AutomationCheck(summary.Contains("2, 5, 12")&&summary.Contains("Sources: row 2")&&summary.Contains("(4 slots)")&&summary.Contains("Skipped hidden rows: 8"),"exact visible Fill source/destination review");await Choose(window,false);AutomationCheck(s.Revision==revision,"declined Fill unchanged");
  window=await Open(false,"Down");await Choose(window,true);var cells=(await s.ReadAsync(P(2,2),P(12,3))).Cells;AutomationCheck(cells.Single(c=>c.At.Row==12&&c.At.Column==2).Text=="001234"&&cells.Single(c=>c.At.Row==12&&c.At.Column==3).Formula=="=B12","routed Down preserves exact Strings and geometric formulas");
  foreach(string direction in new[]{"Up","Right","Left"}){await Select();window=await Open(false,direction);await Choose(window,true);AutomationCheck(!StatusLabel.Text!.Contains("cycle",StringComparison.OrdinalIgnoreCase),"routed "+direction+" succeeds");}
  AutomationCheck((await s.ReadAsync(P(8,2),P(8,2))).Cells.Single().Equals(hidden),"all Fill directions leave hidden cell unchanged");
  // Remove the filter before changing worksheet-wide defaults using its ordinary command.
  await s.SetFilterAsync(s.Revision,null);await s.FormatRegionAsync(s.Revision,P(1),P(1),new CellStyle(Background:0xffff00),1u<<9,defaults:true);
  await s.FormatAsync(s.Revision,P(2,2),P(12,3),new CellStyle(Bold:true,Italic:true),(1u<<5)|(1u<<6));
  await s.SetFilterAsync(s.Revision,new FilterDefinition{First=P(1),Last=P(20,5),Conditions={new FilterCondition{Column=1,Predicates={new FilterPredicate{Operator=0,Text="shown"}}}}});await RefreshAsync();await Ready();await Select();
  window=await Open(true,"Formatting");AutomationCheck(((TextBlock)Panel(window).Children[1]).Text!.Contains("all ten explicit built-in"),"format review distinguishes explicit built-in defaults");await Choose(window,true);
  var styles=await s.ReadStylesAsync(P(2,2),P(12,3));AutomationCheck(CellStyle.Decode(styles.Styles.Single(c=>c.At.Row==5&&c.At.Column==2).Style)==new CellStyle(),"visible exceptional formatting completely reset");AutomationCheck(CellStyle.Decode(styles.Styles.Single(c=>c.At.Row==8&&c.At.Column==2).Style).Bold,"hidden formatting preserved");
  await Select();window=await Open(true,"Contents");await Choose(window,true);AutomationCheck((await s.ReadAsync(P(12,2),P(12,2))).Cells.Single().ContentCase==Cell.ContentOneofCase.Blank,"routed visible Contents clear");
  await Select();window=await Open(true,"Both");await Choose(window,true);AutomationCheck((await s.ReadAsync(P(8,2),P(8,2))).Cells.Single().Equals(hidden),"Clear Both leaves hidden value intact");
  await s.SetFilterAsync(s.Revision,null);await RefreshAsync();await SelectAsync(P(2,2));await SelectAsync(P(2,3),true);await Ready();revision=s.Revision;window=await Open(false,"Down");AutomationCheck(((TextBlock)Panel(window).Children[1]).Text!.Contains("No destinations"),"degenerate Fill still reviewed");await Choose(window,true);AutomationCheck(s.Revision==revision,"reviewed degenerate no-op preserves revision");
  AutomationImage(Path.ChangeExtension(report,"visible-edit.png"));await s.NewAsync(s.Revision,discard:true);CancelDraft();clipboardEditor=null;await RefreshAsync();Sheet.Focus();
 }
}
