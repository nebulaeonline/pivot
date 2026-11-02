using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls.Primitives;
using Avalonia.Controls;
using Avalonia.Input.Platform;
using Avalonia;
using Avalonia.Interactivity;
using Pivot.Protocol.Application.V1;

namespace Pivot.Desktop;

public partial class MainWindow
{
    private sealed record WorksheetViewState(uint Row,uint Column,Position Anchor,Position Active,Position? Focus=null);
    private readonly Dictionary<string,WorksheetViewState> worksheetViews=[];
    private string worksheetDocument="",worksheetTabSignature="";
    private long worksheetViewEpoch;
    private void EnsureWorksheetViews() {
        if(session is null || worksheetDocument==session.DocumentId+":"+session.DocumentGeneration)return;
        worksheetDocument=session.DocumentId+":"+session.DocumentGeneration;worksheetViews.Clear();worksheetTabSignature="";
        ++worksheetViewEpoch;++interactionVersion;++clipboardRequest;frameReady=false;
        Sheet.RestoreView(1,1,new(){Row=1,Column=1},new(){Row=1,Column=1});
    }
    private void RememberWorksheetView() {
        if(session is not null && session.WorksheetId.Length!=0)worksheetViews[session.WorksheetId]=new(Sheet.FirstRow,Sheet.FirstColumn,Sheet.Anchor,Sheet.SelectionActive,Sheet.SelectionFocus);
    }
    private async Task ChangeWorksheetAsync(string id) {
        if(session is null)return;
        RememberWorksheetView();if(session.Windows.Any(w=>w.Id==id))session.SelectWindow(id);else session.SelectWorksheet(id);
        var dimensions=session.GridScopes.Single(s=>s.Id==id);Sheet.SetGridDimensions(dimensions.Rows,dimensions.Columns);
        ++worksheetViewEpoch;++interactionVersion;++clipboardRequest;++displayRequest;refresh?.Cancel();frameReady=false;
        var state=worksheetViews.GetValueOrDefault(id)??new(1,1,new(){Row=1,Column=1},new(){Row=1,Column=1});
        Sheet.RestoreView(state.Row,state.Column,state.Anchor,state.Active,state.Focus);
        worksheetTabSignature="";UpdateWorksheetTabs();await RefreshAsync();Sheet.Focus();
    }
    private async Task SwitchWorksheetAsync(string id) {
        if(session is null)return;
        if(id==session.WorksheetId){worksheetTabSignature="";UpdateWorksheetTabs();return;}
        await FileActionAsync(()=>ChangeWorksheetAsync(id));
        worksheetTabSignature="";UpdateWorksheetTabs();Sheet.Focus();
    }
    private async Task AddWorksheetAsync() {
        await FileActionAsync(async()=>{
            var added=await session!.AddSheetAsync(session.Revision);
            await ChangeWorksheetAsync(added.Worksheet);
        });
        Sheet.Focus();
    }
    private async Task RenameWorksheetAsync(string name) {
        await FileActionAsync(async()=>{
            await session!.RenameSheetAsync(session.Revision,session.WorksheetId,name);
            worksheetTabSignature="";await RefreshAsync();
        });
    }
    private async void AddWorksheetClick(object? sender,RoutedEventArgs args)=>await AddWorksheetAsync();
    private async void RenameWorksheetClick(object? sender,RoutedEventArgs args)=>await RenameWorksheetAsync(WorksheetNameEditor.Text??"");
    private void UpdateWorksheetTabs() {
        if(session is null)return;
        WorksheetBar.IsVisible=!session.IsProjectionProof&&!session.IsBindingProof;
        WindowBar.IsVisible=WorksheetBar.IsVisible;
        string signature=session.DocumentId+"|"+session.WorksheetId+"|"+string.Join("|",session.GridScopes.Select(s=>s.Id+":"+s.Name));
        if(signature==worksheetTabSignature)return;
        worksheetTabSignature=signature;WorksheetTabs.Children.Clear();WindowTabs.Children.Clear();
        foreach(var sheet in session.Worksheets) {
            var tab=new ToggleButton{Content=sheet.Name,Tag=sheet.Id,IsChecked=sheet.Id==session.WorksheetId};
            tab.Click+=async(_,_)=>await SwitchWorksheetAsync(sheet.Id);WorksheetTabs.Children.Add(tab);
        }
        foreach(var window in session.Windows) {
            var tab=new ToggleButton{Content=window.Name,Tag=window.Id,IsChecked=session.IsWindowScope&&window.Id==session.WorksheetId};
            tab.Click+=async(_,_)=>await SwitchWorksheetAsync(window.Id);WindowTabs.Children.Add(tab);
        }
        var selectedWindow=session.Windows.FirstOrDefault(w=>w.Id==session.WorksheetId);
        WindowNameEditor.Text=selectedWindow?.Name??WindowNameEditor.Text;
        RenameWindowButton.IsEnabled=DeleteWindowButton.IsEnabled=session.IsWindowScope;
        RenameWorksheetButton.IsEnabled=!session.IsWindowScope;
        CreateWindowButton.IsEnabled=session.Windows.Count<64;
        WorksheetNameEditor.Text=session.Worksheets.FirstOrDefault(s=>s.Id==session.WorksheetId)?.Name??"";
        AddWorksheetButton.IsEnabled=session.Worksheets.Count<16;
    }
    private async void CreateWindowClick(object? sender,RoutedEventArgs args) {
        await FileActionAsync(async()=>{
            decimal rows=WindowRows.Value??65536,columns=WindowColumns.Value??1024;
            if(decimal.Truncate(rows)!=rows||decimal.Truncate(columns)!=columns)throw new InvalidOperationException("Rows and columns must be whole numbers.");
            var created=await session!.CreateWindowAsync(session.Revision,WindowNameEditor.Text??"",(uint)rows,(uint)columns);
            await ChangeWorksheetAsync(created.Worksheet);
        });
    }
    private async void RenameWindowClick(object? sender,RoutedEventArgs args) {
        if(session?.IsWindowScope!=true)return;
        await FileActionAsync(async()=>{await session.RenameWindowAsync(session.Revision,session.WorksheetId,WindowNameEditor.Text??"");worksheetTabSignature="";await RefreshAsync();});
    }
    private async void DeleteWindowClick(object? sender,RoutedEventArgs args) {
        if(session?.IsWindowScope!=true)return;
        await FileActionAsync(async()=>{
            string id=session.WorksheetId,revision=session.Revision;
            var preview=await session.PrepareWindowDeleteAsync(revision,id);
            var dialog=new Window{Title="Delete Window",Width=440,SizeToContent=SizeToContent.Height,WindowStartupLocation=WindowStartupLocation.CenterOwner};
            var panel=new StackPanel{Margin=new Thickness(16),Spacing=12};
            panel.Children.Add(new TextBlock{Text=$"Delete this Window and its contents? {preview.WindowInboundReferences:N0} inbound reference(s) will become errors. This cannot be undone.",TextWrapping=global::Avalonia.Media.TextWrapping.Wrap});
            var buttons=new StackPanel{Orientation=global::Avalonia.Layout.Orientation.Horizontal,Spacing=8};
            var remove=new Button{Content="Delete"};remove.Click+=(_,_)=>dialog.Close(true);
            var cancel=new Button{Content="Cancel"};cancel.Click+=(_,_)=>dialog.Close(false);
            buttons.Children.Add(remove);buttons.Children.Add(cancel);panel.Children.Add(buttons);dialog.Content=panel;
            if(await dialog.ShowDialog<bool>(this)) {
                await session.DeleteWindowAsync(revision,id,preview.WindowDeleteTicket);
                await ChangeWorksheetAsync(session.WorksheetId);
            }
        });
    }
    private async void CopyValuesClick(object? sender,RoutedEventArgs args) {
        if(session is null||Clipboard is null||editing||fileAction||!await CommitAsync())return;
        try {
            long interaction=interactionVersion;string scope=session.WorksheetId,revision=session.Revision;
            var values=await session.CopyValuesAsync(revision,Sheet.SelectionFirst,Sheet.SelectionLast);
            if(interaction!=interactionVersion||scope!=session.WorksheetId||revision!=session.Revision||!values.ClipboardTextAvailable)return;
            await Clipboard.SetTextAsync(values.ClipboardText);StatusLabel.Text="Copied values as text; formulas and reference authority are omitted.";
        }catch(Exception e){StatusLabel.Text=e.Message;}
    }
    private async void PasteTextClick(object? sender,RoutedEventArgs args)=>await ClipboardAsync(true,textOnly:true);
}
