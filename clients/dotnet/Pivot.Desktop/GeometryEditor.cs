using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Layout;
using Pivot.Protocol.Application.V1;

namespace Pivot.Desktop;
public partial class MainWindow
{
    private Window? activeGeometryEditor;
    private async void GeometryClick(object? sender, RoutedEventArgs args)=>await EditGeometryAsync();
    private async Task EditGeometryAsync(GeometryCommand? initial=null,string? initialDescription=null)
    {
        var owner=session;
        if(owner is null||owner.IsBindingProof||owner.IsProjectionProof||editing||fileAction||shuttingDown||!await CommitAsync())return;
        if(!await Sheet.EnsureSectionSelectionAsync()){StatusLabel.Text="Wait for a qualified section selection before editing geometry. "+Sheet.SectionSelectionError;return;}
        var view=geometryView;if(view is null||view.IsDisposed)return;
        string revision=owner.Revision,document=owner.DocumentId,scope=owner.WorksheetId;ulong generation=owner.DocumentGeneration;
        bool windowScope=owner.IsWindowScope;
        bool Current()=>session==owner&&!shuttingDown&&owner.Revision==revision&&owner.DocumentId==document&&owner.DocumentGeneration==generation&&owner.WorksheetId==scope&&owner.IsWindowScope==windowScope;
        refresh?.Cancel();++displayRequest;editing=true;SetEditing(false);
        var commands=new List<GeometryCommand>();
        var descriptions=new List<string>();
        var dialog=new Window {Title="Section geometry",Width=760,Height=680,MinWidth=650,MinHeight=550,WindowStartupLocation=WindowStartupLocation.CenterOwner};
        activeGeometryEditor=dialog;
        var panel=new StackPanel {Margin=new Thickness(20),Spacing=10};
        var orientation=new ComboBox {Name="SectionOrientation",ItemsSource=new[]{"Column widths in a horizontal section","Row heights in a vertical section"},SelectedIndex=0};
        var at=new NumericUpDown {Name="SectionAt",Minimum=1,Maximum=65536,Value=Sheet.Selected.Row,Width=125};
        var first=new NumericUpDown {Name="SectionFirst",Minimum=1,Maximum=1024,Value=Sheet.Selected.Column,Width=125};
        var last=new NumericUpDown {Name="SectionLast",Minimum=1,Maximum=1024,Value=Sheet.Selected.Column,Width=125};
        var size=new NumericUpDown {Name="SectionSize",Minimum=1,Maximum=4096,Value=70.5m,Increment=0.015625m,FormatString="0.######",Width=145};
        var paired=new CheckBox {Name="SectionPaired",Content="Balance the next adjacent size (one divider; preserve their total)",IsChecked=true};
        var shared=new CheckBox {Name="SectionShared",Content="Explicit batch: apply across all perpendicular profiles",IsChecked=false};
        var mode=new TextBlock {Text=view.Frame.Geometry.Positional?"Positional presentation: sizes belong to displayed ordinals, independently of source-field formatting.":"Local presentation: values, formulas and cell formatting are unchanged.",TextWrapping=global::Avalonia.Media.TextWrapping.Wrap};
        var summary=new TextBox {Name="SectionDraft",IsReadOnly=true,AcceptsReturn=true,Height=170,TextWrapping=global::Avalonia.Media.TextWrapping.Wrap};
        var status=new TextBlock {Name="SectionPreviewStatus",Text="Queue edits, then preview the complete batch. Cancel leaves the workbook unchanged.",TextWrapping=global::Avalonia.Media.TextWrapping.Wrap};
        var apply=new Button {Name="SectionApply",Content="Apply previewed batch",IsEnabled=false};
        var preview=new Button {Name="SectionPreview",Content="Preview batch",IsEnabled=false};
        var undo=new Button {Name="SectionUndo",Content="Remove last queued edit",IsEnabled=false};
        bool busy=false;
        void Changed(){apply.IsEnabled=false;preview.IsEnabled=commands.Count>0;undo.IsEnabled=commands.Count>0;summary.Text=string.Join("\n",descriptions);status.Text="Draft only. Preview validates all profiles together; nothing has been published.";}
        void Add(GeometryCommand command,string description){if(busy)return;commands.Add(command);descriptions.Add(description);Changed();}
        uint Rank(NumericUpDown control)=>checked((uint)(control.Value??1));
        bool Horizontal()=>orientation.SelectedIndex==0;
        panel.Children.Add(mode);panel.Children.Add(orientation);
        StackPanel Row(string label,Control control){var row=new StackPanel{Orientation=Orientation.Horizontal,Spacing=10};row.Children.Add(new TextBlock{Text=label,Width=205,VerticalAlignment=VerticalAlignment.Center});row.Children.Add(control);return row;}
        panel.Children.Add(Row("Section containing row / column",at));
        var breakButtons=new StackPanel {Orientation=Orientation.Horizontal,Spacing=10};
        var addBreak=new Button {Name="SectionAddBreak",Content="Queue break before this position"};var removeBreak=new Button {Name="SectionRemoveBreak",Content="Queue removal of this break"};
        addBreak.Click+=(_,_)=>Add(new GeometryCommand{Action=GeometryCommand.Types.Action.AddBreak,Horizontal=Horizontal(),At=Rank(at)},$"Add {(Horizontal()?"horizontal":"vertical")} break before {Rank(at)} (copy preceding profile)");
        removeBreak.Click+=(_,_)=>Add(new GeometryCommand{Action=GeometryCommand.Types.Action.RemoveBreak,Horizontal=Horizontal(),At=Rank(at)},$"Remove {(Horizontal()?"horizontal":"vertical")} break before {Rank(at)}; preceding profile extends");
        breakButtons.Children.Add(addBreak);breakButtons.Children.Add(removeBreak);panel.Children.Add(breakButtons);
        panel.Children.Add(Row("First size position",first));panel.Children.Add(Row("Last size position",last));panel.Children.Add(Row("Size (points)",size));panel.Children.Add(paired);panel.Children.Add(shared);
        var queueSize=new Button {Name="SectionQueueSize",Content="Queue sizing change"};
        queueSize.Click+=(_,_)=>{
            long units=checked((long)((size.Value??1)*64));
            if((size.Value??1)*64!=units){status.Text="Use multiples of 1/64 point.";return;}
            var command=new GeometryCommand {Action=GeometryCommand.Types.Action.SetSizes,Horizontal=Horizontal(),At=Rank(at),First=Rank(first),Last=Rank(last),Size=units,Paired=paired.IsChecked==true,Shared=shared.IsChecked==true};
            Add(command,$"{(command.Horizontal?"Widths":"Heights")} {command.First}–{command.Last}: {units/64m:0.######} pt; {(command.Shared?"ALL profiles":"section at "+command.At)}; {(command.Paired?"balance next size, preserve total":"independent numeric change")}");
        };panel.Children.Add(queueSize);
        orientation.SelectionChanged+=(_,_)=>{bool h=Horizontal();at.Maximum=h?65536:1024;first.Maximum=last.Maximum=h?1024:65536;at.Value=h?Sheet.Selected.Row:Sheet.Selected.Column;first.Value=last.Value=h?Sheet.Selected.Column:Sheet.Selected.Row;size.Value=h?70.5m:19.5m;};
        var resetButtons=new StackPanel {Orientation=Orientation.Horizontal,Spacing=10};
        var resetSizes=new Button {Name="SectionResetSizes",Content="Queue reset sizes (keep breaks)"};var resetAll=new Button {Name="SectionResetAll",Content="Queue reset all geometry"};
        resetSizes.Click+=(_,_)=>Add(new GeometryCommand{Action=GeometryCommand.Types.Action.ResetSizes},"Reset all sizes to defaults; retain section breaks");
        resetAll.Click+=(_,_)=>Add(new GeometryCommand{Action=GeometryCommand.Types.Action.ResetAll},"Remove all section breaks and reset sizes to defaults");
        resetButtons.Children.Add(resetSizes);resetButtons.Children.Add(resetAll);panel.Children.Add(resetButtons);panel.Children.Add(summary);panel.Children.Add(status);
        var actions=new StackPanel {Orientation=Orientation.Horizontal,Spacing=10};var cancel=new Button{Name="SectionCancel",Content="Cancel",IsCancel=true};cancel.Click+=(_,_)=>dialog.Close();
        undo.Click+=(_,_)=>{if(busy||commands.Count==0)return;commands.RemoveAt(commands.Count-1);descriptions.RemoveAt(descriptions.Count-1);Changed();};
        preview.Click+=async(_,_)=>{
            if(busy)return;busy=true;preview.IsEnabled=apply.IsEnabled=false;
            try{if(!Current())throw new InvalidOperationException("Workbook context changed. Reopen the geometry editor.");var result=await owner.PreviewGeometryAsync(revision,commands);if(!Current())throw new InvalidOperationException("Workbook changed during preview.");summary.Text=string.Join("\n",descriptions)+"\n\nExact resulting profiles (points):\n"+string.Join("\n",result.GeometryProfiles.Select(p=>$"{(p.Horizontal?"Widths at row":"Heights at column")} {p.At}: "+string.Join("; ",p.Sizes.Select(r=>$"{r.First}–{r.Last} = {r.Size/64m:0.######}"))));status.Text="Complete batch is valid, including Blank cells and current filter compaction. Apply publishes these exact queued changes.";apply.IsEnabled=true;}
            catch(Exception error){status.Text=error.Message;}finally{busy=false;preview.IsEnabled=commands.Count>0;}
        };
        apply.Click+=async(_,_)=>{
            if(busy||!apply.IsEnabled)return;busy=true;apply.IsEnabled=preview.IsEnabled=false;
            try{if(!Current())throw new InvalidOperationException("Workbook context changed. Preview again in a new editor.");await owner.EditGeometryAsync(revision,commands);dialog.Close();}
            catch(Exception error){status.Text=error.Message;busy=false;preview.IsEnabled=true;}
        };
        actions.Children.Add(undo);actions.Children.Add(preview);actions.Children.Add(apply);actions.Children.Add(cancel);panel.Children.Add(actions);
        if(initial is not null)Add(initial.Clone(),initialDescription??"Balanced adjacent-divider edit");
        dialog.Content=new ScrollViewer {Content=panel};
        try{await dialog.ShowDialog(this);}finally{activeGeometryEditor=null;editing=false;SetEditing(owner.DocumentId.Length!=0);await RefreshAsync();Sheet.Focus();}
    }
}
