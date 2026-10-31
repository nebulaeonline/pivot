using System;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Input.Platform;
using Avalonia.Interactivity;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
namespace Pivot.Desktop;
public partial class MainWindow {
 private async Task FilteringAutomationAsync(string report){
  var s=session!;await s.NewAsync(s.Revision,discard:true);await RefreshAsync();
  Position P(uint row,uint col=1)=>new(){Row=row,Column=col};void Click(Button button)=>button.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
  async Task Enter(uint row,uint col,string text){AutomationCheck(await SelectAsync(P(row,col)),"filter entry selection");BeginCellEdit(true,text);AutomationCheck(await CommitAsync(),"filter entry commit");await RefreshAsync();}
  async Task Ready()=>await AutomationWaitAsync(()=>s.FilterStatus!=3&&displayedRevision==s.Revision&&frameReady);
  async Task<Window> Confirmation(){await AutomationWaitAsync(()=>activeHiddenRowsDialog?.IsVisible==true);return activeHiddenRowsDialog!;}
  Button Choice(Window window,bool include)=>(Button)((StackPanel)((StackPanel)window.Content!).Children[1]).Children[include?1:0];
  await Enter(1,1,"'Code");await Enter(1,2,"'Value");await Enter(2,1,"'001234");await Enter(3,1,"1234");await Enter(4,1,"'other");await Enter(5,1,"'001234");await Enter(1,3,"=SUM(A3:A3)");
  string original=(await s.ReadAsync(P(2),P(2))).Cells.Single().CellId;
  await SelectAsync(P(1));await SelectAsync(P(5,2),true);StructureOpened(null,new RoutedEventArgs());FilterItem.RaiseEvent(new RoutedEventArgs(MenuItem.ClickEvent));await AutomationWaitAsync(()=>activeFilterDialog?.IsVisible==true);var dialog=activeFilterDialog!;
  AutomationCheck(dialog.HeaderOption.IsChecked==true&&dialog.ExpandButton.IsVisible,"explicit header and adjacent expansion choice");Click(dialog.ExpandButton);await AutomationWaitAsync(()=>dialog.RangeLabel.Text?.Contains("A1:C5")==true&&dialog.AddButton.IsEnabled);
  Click(dialog.AddButton);var condition=(StackPanel)dialog.ConditionsPanel.Children[0];((TextBox)((StackPanel)condition.Children[1]).Children[2]).Text="001234";
  Click(dialog.AddButton);var second=(StackPanel)dialog.ConditionsPanel.Children[1];var column=(ComboBox)((StackPanel)second.Children[0]).Children[1];column.SelectedIndex=0;AutomationCheck(!dialog.ApplyButton.IsEnabled,"duplicate condition columns refused");column.SelectedIndex=1;Click((Button)((StackPanel)second.Children[0]).Children[3]);
  AutomationCheck(((TextBox)((StackPanel)((StackPanel)dialog.ConditionsPanel.Children[0]).Children[1]).Children[2]).Text=="001234","exact text survives condition rebuild");Click(dialog.ApplyButton);await AutomationWaitAsync(()=>activeFilterDialog is null&&!editing&&s.FilterStatus==1&&Sheet.FilterSummary is not null);await Ready();
  AutomationCheck(Sheet.FilterSummary!.Last.Column==3&&Sheet.WorksheetRowAt(0)==1&&Sheet.WorksheetRowAt(1)==2&&Sheet.WorksheetRowAt(2)==5,"confirmed geometry and original worksheet row numbers");
  AutomationCheck(FilterStatusLabel.Text?.Contains("2 of 4")==true&&FilterBar.IsVisible,"active filter counts remain visible");
  await SelectAsync(P(2));await NavigateAsync(1,0);AutomationCheck(Sheet.Selected.Row==5,"keyboard navigation skips hidden rows");AutomationCheck(Sheet.CellBounds(P(5)).Y-Sheet.CellBounds(P(2)).Y==26,"editor geometry maps visible neighbors");
  AutomationCheck(WorkbookSession.Decode((await s.ReadAsync(P(1,3),P(1,3))).Cells.Single().Result.Decimal)==1234,"hidden values remain aggregate members");
  await SelectAsync(P(2));await SelectAsync(P(5),true);string revision=s.Revision;var copy=ClipboardAsync(false);var confirm=await Confirmation();Click(Choice(confirm,false));await copy;AutomationCheck(s.Revision==revision,"declined hidden-row copy leaves workbook intact");
  // TSV marks Strings with an apostrophe, preserving type and leading zeros.
  copy=ClipboardAsync(false);confirm=await Confirmation();AutomationCheck(((TextBlock)((StackPanel)confirm.Content!).Children[0]).Text?.Contains("2 filtered-out")==true,"exact hidden-row count in operation dialog");Click(Choice(confirm,true));await copy;await AutomationExpectClipboardTextAsync(Clipboard!,"'001234\n1234\n'other\n'001234","confirmed copy includes hidden rows and exact Strings");
  await SelectAsync(P(2));revision=s.Revision;var paste=ClipboardAsync(true);confirm=await Confirmation();await Clipboard!.SetTextAsync("replacement after confirmation");Click(Choice(confirm,true));await paste;AutomationCheck(s.Revision==revision&&draftCell is null,"changed clipboard cancels confirmed paste without mutation or draft fallback");
  await SelectAsync(P(2));MoveTarget.Text="B3";MoveButton.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));confirm=await Confirmation();Click(Choice(confirm,true));await AutomationWaitAsync(()=>!editing&&displayedRevision==s.Revision&&s.FilterStatus==1);await Ready();
  AutomationCheck((await s.ReadAsync(P(3,2),P(3,2))).Cells.Single().CellId==original&&!Sheet.RowIsShown(2),"Move keeps identity and automatic refresh hides changed keys");
  var pointer=new Pointer(993,PointerType.Mouse,true);var held=new PointerPointProperties(global::Avalonia.Input.RawInputModifiers.LeftMouseButton,PointerUpdateKind.LeftButtonPressed);var released=new PointerPointProperties(global::Avalonia.Input.RawInputModifiers.None,PointerUpdateKind.LeftButtonReleased);var bounds=Sheet.CellBounds(P(1));var point=new Point(bounds.Right-8,bounds.Center.Y);var local=Sheet.TranslatePoint(point,this)!.Value;
  Sheet.RaiseEvent(new PointerPressedEventArgs(Sheet,pointer,this,local,0,held,KeyModifiers.None,1));await AutomationWaitAsync(()=>activeFilterDialog?.IsVisible==true);Sheet.RaiseEvent(new PointerReleasedEventArgs(Sheet,pointer,this,local,0,released,KeyModifiers.None,MouseButton.Left));Click(activeFilterDialog!.CancelButton);await AutomationWaitAsync(()=>!editing&&activeFilterDialog is null);
  await SelectAsync(P(1));StructureOpened(null,new RoutedEventArgs());InsertRowsItem.RaiseEvent(new RoutedEventArgs(MenuItem.ClickEvent));confirm=await Confirmation();Click(Choice(confirm,true));await AutomationWaitAsync(()=>!editing&&s.FilterStatus==2&&Sheet.FilterSummary?.NeedsReview==true);AutomationCheck(FilterStatusLabel.Text?.Contains("review")==true&&!ReapplyFilterButton.IsEnabled,"structural review is explicit and Reapply does not rebind");
  FilterItem.RaiseEvent(new RoutedEventArgs(MenuItem.ClickEvent));await AutomationWaitAsync(()=>activeFilterDialog?.IsVisible==true);AutomationCheck(!activeFilterDialog!.ApplyButton.IsEnabled&&activeFilterDialog.ConditionsPanel.Children.Count==1,"structural draft requires explicit range/key review");Click(activeFilterDialog.ReviewButton);await AutomationWaitAsync(()=>activeFilterDialog?.ApplyButton.IsEnabled==true);Click(activeFilterDialog!.CancelButton);await AutomationWaitAsync(()=>!editing&&activeFilterDialog is null);AutomationCheck(s.FilterSummary!.NeedsReview,"review cancellation preserves structural suspension");
  string path=Path.Combine(Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetDirectoryName(report)!,"filter-ui-"+Guid.NewGuid()+".pivot");AutomationCheck(await SaveDocumentAsync(true,path),"filter metadata WB12 save");await s.NewAsync(s.Revision,discard:true);await RefreshAsync();await OpenDocumentAsync(path);AutomationCheck(s.FilterStatus==2&&s.FilterSummary!.NeedsReview&&!s.IsDirty,"WB12 preserves definition and structural review");
  Click(ClearFilterButton);await AutomationWaitAsync(()=>!editing&&s.FilterSummary is null&&!Sheet.HasFilterMap);await RefreshAsync();
  var none=new FilterDefinition{First=P(1),Last=P(65536),Conditions={new FilterCondition{Column=1,Predicates={new FilterPredicate{Operator=0,Text="no match"}}}}};await s.SetFilterAsync(s.Revision,none);await RefreshAsync();AutomationCheck(!Sheet.HasVisibleRows&&!Sheet.IsEnabled&&ClearFilterButton.IsEnabled,"no visible rows leaves filter recovery available");
  Click(ClearFilterButton);await AutomationWaitAsync(()=>!editing&&s.FilterSummary is null&&Sheet.HasVisibleRows&&Sheet.IsEnabled);AutomationImage(Path.ChangeExtension(report,"filtering.png"));await s.NewAsync(s.Revision,discard:true);CancelDraft();await RefreshAsync();
 }
}
