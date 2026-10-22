using System;
using Avalonia.Interactivity;
using Pivot.Protocol.Application.V1;
namespace Pivot.Desktop;
public partial class MainWindow {
    private bool bindingRenamed;
    private async void BindingUnavailableClick(object? sender,RoutedEventArgs e)=>await ChangeProofAsync(async()=>{await session!.PublishUnavailableAsync();});
    private async void BindingRenameClick(object? sender,RoutedEventArgs e)=>await ChangeProofAsync(async()=>{
        await session!.RenameProducerAsync(bindingRenamed?"InvoiceLines":"RenamedLines");bindingRenamed=!bindingRenamed;
    });
    private async void BindingDuplicateClick(object? sender,RoutedEventArgs e)=>await ChangeProofAsync(async()=>{
        var first=Sheet.SelectionFirst;var last=Sheet.SelectionLast;
        await session!.DuplicateBlockAsync(first,last,new Position{Row=12,Column=2},new Position{Row=12+last.Row-first.Row,Column=2+last.Column-first.Column});
    });
}
