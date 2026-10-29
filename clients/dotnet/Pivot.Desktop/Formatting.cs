using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
namespace Pivot.Desktop;
public partial class MainWindow {
    private bool loadingFormat;
    private uint formatTouched;
    private string formatRevision="",formatWorksheet="";
    private long formatRequest;
    private Position? formatFirst,formatLast;
    private readonly List<ComboBox> formatCombos=[];
    private readonly uint?[] palette=[null,0x000000,0xffffff,0x1f4e78,0x008000,0xc00000,0xffeb9c,0xddebf7,0xe2efda];
    private TextBox formatPlaces=null!;
    private void InitializeFormatting() {
        var row=new WrapPanel {Orientation=global::Avalonia.Layout.Orientation.Horizontal};FormatFields.Children.Add(row);
        ComboBox Add(string label,string[] items,int field) {
            var box=new ComboBox {Width=155,ItemsSource=new[]{"Mixed"}.Concat(items).ToArray(),SelectedIndex=0};
            var column=new StackPanel {Margin=new global::Avalonia.Thickness(0,0,8,4)};
            column.Children.Add(new TextBlock{Text=label});column.Children.Add(box);row.Children.Add(column);
            box.SelectionChanged+=(_,_)=>{if(!loadingFormat) {formatTouched|=1u<<field;
                if(field==0 && box.SelectedIndex>0) {loadingFormat=true;formatPlaces.Text="2";formatCombos[2].SelectedIndex=box.SelectedIndex==3?2:1;formatCombos[3].SelectedIndex=1;loadingFormat=false;formatTouched|=2|8|16;}}};
            formatCombos.Add(box);return box;
        }
        Add("Numeric format",["General","Number","Currency","Percentage"],0);
        var placesColumn=new StackPanel{Margin=new global::Avalonia.Thickness(0,0,8,4)};
        placesColumn.Children.Add(new TextBlock{Text="Decimal places (0–28)"});formatPlaces=new TextBox{Width=155,PlaceholderText="Mixed",MaxLength=2};placesColumn.Children.Add(formatPlaces);row.Children.Add(placesColumn);
        formatPlaces.TextChanged+=(_,_)=>{if(!loadingFormat)formatTouched|=2;};
        // Combo list index 1 is rounding (field 2); the helpers below keep wire masks explicit.
        Add("Display rounding",["Nearest, ties to even","Nearest, ties away","Nearest, ties toward zero","Toward zero","Away from zero","Toward −infinity","Toward +infinity"],2);
        Add("Grouping",["Off","On"],3);
        Add("Currency symbol",["Dollar ($)","Euro (€)","Pound (£)"],4);
        Add("Bold",["Off","On"],5);Add("Italic",["Off","On"],6);
        Add("Alignment",["Auto","Left","Center","Right"],7);
        string[] colors=["Theme","Black","White","Navy","Green","Red","Pale yellow","Pale blue","Pale green"];
        Add("Foreground",colors,8);Add("Background",colors,9);
        var buttons=new StackPanel{Orientation=global::Avalonia.Layout.Orientation.Horizontal,Spacing=8};
        var apply=new Button{Content="Apply format"};var reset=new Button{Content="Reset Formatting"};var cancel=new Button{Content="Cancel"};
        apply.Click+=async(_,_)=>await ApplyFormatting(false);reset.Click+=async(_,_)=>await ApplyFormatting(true);cancel.Click+=(_,_)=>{formatTouched=0;FormatPane.IsExpanded=false;Sheet.Focus();};
        buttons.Children.Add(apply);buttons.Children.Add(reset);buttons.Children.Add(cancel);FormatFields.Children.Add(buttons);
        FormatPane.PropertyChanged+=async(_,e)=>{if(e.Property==Expander.IsExpandedProperty && FormatPane.IsExpanded)await LoadFormatting();};
        Sheet.SelectionChanged+=()=>{if(FormatPane.IsExpanded)_=LoadFormatting();};
    }
    private async Task LoadFormatting() {
        if(session is null || session.DocumentId.Length==0)return;
        long request=++formatRequest;var s=session;var first=Sheet.SelectionFirst;var last=Sheet.SelectionLast;string sheet=s.WorksheetId,revision=s.Revision;
        formatRevision="";
        try {
            var response=await s.ReadStylesAsync(first,last);
            if(request!=formatRequest || s!=session || s.WorksheetId!=sheet || s.Revision!=revision || !Same(first,Sheet.SelectionFirst)||!Same(last,Sheet.SelectionLast))return;
            var values=response.Styles.Select(p=>CellStyle.Decode(p.Style)).ToArray();
            if(values.Length==0)return;
            loadingFormat=true;
            int Mixed(Func<CellStyle,int> selector)=>values.All(v=>selector(v)==selector(values[0]))?selector(values[0])+1:0;
            formatCombos[0].SelectedIndex=Mixed(v=>v.Numeric);
            formatPlaces.Text=values.All(v=>v.Places==values[0].Places)?values[0].Places.ToString():"";
            formatCombos[1].SelectedIndex=Mixed(v=>v.Rounding);
            formatCombos[2].SelectedIndex=Mixed(v=>v.Grouping?1:0);
            formatCombos[3].SelectedIndex=Mixed(v=>v.Currency);
            formatCombos[4].SelectedIndex=Mixed(v=>v.Bold?1:0);
            formatCombos[5].SelectedIndex=Mixed(v=>v.Italic?1:0);
            formatCombos[6].SelectedIndex=Mixed(v=>v.Alignment);
            for(int i=0;i<2;i++) {uint? color=i==0?values[0].Foreground:values[0].Background;bool mixed=values.Any(v=>(i==0?v.Foreground:v.Background)!=color);int index=Array.IndexOf(palette,color);formatCombos[7+i].SelectedIndex=mixed||index<0?0:index+1;}
            formatTouched=0;formatRevision=revision;formatWorksheet=sheet;formatFirst=first;formatLast=last;
        }catch(Exception e){StatusLabel.Text=e.Message;}
        finally{loadingFormat=false;}
    }
    private static bool Same(Position a,Position b)=>a.Row==b.Row&&a.Column==b.Column;
    private async Task ApplyFormatting(bool reset) {
        if(session is null||editing||fileAction||shuttingDown||Sheet.IsSelecting||Sheet.ProjectedSelection is not null)return;
        if(formatRevision!=session.Revision) {StatusLabel.Text="Reload the format pane after workbook changes.";return;}
        if(!await CommitAsync())return;
        // Committing a draft changes the revision; retain explicit field choices but bind to this exact state.
        if(formatFirst is null||formatLast is null||formatWorksheet!=session.WorksheetId||!Same(formatFirst,Sheet.SelectionFirst)||!Same(formatLast,Sheet.SelectionLast)) {StatusLabel.Text="Reload the format pane for the current selection.";return;}
        if(!reset && formatTouched==0)return;
        try {
            int Value(int combo,int field) {int i=formatCombos[combo].SelectedIndex;if((formatTouched&(1u<<field))!=0&&i<=0)throw new ArgumentException("Choose a value for each changed field.");return Math.Max(0,i-1);}
            byte places=2;if(!reset && (formatTouched&2)!=0 && (!byte.TryParse(formatPlaces.Text,out places)||places>28))throw new ArgumentException("Decimal places must be 0–28.");
            var style=reset ? new CellStyle() : new CellStyle((byte)Value(0,0),places,(byte)Value(1,2),Value(2,3)==1,(byte)Value(3,4),Value(4,5)==1,Value(5,6)==1,(byte)Value(6,7),palette[Value(7,8)],palette[Value(8,9)]);
            editing=true;SetEditing(false);refresh?.Cancel();++displayRequest;
            await session.FormatAsync(session.Revision,formatFirst,formatLast,style,formatTouched,reset);
            await RefreshAsync();await LoadFormatting();StatusLabel.Text=reset?"Formatting reset; contents retained.":"Formatting applied.";
        }catch(Exception e){StatusLabel.Text=e.Message;}
        finally{editing=false;SetEditing(session.DocumentId.Length!=0);Sheet.Focus();}
    }
}
