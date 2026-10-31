using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls.Primitives;
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
        RememberWorksheetView();session.SelectWorksheet(id);
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
        string signature=session.DocumentId+"|"+session.WorksheetId+"|"+string.Join("|",session.Worksheets.Select(s=>s.Id+":"+s.Name));
        if(signature==worksheetTabSignature)return;
        worksheetTabSignature=signature;WorksheetTabs.Children.Clear();
        foreach(var sheet in session.Worksheets) {
            var tab=new ToggleButton{Content=sheet.Name,Tag=sheet.Id,IsChecked=sheet.Id==session.WorksheetId};
            tab.Click+=async(_,_)=>await SwitchWorksheetAsync(sheet.Id);WorksheetTabs.Children.Add(tab);
        }
        WorksheetNameEditor.Text=session.Worksheets.FirstOrDefault(s=>s.Id==session.WorksheetId)?.Name??"";
        AddWorksheetButton.IsEnabled=session.Worksheets.Count<16;
    }
}
