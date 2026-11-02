using System;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Layout;
using Pivot.Protocol.Application.V1;

namespace Pivot.Desktop;
public partial class MainWindow {
    private ProducerInfo? SelectedProducer => session?.Windows.FirstOrDefault(w=>w.Id==session.WorksheetId) is {Bound:true} window
        ? session.Producers.FirstOrDefault(p=>p.Id==window.Producer) : null;
    private void UpdateBoundControls() {
        var p=SelectedProducer;
        BoundSourceButton.IsEnabled=DetachWindowButton.IsEnabled=p is not null;
        RefreshProducerButton.IsVisible=p?.Kind=="Explicit refresh";
        RefreshProducerButton.IsEnabled=p is {Retired:false,RefreshStatus:not "Disconnected"};
        BindWindowButton.IsEnabled=session is not null && session.Windows.Count<64;
        BoundStatusLabel.IsVisible=p is not null;
        if(p is null)return;
        string source=p.Kind=="Live worksheet" ? $"{session!.Worksheets.FirstOrDefault(s=>s.Id==p.SourceWorksheet)?.Name??"Retired worksheet"}!{Address(p.First)}:{Address(p.Last)}" : $"Fixture {p.Fixture}";
        BoundStatusLabel.Text=$"Read-only · {p.Kind} · {source} · {p.Rows:N0} × {p.Columns:N0} · {p.RefreshStatus}"
            +(p.SelectedGeneration.Length==0?"":$" · Selected generation {p.SelectedGeneration}")
            +(p.LastOutcome is "Failed; retained snapshot" or "Interrupted on reopen"?$" · Last attempt: {p.LastOutcome}":"");
    }
    private static string Address(Position? p)=>p is null?"unavailable":Pivot.Avalonia.WorksheetView.Address(p);
    private async void BindWindowClick(object? sender,RoutedEventArgs args) {
        if(session is null)return;
        await FileActionAsync(async()=>{
            var dialog=new BoundWindowDialog(session,session.IsWindowScope?null:session.WorksheetId,Sheet.SelectionFirst,Sheet.SelectionLast);
            if(await dialog.ShowDialog<bool>(this))await ChangeWorksheetAsync(dialog.CreatedWindow);
        });
    }
    private async void RefreshProducerClick(object? sender,RoutedEventArgs args) {
        if(session is null || SelectedProducer is not { } p)return;
        await FileActionAsync(async()=>{await session.RefreshProducerAsync(session.Revision,p.Id);await RefreshAsync();});
    }
    private async void BoundSourceClick(object? sender,RoutedEventArgs args) {
        if(session is null || SelectedProducer is not { } p)return;
        await FileActionAsync(async()=>{
            var dialog=new Window{Title="Bound source",Width=560,SizeToContent=SizeToContent.Height,WindowStartupLocation=WindowStartupLocation.CenterOwner};
            var body=new StackPanel{Margin=new Thickness(16),Spacing=10};
            body.Children.Add(new TextBlock{Text=BoundStatusLabel.Text,TextWrapping=global::Avalonia.Media.TextWrapping.Wrap});
            if(p.Kind=="Explicit refresh") {
                body.Children.Add(new TextBlock{Text=$"{(p.GenerationQualified?"Generation-qualified keys: a new generation does not inherit old references.":"Durable keys: references follow the same record across successful refreshes.")}\nSnapshots are saved in this workbook. Opening the file never refreshes it.",TextWrapping=global::Avalonia.Media.TextWrapping.Wrap});
                var version=new ComboBox{ItemsSource=new[]{"Initial fixture","Reordered fixture with changed amounts","Fixture with first item absent","Unavailable fixture"},SelectedIndex=0};body.Children.Add(version);
                var refreshButton=new Button{Content="Refresh selected fixture",IsEnabled=!p.Retired&&p.RefreshStatus!="Disconnected"};
                refreshButton.Click+=async(_,_)=>{refreshButton.IsEnabled=false;try{await session.RefreshProducerAsync(session.Revision,p.Id,(uint)version.SelectedIndex);dialog.Close();}catch(Exception e){StatusLabel.Text=e.Message;dialog.Close();}};body.Children.Add(refreshButton);
                var connection=new Button{Content=p.RefreshStatus=="Disconnected"?"Reconnect locally":"Disconnect",IsEnabled=!p.Retired};
                connection.Click+=async(_,_)=>{connection.IsEnabled=false;try{if(p.RefreshStatus=="Disconnected")await session.ReconnectProducerAsync(session.Revision,p.Id,p.Fixture);else await session.DisconnectProducerAsync(session.Revision,p.Id);dialog.Close();}catch(Exception e){StatusLabel.Text=e.Message;dialog.Close();}};body.Children.Add(connection);
            } else body.Children.Add(new TextBlock{Text="Committed source changes appear automatically. Hidden positions remain included in physical order. Edit values, formatting, sorting and filters in the source worksheet.",TextWrapping=global::Avalonia.Media.TextWrapping.Wrap});
            if(p.Fields.Count!=0)body.Children.Add(new TextBlock{Text=string.Join(" · ",p.Fields.Select(f=>$"{f.Label}: {f.Kind}{(f.Nullable?" / Blank":"")}")),TextWrapping=global::Avalonia.Media.TextWrapping.Wrap});
            var remove=new Button{Content="Delete producer…",IsEnabled=!p.Retired};
            remove.Click+=async(_,_)=>{
                remove.IsEnabled=false;
                try{
                    var preview=await session.PrepareProducerDeleteAsync(session.Revision,p.Id);string expected=session.Revision;
                    var confirm=new Window{Title="Delete producer",Width=460,SizeToContent=SizeToContent.Height,WindowStartupLocation=WindowStartupLocation.CenterOwner};var panel=new StackPanel{Margin=new Thickness(16),Spacing=12};
                    panel.Children.Add(new TextBlock{Text=$"Delete this producer? Its bound views become unavailable and {preview.ProducerInboundReferences:N0} retained inbound reference occurrence(s) will be faulted. Source worksheet cells are preserved.",TextWrapping=global::Avalonia.Media.TextWrapping.Wrap});
                    var buttons=new StackPanel{Orientation=Orientation.Horizontal,Spacing=8};var yes=new Button{Content="Delete"};var no=new Button{Content="Cancel"};yes.Click+=(_,_)=>confirm.Close(true);no.Click+=(_,_)=>confirm.Close(false);buttons.Children.Add(yes);buttons.Children.Add(no);panel.Children.Add(buttons);confirm.Content=panel;
                    if(await confirm.ShowDialog<bool>(dialog)){await session.DeleteProducerAsync(expected,p.Id,preview.ProducerDeleteTicket);dialog.Close();}else remove.IsEnabled=true;
                }catch(Exception e){StatusLabel.Text=e.Message;dialog.Close();}
            };body.Children.Add(remove);
            var close=new Button{Content="Close",HorizontalAlignment=HorizontalAlignment.Right};close.Click+=(_,_)=>dialog.Close();body.Children.Add(close);dialog.Content=body;
            await dialog.ShowDialog(this);worksheetTabSignature="";await RefreshAsync();
        });
    }
    private async void DetachWindowClick(object? sender,RoutedEventArgs args) {
        if(session?.IsReadOnly!=true)return;
        await FileActionAsync(async()=>{
            var dialog=new Window{Title="Detach a copy",Width=480,SizeToContent=SizeToContent.Height,WindowStartupLocation=WindowStartupLocation.CenterOwner};var panel=new StackPanel{Margin=new Thickness(16),Spacing=12};
            panel.Children.Add(new TextBlock{Text="Create a new unbound Window from the selected snapshot's exact values and built-in formatting. Existing references keep their original bound source. Up to 256 positions can be detached at once.",TextWrapping=global::Avalonia.Media.TextWrapping.Wrap});
            var name=new TextBox{PlaceholderText="New Window name",MaxLength=64};panel.Children.Add(name);var create=new Button{Content="Create detached copy"};create.Click+=(_,_)=>dialog.Close(true);panel.Children.Add(create);dialog.Content=panel;
            if(await dialog.ShowDialog<bool>(this)){var result=await session.DetachBoundWindowAsync(session.Revision,session.WorksheetId,name.Text??"");await ChangeWorksheetAsync(result.Worksheet);}
        });
    }
}
