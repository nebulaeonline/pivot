using System;
using System.Globalization;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Layout;
using Pivot.Client;
using Pivot.Protocol.Application.V1;

namespace Pivot.Desktop;
internal sealed class BoundWindowDialog:Window {
    internal readonly TextBox NameEditor=new(){MaxLength=64,PlaceholderText="Window name"};
    internal readonly ComboBox Mode=new(){ItemsSource=new[]{"Live worksheet range","Fixture snapshot","Existing producer"},SelectedIndex=0};
    internal readonly ComboBox Worksheet=new();
    internal readonly TextBox First=new(){Text="A1"},Last=new(){Text="A1"};
    internal readonly ComboBox Fixture=new(){ItemsSource=new[]{"ledger","typed"},SelectedIndex=0};
    internal readonly CheckBox GenerationQualified=new(){Content="Use generation-qualified item identities"};
    internal readonly ComboBox Existing=new();
    internal readonly TextBlock Preview=new(){TextWrapping=global::Avalonia.Media.TextWrapping.Wrap};
    internal readonly TextBlock Error=new(){TextWrapping=global::Avalonia.Media.TextWrapping.Wrap};
    internal readonly Button Create=new(){Content="Create bound Window"};
    private readonly WorkbookSession session;
    private readonly string expected;
    private readonly StackPanel sourceRange=new(){Spacing=6},fixtureOptions=new(){Spacing=6};
    private bool busy;
    public string CreatedWindow {get;private set;}="";
    private sealed record Choice(string Id,string Label){public override string ToString()=>Label;}
    public BoundWindowDialog(WorkbookSession session,string? worksheet,Position first,Position last) {
        this.session=session;expected=session.Revision;
        Title="Bind a Grid Window";Width=570;SizeToContent=SizeToContent.Height;WindowStartupLocation=WindowStartupLocation.CenterOwner;
        Worksheet.ItemsSource=session.Worksheets.Select(s=>new Choice(s.Id,s.Name)).ToArray();
        Worksheet.SelectedIndex=Math.Max(0,session.Worksheets.ToList().FindIndex(s=>s.Id==worksheet));
        Existing.ItemsSource=session.Producers.Where(p=>!p.Retired).Select(p=>new Choice(p.Id,$"{p.Kind} · {(p.Fixture.Length==0?session.Worksheets.FirstOrDefault(s=>s.Id==p.SourceWorksheet)?.Name:p.Fixture)} · {p.Rows:N0} × {p.Columns:N0}")).ToArray();Existing.SelectedIndex=0;
        First.Text=Pivot.Avalonia.WorksheetView.Address(first);Last.Text=Pivot.Avalonia.WorksheetView.Address(last);
        var body=new StackPanel{Margin=new Thickness(16),Spacing=10};body.Children.Add(NameEditor);body.Children.Add(Mode);
        sourceRange.Children.Add(new TextBlock{Text="Source worksheet"});sourceRange.Children.Add(Worksheet);
        var endpoints=new Grid{ColumnDefinitions=new ColumnDefinitions("*,*")};endpoints.Children.Add(First);Grid.SetColumn(Last,1);endpoints.Children.Add(Last);sourceRange.Children.Add(new TextBlock{Text="First and last physical source positions"});sourceRange.Children.Add(endpoints);body.Children.Add(sourceRange);
        fixtureOptions.Children.Add(Fixture);fixtureOptions.Children.Add(GenerationQualified);body.Children.Add(fixtureOptions);body.Children.Add(Existing);body.Children.Add(Preview);body.Children.Add(Error);
        var buttons=new StackPanel{Orientation=Orientation.Horizontal,Spacing=8};var cancel=new Button{Content="Cancel"};cancel.Click+=(_,_)=>Close(false);buttons.Children.Add(Create);buttons.Children.Add(cancel);body.Children.Add(buttons);Content=body;
        Mode.SelectionChanged+=(_,_)=>UpdatePreview();Worksheet.SelectionChanged+=(_,_)=>UpdatePreview();Fixture.SelectionChanged+=(_,_)=>UpdatePreview();Existing.SelectionChanged+=(_,_)=>UpdatePreview();GenerationQualified.IsCheckedChanged+=(_,_)=>UpdatePreview();
        NameEditor.TextChanged+=(_,_)=>UpdatePreview();First.TextChanged+=(_,_)=>UpdatePreview();Last.TextChanged+=(_,_)=>UpdatePreview();Create.Click+=async(_,_)=>await CreateAsync();UpdatePreview();
    }
    private static Position? Parse(string? text) {
        text=text?.Trim().ToUpperInvariant();if(string.IsNullOrEmpty(text))return null;int i=0;uint column=0;
        while(i<text.Length&&text[i] is >= 'A' and <= 'Z'){if(column>1024)return null;column=column*26+(uint)(text[i++]-'A'+1);}
        if(i==0||i==text.Length||column>1024||!uint.TryParse(text.AsSpan(i),NumberStyles.None,CultureInfo.InvariantCulture,out uint row)||row is 0 or >65536)return null;
        return new Position{Row=row,Column=column};
    }
    private void UpdatePreview() {
        bool worksheet=Mode.SelectedIndex==0,fixture=Mode.SelectedIndex==1;sourceRange.IsVisible=worksheet;fixtureOptions.IsVisible=fixture;Existing.IsVisible=!worksheet&&!fixture;
        string name=NameEditor.Text??"";bool valid=!string.IsNullOrWhiteSpace(name)&&name==name.Trim()&&!name.Any(char.IsControl)&&name.IndexOfAny("[]:*?/\\!|".ToCharArray())<0;
        if(worksheet){var first=Parse(First.Text);var last=Parse(Last.Text);valid&=Worksheet.SelectedItem is Choice&&first is not null&&last is not null&&first.Row<=last.Row&&first.Column<=last.Column;Preview.Text=first is not null&&last is not null&&first.Row<=last.Row&&first.Column<=last.Column?$"Live read-only projection of {(last.Row-first.Row+1):N0} × {(last.Column-first.Column+1):N0} physical positions. Blank and hidden positions remain included. Committed source changes update automatically; there is no Refresh step.":"Choose a finite worksheet rectangle.";}
        else if(fixture){Preview.Text="Read-only deterministic fixture. Values stay unavailable until you explicitly Refresh. Selected snapshots are stored with the workbook; reopening never fetches data. "+(GenerationQualified.IsChecked==true?"A new generation does not inherit old item references.":"This fixture declares durable keys; references follow the same keyed item across refreshes.");}
        else{valid&=Existing.SelectedItem is Choice;Preview.Text="A second read-only view shares the selected producer and its generation. It does not copy values or introduce independent refresh authority.";}
        Create.IsEnabled=valid&&!busy;
    }
    private async System.Threading.Tasks.Task CreateAsync() {
        if(busy||!Create.IsEnabled)return;busy=true;Create.IsEnabled=false;Error.Text="";
        try{
            string producer,revision=expected;
            if(Mode.SelectedIndex==0){var result=await session.CreateWorksheetProducerAsync(expected,((Choice)Worksheet.SelectedItem!).Id,Parse(First.Text)!,Parse(Last.Text)!);producer=result.ProducerId;revision=result.Revision;}
            else if(Mode.SelectedIndex==1){var result=await session.CreateFixtureProducerAsync(expected,(string)Fixture.SelectedItem!,GenerationQualified.IsChecked==true);producer=result.ProducerId;revision=result.Revision;}
            else producer=((Choice)Existing.SelectedItem!).Id;
            var created=await session.CreateBoundWindowAsync(revision,NameEditor.Text??"",producer);CreatedWindow=created.Worksheet;Close(true);
        }catch(Exception e){Error.Text=e.Message;}finally{busy=false;UpdatePreview();}
    }
}
