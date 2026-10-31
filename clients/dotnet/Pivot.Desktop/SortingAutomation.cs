using System;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
namespace Pivot.Desktop;
public partial class MainWindow {
 private async Task SortingAutomationAsync(string report){
  var s=session!;await s.NewAsync(s.Revision,discard:true);await RefreshAsync();
  Position P(uint row,uint col=1)=>new(){Row=row,Column=col};
  async Task Enter(uint row,uint col,string text){AutomationCheck(await SelectAsync(P(row,col)),"sort editor selection");BeginCellEdit(true,text);AutomationCheck(await CommitAsync(),"sort editor commit");await RefreshAsync();}
  void Click(Button button)=>button.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
  async Task<SortDialog> Open(){StructureOpened(null,new RoutedEventArgs());AutomationCheck(SortItem.IsEnabled,"sort menu enabled");SortItem.RaiseEvent(new RoutedEventArgs(MenuItem.ClickEvent));await AutomationWaitAsync(()=>activeSortDialog?.IsVisible==true);return activeSortDialog!;}
  async Task Apply(SortDialog dialog){AutomationCheck(dialog.ApplyButton.IsEnabled,"sort scope and keys reviewed");Click(dialog.ApplyButton);await AutomationWaitAsync(()=>!editing&&activeSortDialog is null&&displayedRevision==s.Revision);}
  await Enter(1,1,"'Key");await Enter(1,2,"'Then");await Enter(1,3,"'Code");
  await Enter(2,1,"1");await Enter(2,2,"2");await Enter(2,3,"'001234");
  await Enter(3,1,"1");await Enter(3,2,"1");await Enter(3,3,"'000042");
  await Enter(4,1,"0");await Enter(4,2,"3");await Enter(4,3,"'012345");await Enter(1,5,"=A2");
  await s.FormatAsync(s.Revision,P(2),P(2,3),new CellStyle(Bold:false),32);
  await s.FormatAsync(s.Revision,P(1,2),P(65536,2),new CellStyle(Italic:true),64);await RefreshAsync();
  string original=(await s.ReadAsync(P(2,3),P(2,3))).Cells.Single().CellId;
  await SelectAsync(P(1));await SelectAsync(P(4,2),true);var focus=Sheet.Selected.Clone();
  var dialog=await Open();AutomationCheck(dialog.RangeLabel.Text?.Contains("A1:B4")==true&&dialog.ExpandButton.IsVisible&&dialog.Warning.Text?.Contains("separate related records")==true,"visible partial-record warning without silent expansion");
  dialog.HeaderOption.IsChecked=true;Click(dialog.AddButton);AutomationCheck(dialog.LevelsPanel.Children.Count==2,"multiple sort keys");
  var second=(StackPanel)dialog.LevelsPanel.Children[1];Click((Button)second.Children[2]);var first=(StackPanel)dialog.LevelsPanel.Children[0];Click((Button)first.Children[3]);
  AutomationCheck(((ComboBox)((StackPanel)dialog.LevelsPanel.Children[0]).Children[0]).SelectedIndex==0,"routed key ordering controls");
  bool rangeRendered=false;dialog.RangeEditor.TextChanged+=(_,_)=>rangeRendered=true;
  Click(dialog.ExpandButton);
  try{await AutomationWaitAsync(()=>rangeRendered&&dialog.RangeLabel.Text?.Contains("A1:C4")==true&&dialog.ApplyButton.IsEnabled);}
  catch(Exception e){throw new InvalidOperationException($"Sort expansion state: text={dialog.RangeEditor.Text}; range={dialog.RangeLabel.Text}; review={dialog.ReviewButton.IsEnabled}; apply={dialog.ApplyButton.IsEnabled}; error={dialog.ErrorLabel.Text}",e);}
  Click(dialog.AddButton);AutomationCheck(dialog.LevelsPanel.Children.Count==3&&((ComboBox)((StackPanel)dialog.LevelsPanel.Children[2]).Children[0]).SelectedIndex==2,"expanded range exposes its new key column");
  Click((Button)((StackPanel)dialog.LevelsPanel.Children[2]).Children[4]);AutomationCheck(dialog.LevelsPanel.Children.Count==2,"remove expanded-range key");
  AutomationCheck(dialog.FormatOption.IsChecked==true,"record-format movement default");await Apply(dialog);
  var moved=(await s.ReadAsync(P(4,3),P(4,3))).Cells.Single();
  AutomationCheck(moved.CellId==original,$"expanded-range record identity survives sort: expected={original}; actual={moved.CellId}; text={moved.Result.Text}; status={StatusLabel.Text}");
  AutomationCheck(Sheet.Selected.Equals(focus),$"active coordinate survives sort: expected={focus}; actual={Sheet.Selected}");
  AutomationCheck(Sheet.SelectionLast.Column==3&&StatusLabel.Text?.Contains("header")==true,"confirmed rectangle and header visible after sort");
  AutomationCheck((await s.ReadAsync(P(1,5),P(1,5))).Cells.Single().Formula=="=A4","outside scalar follows sorted identity");
  AutomationCheck((await s.ReadAsync(P(4,3),P(4,3))).Cells.Single().Result.Text=="001234","leading-zero text exact");
  await s.FormatRegionAsync(s.Revision,P(1),P(1),new CellStyle(Bold:true),32,defaults:true);await RefreshAsync();
  AutomationCheck(!CellStyle.Decode((await s.ReadStylesAsync(P(4),P(4))).Styles.Single().Style).Bold&&CellStyle.Decode((await s.ReadStylesAsync(P(2),P(2))).Styles.Single().Style).Bold,"explicit false travels and inheritance remains live");
  dialog=await Open();AutomationCheck(dialog.HeaderOption.IsChecked==true,"header choice retained in window");dialog.FormatOption.IsChecked=false;
  ((ComboBox)((StackPanel)dialog.LevelsPanel.Children[0]).Children[1]).SelectedIndex=1;await Apply(dialog);
  AutomationCheck(!CellStyle.Decode((await s.ReadStylesAsync(P(4),P(4))).Styles.Single().Style).Bold,"keep-format mode retains position assignment");
  string revision=s.Revision;dialog=await Open();Click(dialog.CancelButton);await AutomationWaitAsync(()=>!editing);AutomationCheck(s.Revision==revision,"dialog cancellation does not mutate workbook");
  string path=Path.Combine(Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetDirectoryName(report)!,"sorting-ui-"+Guid.NewGuid()+".pivot");
  AutomationCheck(await SaveDocumentAsync(true,path),"sorted WB11 save");await s.NewAsync(s.Revision,discard:true);await RefreshAsync();await OpenDocumentAsync(path);
  AutomationCheck((await s.ReadAsync(P(2,3),P(2,3))).Cells.Single().Result.Text=="000042","sorted exact String survives native reopen");
  AutomationImage(Path.ChangeExtension(report,"sorting.png"));await s.NewAsync(s.Revision,discard:true);CancelDraft();clipboardEditor=null;await RefreshAsync();Sheet.Focus();
 }
}
