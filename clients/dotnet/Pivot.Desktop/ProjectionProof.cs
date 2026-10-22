using System;
using System.Globalization;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Pivot.Avalonia;
namespace Pivot.Desktop;
public partial class MainWindow
{
    private void InitializeProofInteractions()
    {
        Sheet.ProjectedSelectionChanged+=()=>{++interactionVersion;SelectCell();SetEditing(session?.DocumentId.Length>0 && !editing);};
        Sheet.ProjectionRequested=async target=>{if(editing||fileAction||shuttingDown)return;if(await CommitAsync()){Sheet.ApplyProjectionTarget(target);Sheet.Focus();}};
    }
    private bool ShowProjectedSelection()
    {
        if(Sheet.ProjectedSelection is not { } at)return false;
        AddressLabel.Text=at.Key.Length==0?"Source":at.Key;
        SelectionLabel.Text=$"Read-only projection {at.Projection} · Record {at.Key} · Generation {at.Generation}";
        synchronizing=true;FormulaEntry.Text="";synchronizing=false;
        StatusLabel.Text=Sheet.MissingProjectedRecord?"Selected record is missing; projection header selected.":"Projected records are read-only. Escape returns to the worksheet.";
        return true;
    }
    private async Task ChangeProofAsync(Func<Task> action)
    {
        if(session is null || !session.IsProjectionProof || editing || fileAction || shuttingDown || !await CommitAsync())return;
        ++interactionVersion;editing=true;SetEditing(false);
        try{await action();await RefreshAsync();UpdateDocument();}
        catch(Exception e){StatusLabel.Text=e.Message;}
        finally{editing=false;SetEditing(session.DocumentId.Length>0);}
    }
    private async void ProofPublishClick(object? sender,RoutedEventArgs e)
    {if(sender is Button b && uint.TryParse(b.Tag?.ToString(),NumberStyles.None,CultureInfo.InvariantCulture,out uint count))await ChangeProofAsync(async()=>{await session!.PublishProofAsync(count);});}
    private async void ProofRemoveClick(object? sender,RoutedEventArgs e)=>await ChangeProofAsync(async()=>{await session!.PublishProofAsync(3,"L2");});
    private async void ProofReverseClick(object? sender,RoutedEventArgs e)
    {if(Sheet.ProjectionFrame is { } f){var p=f.Placements[0];await ChangeProofAsync(async()=>{await session!.LayoutProofAsync(p.Id,p.AfterRow,!p.Reverse);});}}
    private async void ProofMoveClick(object? sender,RoutedEventArgs e)
    {if(Sheet.ProjectionFrame is { } f){var p=f.Placements[0];await ChangeProofAsync(async()=>{await session!.LayoutProofAsync(p.Id,p.AfterRow==8?4u:8u,p.Reverse);});}}
}
