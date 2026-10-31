using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Input.Platform;
using Avalonia.Interactivity;
using Avalonia.Layout;
using Avalonia.Threading;
using Pivot.Protocol.Application.V1;
namespace Pivot.Desktop;
public partial class MainWindow {
 private static readonly DataFormat<byte[]> VisibleBlockFormat=DataFormat.CreateBytesPlatformFormat(OperatingSystem.IsMacOS()?"org.pivot.cell-block-v6":"application/x-pivot-cell-block-v6");
 private async void CopyVisibleClick(object? sender,RoutedEventArgs args)=>await ClipboardAsync(false,visible:true);
 private async void PasteVisibleClick(object? sender,RoutedEventArgs args)=>await ClipboardAsync(true,visible:true);
 private static async Task<ClipboardRead.Snapshot> ReadWorksheetClipboardAsync(IAsyncDataTransfer? data,bool worksheet){
  bool cb6=worksheet&&data?.Contains(VisibleBlockFormat)==true;
  bool native=worksheet&&(cb6||data?.Contains(CellBlockFormat)==true);
  bool obsolete=worksheet&&data is not null&&(data.Contains(OldCellBlockFormat)||data.Contains(OldCellBlock2Format)||data.Contains(OldCellBlock3Format)||data.Contains(OldCellBlock4Format));
  byte[]? bytes=native?await data!.TryGetValueAsync(cb6?VisibleBlockFormat:CellBlockFormat):null;
  if(bytes is {Length:>0}&&!bytes.AsSpan().StartsWith(cb6?"PIVOTCB6"u8:"PIVOTCB5"u8))throw new InvalidDataException("Preferred native clipboard representation is malformed; no fallback.");
  return new(data is not null,native,obsolete,data?.Contains(DataFormat.Text)==true,bytes,native||obsolete||data is null?null:await data.TryGetTextAsync(),native?(cb6?6u:5u):0u);
 }
 private static string VisibleRowRuns(IEnumerable<uint> source){
  var rows=source.ToArray();if(rows.Length==0)return "none";var runs=new List<string>();uint first=rows[0],last=first;
  foreach(uint row in rows.Skip(1)){if(row==last+1){last=row;continue;}runs.Add(first==last?$"{first}":$"{first}–{last}");first=last=row;}
  runs.Add(first==last?$"{first}":$"{first}–{last}");return string.Join(", ",runs);
 }
 private Window? activeVisiblePasteDialog;
 private Task<bool> ConfirmVisiblePasteAsync(Request operation,Response preview,CancellationToken cancellation)=>OnUiAsync(async()=>{
  cancellation.ThrowIfCancellationRequested();if(preview.VisibleRows.Count==0||preview.VisibleRows.Count*preview.VisibleWidth>256)throw new InvalidDataException("Invalid visible destination plan.");
  string first=Pivot.Avalonia.WorksheetView.Address(new Position{Row=1,Column=preview.VisibleFirstColumn}).TrimEnd('1');
  string last=Pivot.Avalonia.WorksheetView.Address(new Position{Row=1,Column=preview.VisibleFirstColumn+preview.VisibleWidth-1}).TrimEnd('1');
  var panel=new StackPanel{Margin=new global::Avalonia.Thickness(20),Spacing=12};
  var window=new Window{Title="Review Paste into Visible Rows",Width=660,SizeToContent=SizeToContent.Height,MaxHeight=650,WindowStartupLocation=WindowStartupLocation.CenterOwner};activeVisiblePasteDialog=window;
  panel.Children.Add(new TextBlock{Text=$"Worksheet: {session!.Worksheets.Single(sheet=>sheet.Id==operation.Worksheet).Name}\nDestination worksheet rows: {VisibleRowRuns(preview.VisibleRows)}\nColumns: {first}:{last}\n{preview.VisibleRows.Count} rows × {preview.VisibleWidth} columns = {preview.VisibleRows.Count*preview.VisibleWidth} addressed slots\n{preview.VisibleSkippedRows} rows skipped between destinations.",TextWrapping=global::Avalonia.Media.TextWrapping.Wrap});
  panel.Children.Add(new TextBlock{Text=operation.NativeClipboard?"Formulas move by each cell's actual source-to-destination offset. References between copied records are not rewired. Hidden reference targets and Blank reference anchors retain ordinary formula semantics.":"External TSV formulas use their literal addresses. No native copy origin is inferred.",TextWrapping=global::Avalonia.Media.TextWrapping.Wrap});
  var buttons=new StackPanel{Orientation=Orientation.Horizontal,Spacing=12};var cancel=new Button{Content="Cancel",IsCancel=true};var paste=new Button{Content="Paste into these rows"};cancel.Click+=(_,_)=>window.Close(false);paste.Click+=(_,_)=>window.Close(true);buttons.Children.Add(cancel);buttons.Children.Add(paste);panel.Children.Add(buttons);window.Content=new ScrollViewer{Content=panel};
  try{var result=window.ShowDialog<bool>(this);using var registration=cancellation.Register(()=>Dispatcher.UIThread.Post(()=>{if(window.IsVisible)window.Close(false);}));return await result;}finally{activeVisiblePasteDialog=null;}
 });
}
