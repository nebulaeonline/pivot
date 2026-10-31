using System;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Input.Platform;
using Avalonia.Interactivity;
using Avalonia.Threading;
using Pivot.Protocol.Application.V1;
namespace Pivot.Desktop;
public partial class MainWindow {
 private async Task VisibleClipboardAutomationAsync(string report){
  var s=session!;await s.NewAsync(s.Revision,discard:true);CancelDraft();clipboardEditor=null;await RefreshAsync();Sheet.Focus();
  Position P(uint r,uint c=1)=>new(){Row=r,Column=c};
  async Task Ready(){var clock=global::System.Diagnostics.Stopwatch.StartNew();while(true){if(clock.Elapsed.TotalSeconds>10)throw new TimeoutException("Visible test did not obtain a painted, stable worksheet frame.");await AutomationWaitAsync(()=>s.FilterStatus is 0 or 1&&!editing&&frameReady&&displayedRevision==s.Revision);long interaction=interactionVersion;try{await Sheet.PaintCompletion.WaitAsync(TimeSpan.FromSeconds(5));}catch(OperationCanceledException){continue;}await Dispatcher.UIThread.InvokeAsync(()=>{},DispatcherPriority.Background);if(frameReady&&!editing&&displayedRevision==s.Revision&&interaction==interactionVersion)return;}}
  async Task<Window> Review(Task command){await AutomationWaitAsync(()=>activeVisiblePasteDialog?.IsVisible==true||command.IsCompleted);if(activeVisiblePasteDialog?.IsVisible!=true){await command;throw new InvalidOperationException($"Visible paste completed before review; frame={frameReady}; editing={editing}; draft={draftCell is not null}; qualified={displayedRevision==s.Revision}; filter={s.FilterStatus}; statusLength={StatusLabel.Text?.Length}");}return activeVisiblePasteDialog;}
  StackPanel Panel(Window window)=>(StackPanel)((ScrollViewer)window.Content!).Content!;
  Button Choice(Window window,bool accept)=>(Button)((StackPanel)Panel(window).Children.Last()).Children[accept?1:0];
  await s.ApplyAsync(s.Revision,[new Edit{At=P(2),Text="001234"},new Edit{At=P(5),Text="001234"},new Edit{At=P(10),Text="001234"},new Edit{At=P(12),Text="001234"},new Edit{At=P(2,2),Formula="=A2"},new Edit{At=P(5,2),Formula="=A2"},new Edit{At=P(11,4),Text="hidden sentinel"}]);
  await s.SetFilterAsync(s.Revision,new FilterDefinition{First=P(1),Last=P(30,5),Conditions={new FilterCondition{Column=1,Predicates={new FilterPredicate{Operator=0,Text="001234"}}}}});await RefreshAsync();await Ready();
  var hidden=(await s.ReadAsync(P(11,4),P(11,4))).Cells.Single();await SelectAsync(P(2));await SelectAsync(P(5,2),true);StructureOpened(null,new RoutedEventArgs());AutomationCheck(CopyVisibleItem.IsEnabled,"visible source command enabled");CopyVisibleItem.RaiseEvent(new RoutedEventArgs(MenuItem.ClickEvent));
  await AutomationExpectClipboardTextAsync(Clipboard!,"'001234\t=A2\n'001234\t=A2","visible source copy publishes exact typed text");
  using(var data=await Clipboard!.TryGetDataAsync())AutomationCheck(data?.Contains(VisibleBlockFormat)==true&&data.Contains(CellBlockFormat)==false,"Copy Visible publishes CB6 without a fabricated CB5 fallback");
  await SelectAsync(P(10,4));await Ready();string revision=s.Revision;var paste=ClipboardAsync(true,visible:true);var window=await Review(paste);string summary=((TextBlock)Panel(window).Children[0]).Text!;AutomationCheck(summary.Contains("10, 12")&&summary.Contains("D:E")&&summary.Contains("4 addressed slots")&&summary.Contains("1 rows skipped"),"review lists actual row runs, columns, addressed slots and skipped rows");Choice(window,false).RaiseEvent(new RoutedEventArgs(Button.ClickEvent));await paste;AutomationCheck(s.Revision==revision,"declined visible paste does not edit");
  await Ready();paste=ClipboardAsync(true,visible:true);window=await Review(paste);Choice(window,true).RaiseEvent(new RoutedEventArgs(Button.ClickEvent));await paste;await Ready();
  var frame=await s.ReadAsync(P(9,4),P(12,5));AutomationCheck(frame.Cells.Single(c=>c.At.Row==12&&c.At.Column==5).Formula=="=D9","actual source-row formula origin survives gaps");AutomationCheck(frame.Cells.Single(c=>c.At.Row==11&&c.At.Column==4).Equals(hidden),"hidden destination unchanged");AutomationCheck(frame.Cells.Single(c=>c.At.Row==10&&c.At.Column==4).Text=="001234"&&Sheet.Selected.Row==10&&Sheet.Selected.Column==4&&!Sheet.HasRegion,"exact String and original single anchor retained");
  revision=s.Revision;paste=ClipboardAsync(true,visible:true);window=await Review(paste);await Clipboard!.SetTextAsync("replacement during review");Choice(window,true).RaiseEvent(new RoutedEventArgs(Button.ClickEvent));await paste;AutomationCheck(s.Revision==revision&&draftCell is null,"clipboard replacement cancels visible paste without draft fallback");
  var valid=await s.CopyBlockAsync(s.Revision,P(10,4),P(10,5));var forged=new DataTransferItem();forged.Set(VisibleBlockFormat,new byte[]{1});forged.Set(CellBlockFormat,valid.Clipboard.ToByteArray());forged.SetText("99");var transfer=new DataTransfer();transfer.Add(forged);await Clipboard.SetDataAsync(transfer);await ClipboardAsync(true);AutomationCheck(s.Revision==revision&&draftCell is null,"malformed preferred CB6 cannot fall through to CB5 or TSV");
  await s.SetFilterAsync(s.Revision,null);await s.ApplyAsync(s.Revision,[new Edit{At=P(70),Text="001234\tline\nnext"}]);await RefreshAsync();await SelectAsync(P(70));await ClipboardAsync(false,visible:true);
  using(var data=await Clipboard.TryGetDataAsync())AutomationCheck(data?.Contains(VisibleBlockFormat)==true&&!data.Contains(DataFormat.Text),"separator-containing String publishes native-only CB6");
  await SelectAsync(P(71));await Ready();paste=ClipboardAsync(true,visible:true);window=await Review(paste);Choice(window,true).RaiseEvent(new RoutedEventArgs(Button.ClickEvent));await paste;AutomationCheck((await s.ReadAsync(P(71),P(71))).Cells.Single().Text=="001234\tline\nnext","native-only exact String roundtrip");
  AutomationImage(Path.ChangeExtension(report,"visible-clipboard.png"));await s.NewAsync(s.Revision,discard:true);CancelDraft();clipboardEditor=null;await RefreshAsync();Sheet.Focus();
 }
}
