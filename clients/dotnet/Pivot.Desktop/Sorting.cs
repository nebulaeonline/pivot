using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Pivot.Avalonia;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
namespace Pivot.Desktop;
internal sealed record SortChoice(Position First,Position Last,bool Header,bool MoveFormatting,SortKey[] Keys);
internal sealed class SortDialog:Window {
 internal readonly TextBox RangeEditor=new(){PlaceholderText="A1:D100"};
 internal readonly Button ReviewButton=new(){Content="Review range"},ExpandButton=new(){Content="Include adjacent columns",IsVisible=false},AddButton=new(){Content="Add key"},ApplyButton=new(){Content="Sort",IsEnabled=false},CancelButton=new(){Content="Cancel"};
 internal readonly CheckBox HeaderOption=new(){Content="Exclude the first row (header)",IsChecked=false},FormatOption=new(){Content="Move formatting with records",IsChecked=true};
 internal readonly TextBlock RangeLabel=new(),Warning=new(){TextWrapping=global::Avalonia.Media.TextWrapping.Wrap},ErrorLabel=new(){TextWrapping=global::Avalonia.Media.TextWrapping.Wrap};
 internal readonly StackPanel LevelsPanel=new(){Spacing=6};
 private sealed class Level {internal uint Column;internal bool Descending;}
 private readonly List<Level> levels=[];
 private readonly WorkbookSession session;
 private readonly string expected;
 private Position? first,last,suggestedFirst,suggestedLast;
 private bool busy,reviewed;
 private int epoch;
 internal SortDialog(WorkbookSession session,string expected,Position first,Position last){
  this.session=session;this.expected=expected;
  Title="Sort worksheet records";Width=710;Height=650;MinWidth=650;MinHeight=550;WindowStartupLocation=WindowStartupLocation.CenterOwner;
  var panel=new StackPanel{Margin=new Thickness(18),Spacing=12};
  panel.Children.Add(new TextBlock{Text="Worksheet range",FontWeight=global::Avalonia.Media.FontWeight.Bold});
  var rangeRow=new Grid{ColumnDefinitions=new ColumnDefinitions("*,Auto"),ColumnSpacing=8};rangeRow.Children.Add(RangeEditor);Grid.SetColumn(ReviewButton,1);rangeRow.Children.Add(ReviewButton);panel.Children.Add(rangeRow);
  panel.Children.Add(RangeLabel);panel.Children.Add(Warning);panel.Children.Add(ExpandButton);panel.Children.Add(HeaderOption);
  panel.Children.Add(new TextBlock{Text="Sort by, then by (top key has priority)"});
  panel.Children.Add(new ScrollViewer{Content=LevelsPanel,MaxHeight=200});panel.Children.Add(AddButton);panel.Children.Add(FormatOption);
  panel.Children.Add(new TextBlock{Text="Unchecked: keep formatting at worksheet positions. Every selected column moves with its record segment.",TextWrapping=global::Avalonia.Media.TextWrapping.Wrap});
  panel.Children.Add(new TextBlock{Text="Ascending: numbers, text, Booleans. Text is case-sensitive ordinal Unicode; leading-zero Strings stay exact. Errors then Blanks remain last in either direction. Equal keys keep their order. Formula keys use values before sorting; recalculation may change them. Scalar references follow cells; SUM(A1:A1) reads the current occupant of A1.",TextWrapping=global::Avalonia.Media.TextWrapping.Wrap});
  panel.Children.Add(ErrorLabel);var buttons=new StackPanel{Orientation=global::Avalonia.Layout.Orientation.Horizontal,Spacing=12};buttons.Children.Add(ApplyButton);buttons.Children.Add(CancelButton);panel.Children.Add(buttons);Content=new ScrollViewer{Content=panel};
  // TextChanged is posted after rendering. It can otherwise invalidate a review
  // started by the same range edit after that review has already been dispatched.
  RangeEditor.TextChanging+=(_,_)=>{reviewed=false;++epoch;RangeLabel.Text="Review and confirm the final range before sorting.";ExpandButton.IsVisible=false;UpdateButtons();};
  RangeEditor.Text=first.Equals(last)?WorksheetView.Address(first):RangeText(first,last);
  ReviewButton.Click+=async(_,_)=>await ReviewAsync();
  ExpandButton.Click+=async(_,_)=>{if(suggestedFirst is null||suggestedLast is null)return;RangeEditor.Text=RangeText(suggestedFirst,suggestedLast);await ReviewAsync();};
  AddButton.Click+=(_,_)=>{if(this.first is null||this.last is null)return;for(uint c=this.first.Column;c<=this.last.Column;c++)if(!levels.Any(l=>l.Column==c)){levels.Add(new Level{Column=c});BuildLevels();break;}};
  ApplyButton.Click+=(_,_)=>{if(!ApplyButton.IsEnabled||this.first is null||this.last is null)return;Close(new SortChoice(this.first.Clone(),this.last.Clone(),HeaderOption.IsChecked==true,FormatOption.IsChecked==true,levels.Select(l=>new SortKey{Column=l.Column,Descending=l.Descending}).ToArray()));};
  CancelButton.Click+=(_,_)=>Close();UpdateButtons();
 }
 internal static string RangeText(Position first,Position last)=>WorksheetView.Address(first)+":"+WorksheetView.Address(last);
 internal async Task ReviewAsync(){
  int request=++epoch;busy=true;reviewed=false;UpdateButtons();
  try{
   var parts=(RangeEditor.Text??"").Split(':');if(parts.Length!=2)throw new FormatException("Enter an explicit range, such as A1:D100, then review it.");
   var a=WorksheetView.ParseAddress(parts[0]);var b=WorksheetView.ParseAddress(parts[1]);if(a.Row>b.Row||a.Column>b.Column)throw new FormatException("The range must run from its top-left to bottom-right corner.");
   var response=await session.SortBoundsAsync(expected,a,b);if(request!=epoch)return;
   first=a;last=b;suggestedFirst=response.SortSuggestedFirst;suggestedLast=response.SortSuggestedLast;
   bool adjacent=!a.Equals(suggestedFirst)||!b.Equals(suggestedLast);
   RangeLabel.Text="Final sort range: "+RangeText(a,b);
   Warning.Text=adjacent?"Adjacent populated columns are outside this range. Sorting only these columns can separate related records. Include adjacent columns below, or deliberately sort the displayed range.":"Only the displayed rectangle moves. Cells outside it stay at their worksheet positions. Check that the range includes every column belonging to each record.";
   ExpandButton.IsVisible=adjacent;ExpandButton.Content="Include adjacent columns: "+RangeText(suggestedFirst!,suggestedLast!);
   levels.RemoveAll(l=>l.Column<a.Column||l.Column>b.Column);if(levels.Count==0)levels.Add(new Level{Column=a.Column});
   reviewed=true;ErrorLabel.Text="";BuildLevels();
  }catch(Exception e){if(request==epoch)ErrorLabel.Text=e.Message;}
  finally{busy=false;UpdateButtons();}
 }
 private void BuildLevels(){
  LevelsPanel.Children.Clear();if(first is null||last is null)return;
  uint firstColumn=first.Column;
  var columns=Enumerable.Range((int)firstColumn,(int)(last.Column-firstColumn+1)).Select(c=>WorksheetView.Address(new Position{Row=1,Column=(uint)c}).TrimEnd('1')).ToArray();
  for(int i=0;i<levels.Count;i++){
   int index=i;var level=levels[i];var row=new StackPanel{Orientation=global::Avalonia.Layout.Orientation.Horizontal,Spacing=6};
   var column=new ComboBox{ItemsSource=columns,SelectedIndex=(int)(level.Column-firstColumn),Width=130};var direction=new ComboBox{ItemsSource=new[]{"Ascending","Descending"},SelectedIndex=level.Descending?1:0,Width=140};
   column.SelectionChanged+=(_,_)=>{if(column.SelectedIndex>=0)level.Column=firstColumn+(uint)column.SelectedIndex;UpdateButtons();};direction.SelectionChanged+=(_,_)=>level.Descending=direction.SelectedIndex==1;
   var up=new Button{Content="Up",IsEnabled=i>0};var down=new Button{Content="Down",IsEnabled=i+1<levels.Count};var remove=new Button{Content="Remove"};
   up.Click+=(_,_)=>{(levels[index-1],levels[index])=(levels[index],levels[index-1]);BuildLevels();};down.Click+=(_,_)=>{(levels[index+1],levels[index])=(levels[index],levels[index+1]);BuildLevels();};remove.Click+=(_,_)=>{levels.RemoveAt(index);BuildLevels();};
   row.Children.Add(column);row.Children.Add(direction);row.Children.Add(up);row.Children.Add(down);row.Children.Add(remove);LevelsPanel.Children.Add(row);
  }
  UpdateButtons();
 }
 private void UpdateButtons(){
  bool unique=levels.Select(l=>l.Column).Distinct().Count()==levels.Count;
  ApplyButton.IsEnabled=reviewed&&!busy&&levels.Count>0&&unique;
  ReviewButton.IsEnabled=!busy;AddButton.IsEnabled=reviewed&&!busy&&first is not null&&last is not null&&levels.Count<last.Column-first.Column+1;
  if(reviewed&&!unique)ErrorLabel.Text="Choose each key column only once.";else if(reviewed)ErrorLabel.Text="";
 }
}
public partial class MainWindow {
 private SortDialog? activeSortDialog;
 private bool sortExcludeHeader,sortMoveFormatting=true;
 private async void SortClick(object? sender,RoutedEventArgs e)=>await SortAsync();
 private async Task SortAsync(){
  if(session is null||session.IsBindingProof||session.IsProjectionProof||editing||fileAction||shuttingDown||Sheet.IsSelecting||Sheet.ProjectedSelection is not null||!await CommitAsync())return;
  var s=session;string expected=s.Revision,sheet=s.WorksheetId;var focus=Sheet.Selected.Clone();
  editing=true;SetEditing(false);++clipboardRequest;refresh?.Cancel();++displayRequest;
  try{
   activeSortDialog=new SortDialog(s,expected,Sheet.SelectionFirst,Sheet.SelectionLast);
   activeSortDialog.HeaderOption.IsChecked=sortExcludeHeader;activeSortDialog.FormatOption.IsChecked=sortMoveFormatting;
   await activeSortDialog.ReviewAsync();var choice=await activeSortDialog.ShowDialog<SortChoice?>(this);if(choice is null)return;
   if(session!=s||s.Revision!=expected||s.WorksheetId!=sheet)throw new InvalidOperationException("The workbook changed while the Sort dialog was open. Review the range again.");
   var outcome=await s.SortAsync(expected,choice.First,choice.Last,choice.Header,choice.Keys,choice.MoveFormatting);
   sortExcludeHeader=choice.Header;sortMoveFormatting=choice.MoveFormatting;
   var selected=new Position{Row=Math.Clamp(focus.Row,choice.First.Row,choice.Last.Row),Column=Math.Clamp(focus.Column,choice.First.Column,choice.Last.Column)};
   Sheet.SelectStructuralBand(choice.First,choice.Last,selected);await RefreshAsync();if(FormatPane.IsExpanded)await LoadFormatting();
   StatusLabel.Text=(outcome.SortChanged?$"Sorted {SortDialog.RangeText(choice.First,choice.Last)}. {outcome.SortMovedRows:N0} row segments and {outcome.SortMovedCells:N0} cells moved.":"No workbook change; the selected range is already in this order.")+(choice.Header?" First row excluded as header.":"");
  }catch(Exception e){StatusLabel.Text=e.Message;}
  finally{activeSortDialog=null;editing=false;SetEditing(s.DocumentId.Length!=0);clipboardEditor=null;Sheet.Focus();}
 }
}
