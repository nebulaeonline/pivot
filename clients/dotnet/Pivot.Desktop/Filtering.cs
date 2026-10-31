using System;
using System.Globalization;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Input.Platform;
using Avalonia.Interactivity;
using Avalonia.Threading;
using Pivot.Avalonia;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
namespace Pivot.Desktop;
internal sealed class FilterDialog:Window {
 internal readonly TextBox RangeEditor=new(){PlaceholderText="A1:D100"};
 internal readonly CheckBox HeaderOption=new(){Content="Exclude first row as header",IsChecked=true};
 internal readonly Button ReviewButton=new(){Content="Review range"},ExpandButton=new(){Content="Include adjacent columns",IsVisible=false},AddButton=new(){Content="Add column condition",IsEnabled=false},ApplyButton=new(){Content="Apply filter",IsEnabled=false},CancelButton=new(){Content="Cancel"};
 internal readonly TextBlock RangeLabel=new(),ErrorLabel=new(){TextWrapping=global::Avalonia.Media.TextWrapping.Wrap};
 internal readonly StackPanel ConditionsPanel=new(){Spacing=12};
 private readonly WorkbookSession session;private readonly string expected;private Position? first,last,expandedFirst,expandedLast;private bool reviewed,busy;private int epoch;
 private sealed class Rule {
  internal readonly ComboBox Type=new(){ItemsSource=new[]{"Text","Decimal","Boolean","Blank / Error"},SelectedIndex=0,Width=105};
  internal readonly ComboBox Truth=new(){ItemsSource=new[]{"false","true"},SelectedIndex=0,Width=150};
  internal readonly ComboBox Op=new(){Width=170};internal readonly TextBox Value=new(){Width=150,MaxLength=4096};
  internal Rule(FilterPredicate? value=null){
   Type.SelectionChanged+=(_,_)=>UpdateOps();UpdateOps();
   if(value is not null){Type.SelectedIndex=value.LiteralCase switch{FilterPredicate.LiteralOneofCase.Decimal=>1,FilterPredicate.LiteralOneofCase.Boolean=>2,FilterPredicate.LiteralOneofCase.Text=>0,_=>3};UpdateOps();Op.SelectedIndex=Type.SelectedIndex==3?(int)value.Operator-9:(int)value.Operator;Truth.SelectedIndex=value.Boolean?1:0;Value.Text=value.LiteralCase switch{FilterPredicate.LiteralOneofCase.Decimal=>WorkbookSession.Decode(value.Decimal).ToString(CultureInfo.InvariantCulture),FilterPredicate.LiteralOneofCase.Boolean=>value.Boolean?"true":"false",_=>value.Text};}
  }
  private void UpdateOps(){Op.ItemsSource=Type.SelectedIndex switch{0=>new[]{"Text equals","Text not equal","Text less than","Text less or equal","Text greater than","Text greater or equal","Text contains","Text starts with","Text ends with"},1=>new[]{"Decimal equals","Decimal not equal","Decimal less than","Decimal less or equal","Decimal greater than","Decimal greater or equal"},2=>new[]{"Boolean equals","Boolean not equal"},_=>new[]{"Is Blank","Is not Blank","Is Error","Is not Error"}};Op.SelectedIndex=0;Value.IsEnabled=Type.SelectedIndex!=3;Value.IsVisible=Type.SelectedIndex!=2;Truth.IsVisible=Type.SelectedIndex==2;}
  internal FilterPredicate Read(){
   if(Type.SelectedIndex<0||Op.SelectedIndex<0)throw new FormatException("Choose a type and operator.");
   var p=new FilterPredicate{Operator=(uint)(Op.SelectedIndex+(Type.SelectedIndex==3?9:0))};string text=Value.Text??"";
   switch(Type.SelectedIndex){case 0:if(System.Text.Encoding.UTF8.GetByteCount(text)>4096)throw new FormatException("Text criteria allow at most 4096 UTF-8 bytes.");p.Text=text;break;case 1:p.DecimalText=text;break;case 2:p.Boolean=Truth.SelectedIndex==1;break;}
   return p;
  }
  internal StackPanel Controls(){var row=new StackPanel{Orientation=global::Avalonia.Layout.Orientation.Horizontal,Spacing=6};foreach(var control in new Control[]{Type,Op,Value,Truth}){Detach(control);row.Children.Add(control);}return row;}
 }
 private sealed class ColumnRule {internal readonly ComboBox Column=new(){Width=110},Join=new(){ItemsSource=new[]{"One predicate","AND","OR"},SelectedIndex=0,Width=150};internal readonly Rule First,Second;internal uint Origin;internal StackPanel? SecondPanel;internal ColumnRule(FilterCondition? c=null,Action? changed=null){Column.SelectionChanged+=(_,_)=>{if(Column.SelectedIndex>=0)Column.Tag=Origin+(uint)Column.SelectedIndex;changed?.Invoke();};Join.SelectionChanged+=(_,_)=>{if(SecondPanel is not null)SecondPanel.IsVisible=Join.SelectedIndex>0;};First=new(c?.Predicates.FirstOrDefault());Second=new(c?.Predicates.Skip(1).FirstOrDefault());if(c is {Predicates.Count:2})Join.SelectedIndex=c.Either?2:1;} }
 private static void Detach(Control child){if(child.Parent is Panel panel)panel.Children.Remove(child);}
 private readonly System.Collections.Generic.List<ColumnRule> columns=[];
 internal FilterDialog(WorkbookSession session,string expected,Position initialFirst,Position initialLast,uint? focusColumn=null,FilterDefinition? existing=null){
  this.session=session;this.expected=expected;Title="Filter worksheet records";Width=830;Height=700;MinWidth=790;MinHeight=580;WindowStartupLocation=WindowStartupLocation.CenterOwner;
  var panel=new StackPanel{Margin=new Thickness(18),Spacing=12};
  panel.Children.Add(new TextBlock{Text="Filter range"});var range=new Grid{ColumnDefinitions=new("*,Auto"),ColumnSpacing=8};range.Children.Add(RangeEditor);Grid.SetColumn(ReviewButton,1);range.Children.Add(ReviewButton);panel.Children.Add(range);panel.Children.Add(RangeLabel);panel.Children.Add(ExpandButton);panel.Children.Add(HeaderOption);
  panel.Children.Add(new TextBlock{Text="Filtering hides entire worksheet rows, including cells outside these columns. Formulas still include hidden rows. Every column condition below must match.",TextWrapping=global::Avalonia.Media.TextWrapping.Wrap});
  panel.Children.Add(new ScrollViewer{Content=ConditionsPanel,MaxHeight=350});panel.Children.Add(AddButton);
  panel.Children.Add(new TextBlock{Text="Text is exact and case-sensitive; leading zeros stay intact. Empty String differs from Blank. Typed negatives match only their own type. Blank/Error predicates do not need a literal.",TextWrapping=global::Avalonia.Media.TextWrapping.Wrap});panel.Children.Add(ErrorLabel);
  var buttons=new StackPanel{Orientation=global::Avalonia.Layout.Orientation.Horizontal,Spacing=12};buttons.Children.Add(ApplyButton);buttons.Children.Add(CancelButton);panel.Children.Add(buttons);Content=new ScrollViewer{Content=panel};
  RangeEditor.TextChanging+=(_,_)=>{reviewed=false;epoch++;UpdateButtons();};
  HeaderOption.IsChecked=existing?.ExcludeHeader??true;
  if(existing is not null){initialFirst=existing.First;initialLast=existing.Last;foreach(var c in existing.Conditions){var row=new ColumnRule(c,UpdateButtons);row.Column.Tag=c.Column;columns.Add(row);}}
  if(focusColumn is uint key&&key>=initialFirst.Column&&key<=initialLast.Column&&!columns.Any(c=>c.Column.Tag is uint column&&column==key)){var rule=new ColumnRule(null,UpdateButtons);rule.Column.Tag=key;columns.Add(rule);}
  RangeEditor.Text=initialFirst.Equals(initialLast)?WorksheetView.Address(initialFirst):SortDialog.RangeText(initialFirst,initialLast);
  if(existing?.NeedsReview==true){this.first=initialFirst.Clone();this.last=initialLast.Clone();panel.Children.Insert(0,new TextBlock{Text="Rows or columns changed. These are the former range and column letters. Review the range and every key, then click Review range before Apply.",TextWrapping=global::Avalonia.Media.TextWrapping.Wrap});BuildColumns();}
  Opened+=(_,_)=>{if(focusColumn is uint key){int index=columns.FindIndex(c=>c.Column.Tag is uint value&&value==key);if(index>=0&&index<ConditionsPanel.Children.Count){ConditionsPanel.Children[index].BringIntoView();columns[index].First.Type.Focus();}}};
  ReviewButton.Click+=async(_,_)=>await ReviewAsync();ExpandButton.Click+=async(_,_)=>{if(expandedFirst is null||expandedLast is null)return;RangeEditor.Text=SortDialog.RangeText(expandedFirst,expandedLast);await ReviewAsync();};
  AddButton.Click+=(_,_)=>{if(first is null||last is null)return;for(uint c=first.Column;c<=last.Column;c++)if(!columns.Any(r=>r.Column.Tag is uint used&&used==c)){var row=new ColumnRule(null,UpdateButtons);row.Column.Tag=c;columns.Add(row);BuildColumns();break;}};
  ApplyButton.Click+=async(_,_)=>{int request=epoch;try{if(!ApplyButton.IsEnabled||first is null||last is null)return;var definition=new FilterDefinition{First=first.Clone(),Last=last.Clone(),ExcludeHeader=HeaderOption.IsChecked==true};foreach(var row in columns){var c=new FilterCondition{Column=(uint)row.Column.Tag!,Either=row.Join.SelectedIndex==2};c.Predicates.Add(row.First.Read());if(row.Join.SelectedIndex>0)c.Predicates.Add(row.Second.Read());definition.Conditions.Add(c);}busy=true;UpdateButtons();await session.ValidateFilterAsync(expected,definition);if(request==epoch)Close(definition);}catch(Exception e){ErrorLabel.Text=e.Message;}finally{busy=false;UpdateButtons();}};
  CancelButton.Click+=(_,_)=>Close();
 }
 internal async Task ReviewAsync(){int current=++epoch;busy=true;reviewed=false;UpdateButtons();try{var parts=(RangeEditor.Text??"").Split(':');if(parts.Length!=2)throw new FormatException("Enter an explicit range such as A1:D100.");var a=WorksheetView.ParseAddress(parts[0]);var b=WorksheetView.ParseAddress(parts[1]);if(a.Row>b.Row||a.Column>b.Column)throw new FormatException("Use top-left to bottom-right range order.");var bounds=await session.SortBoundsAsync(expected,a,b);if(current!=epoch)return;first=a;last=b;expandedFirst=bounds.SortSuggestedFirst;expandedLast=bounds.SortSuggestedLast;ExpandButton.IsVisible=!a.Equals(expandedFirst)||!b.Equals(expandedLast);ExpandButton.Content="Include adjacent columns: "+SortDialog.RangeText(expandedFirst,expandedLast);RangeLabel.Text="Final filter range: "+SortDialog.RangeText(a,b);columns.RemoveAll(r=>r.Column.Tag is not uint c||c<a.Column||c>b.Column);reviewed=true;ErrorLabel.Text="";BuildColumns();}catch(Exception e){if(current==epoch)ErrorLabel.Text=e.Message;}finally{busy=false;UpdateButtons();}}
 private void BuildColumns(){ConditionsPanel.Children.Clear();if(first is null||last is null)return;uint origin=first.Column;var names=Enumerable.Range((int)origin,(int)(last.Column-origin+1)).Select(c=>WorksheetView.Address(new(){Row=1,Column=(uint)c}).TrimEnd('1')).ToArray();foreach(var rule in columns){var box=new StackPanel{Spacing=6};uint selected=(uint)rule.Column.Tag!;Detach(rule.Column);Detach(rule.Join);rule.Origin=origin;rule.Column.ItemsSource=names;rule.Column.SelectedIndex=(int)(selected-origin);var header=new StackPanel{Orientation=global::Avalonia.Layout.Orientation.Horizontal,Spacing=8};header.Children.Add(new TextBlock{Text="Column"});header.Children.Add(rule.Column);header.Children.Add(rule.Join);var remove=new Button{Content="Remove"};remove.Click+=(_,_)=>{columns.Remove(rule);BuildColumns();};header.Children.Add(remove);box.Children.Add(header);box.Children.Add(rule.First.Controls());var second=rule.Second.Controls();second.IsVisible=rule.Join.SelectedIndex>0;rule.SecondPanel=second;box.Children.Add(second);ConditionsPanel.Children.Add(box);}UpdateButtons();}
 private void UpdateButtons(){RangeEditor.IsEnabled=!busy;HeaderOption.IsEnabled=!busy;ConditionsPanel.IsEnabled=!busy;ExpandButton.IsEnabled=!busy;bool unique=columns.Select(c=>c.Column.Tag).Distinct().Count()==columns.Count;ApplyButton.IsEnabled=reviewed&&!busy&&unique;ReviewButton.IsEnabled=!busy;AddButton.IsEnabled=reviewed&&!busy&&first is not null&&last is not null&&columns.Count<last.Column-first.Column+1;}
}
public partial class MainWindow {
 private bool filterPublicationDeferred,filterViewPending;
 private string displayedFilterGeneration="";
 private bool FilterBulkReady=>!filterViewPending&&session?.FilterStatus is not (3 or 4);
 private FilterDialog? activeFilterDialog;private Window? activeHiddenRowsDialog;
 private void ConfigureFilterSession(WorkbookSession owner){
  owner.ConfirmHiddenRowsAsync=ConfirmHiddenRowsAsync;
  owner.ConfirmVisiblePasteAsync=ConfirmVisiblePasteAsync;
  owner.CaptureFilterInteractionAsync=cancel=>OnUiAsync(()=>{cancel.ThrowIfCancellationRequested();return Task.FromResult($"{owner.DocumentGeneration}:{owner.WorksheetId}:{interactionVersion}:{displayedFilterGeneration}:{Sheet.SelectionFirst}:{Sheet.SelectionLast}:{Sheet.Selected}");});
  owner.RevalidateFilteredOperationAsync=(operation,cancel)=>OnUiAsync(async()=>{
   cancel.ThrowIfCancellationRequested();if(operation.Operation is not (Request.Types.Operation.PasteBlock or Request.Types.Operation.PasteVisible))return true;
   if(Clipboard is null)return false;
   var current=await ClipboardRead.CaptureAsync<IAsyncDataTransfer>(Clipboard.TryGetInProcessDataAsync,Clipboard.TryGetDataAsync,async(data,owned)=>{
    return await ReadWorksheetClipboardAsync(data,true);
   });
   cancel.ThrowIfCancellationRequested();return operation.NativeClipboard?current.Native&&operation.ClipboardKind==current.NativeKind&&current.Bytes is not null&&operation.Clipboard.Span.SequenceEqual(current.Bytes):!current.Native&&!current.Obsolete&&current.Text is not null&&operation.Clipboard.Span.SequenceEqual(System.Text.Encoding.UTF8.GetBytes(current.Text));
  });
 }
 private async void FilterClick(object? sender,RoutedEventArgs e)=>await ConfigureFilterAsync();
 private async Task ConfigureFilterAsync(uint? column=null){
  if(session is null||editing||fileAction||shuttingDown||session.IsBindingProof||session.IsProjectionProof||!await CommitAsync())return;
  var s=session;string expected=s.Revision,sheet=s.WorksheetId;editing=true;SetEditing(false);++clipboardRequest;
  try{var stored=(await s.GetFilterAsync()).FilterDefinition;activeFilterDialog=new FilterDialog(s,expected,Sheet.SelectionFirst,Sheet.SelectionLast,column,stored);if(stored?.NeedsReview!=true)await activeFilterDialog.ReviewAsync();var definition=await activeFilterDialog.ShowDialog<FilterDefinition?>(this);if(definition is null)return;if(session!=s||s.WorksheetId!=sheet||s.Revision!=expected)throw new InvalidOperationException("The worksheet changed; review the filter again.");await s.SetFilterAsync(expected,definition);await RefreshAsync();UpdateDocument();}catch(Exception e){StatusLabel.Text=e.Message;}finally{activeFilterDialog=null;editing=false;SetEditing(s.DocumentId.Length!=0);Sheet.Focus();}
 }
 private async Task FilterCommandAsync(Func<WorkbookSession,Task> action){
  if(session is null||editing||fileAction||shuttingDown||!await CommitAsync())return;var owner=session;editing=true;SetEditing(false);++clipboardRequest;
  try{await action(owner);await RefreshAsync();UpdateDocument();}catch(OperationCanceledException){}catch(Exception error){StatusLabel.Text=error.Message;}finally{editing=false;SetEditing(owner.DocumentId.Length!=0);Sheet.Focus();}
 }
 private async void ReapplyFilterClick(object? sender,RoutedEventArgs e)=>await FilterCommandAsync(owner=>owner.ReapplyFilterAsync());
 private async void ClearFilterClick(object? sender,RoutedEventArgs e)=>await FilterCommandAsync(owner=>owner.SetFilterAsync(owner.Revision,null));
 private static Task<T> OnUiAsync<T>(Func<Task<T>> action){if(Dispatcher.UIThread.CheckAccess())return action();var completion=new TaskCompletionSource<T>(TaskCreationOptions.RunContinuationsAsynchronously);Dispatcher.UIThread.Post(async()=>{try{completion.SetResult(await action());}catch(Exception e){completion.SetException(e);}});return completion.Task;}
 private Task<bool> ConfirmHiddenRowsAsync(Request operation,Response preview,CancellationToken cancellation)=>OnUiAsync(async()=>{
  cancellation.ThrowIfCancellationRequested();var panel=new StackPanel{Margin=new Thickness(20),Spacing=12};var window=new Window{Title="Operation includes filtered-out rows",Width=600,SizeToContent=SizeToContent.Height,WindowStartupLocation=WindowStartupLocation.CenterOwner};activeHiddenRowsDialog=window;
  string ranges=string.Join(", ",preview.FilterFootprints.Select(f=>SortDialog.RangeText(f.First,f.Last)));
  panel.Children.Add(new TextBlock{Text=$"{operation.Operation}: {ranges}\nThis operation includes {preview.FilterHiddenRows:N0} filtered-out worksheet rows. All addressed rows participate; this is not a visible-only operation.",TextWrapping=global::Avalonia.Media.TextWrapping.Wrap});var buttons=new StackPanel{Orientation=global::Avalonia.Layout.Orientation.Horizontal,Spacing=12};var include=new Button{Content=$"Include {preview.FilterHiddenRows:N0} filtered-out rows"};var cancel=new Button{Content="Cancel",IsCancel=true};include.Click+=(_,_)=>window.Close(true);cancel.Click+=(_,_)=>window.Close(false);buttons.Children.Add(cancel);buttons.Children.Add(include);panel.Children.Add(buttons);window.Content=panel;
  try{var choice=window.ShowDialog<bool>(this);using var registration=cancellation.Register(()=>Dispatcher.UIThread.Post(()=>{if(window.IsVisible)window.Close(false);}));return await choice;}finally{activeHiddenRowsDialog=null;}
 });
 private async Task RefreshPendingFilterAsync(WorkbookSession owner,string worksheet,ulong document,long request){
  await Task.Delay(60);if(session!=owner||owner.WorksheetId!=worksheet||owner.DocumentGeneration!=document||request!=displayRequest||shuttingDown)return;await RefreshAsync();
 }
 private void ShowFilterStatus(Response frame){
  filterViewPending=frame.FilterStatus is 3 or 4;filterPublicationDeferred=false;
  FilterBar.IsVisible=frame.FilterSummary is not null;
  FilterStatusLabel.Text=frame.FilterStatus switch{1 when frame.FilterSummary is { } definition=>$"Filter {SortDialog.RangeText(definition.First,definition.Last)}: {frame.FilterShownDataRows:N0} of {frame.FilterTotalDataRows:N0} data rows shown",2=>"Filter needs range/key review after a structural edit; all rows shown.",3=>"Refreshing filter; all rows shown. Bulk actions unavailable.",4=>"Filter suspended; all rows shown. "+frame.FilterError+" Reapply or Clear Filter.",_=>""};
  SetEditing(session is not null&&session.DocumentId.Length!=0&&!editing);
  ReapplyFilterButton.IsEnabled=frame.FilterStatus is 1 or 4;ClearFilterButton.IsEnabled=frame.FilterSummary is not null;
 }
}
