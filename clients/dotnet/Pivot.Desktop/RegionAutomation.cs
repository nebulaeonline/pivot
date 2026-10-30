using System;
using System.IO;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Input;
using Pivot.Client;
namespace Pivot.Desktop;
public partial class MainWindow {
 private async Task RegionAutomationAsync(string report) {
  var s=session!;await s.NewAsync(s.Revision,discard:true);await RefreshAsync();
  for(uint i=0;i<100;i++)await s.FormatAsync(s.Revision,AutomationPosition(1+i*10,3),AutomationPosition(1+i*10,3),new CellStyle(Numeric:1,Bold:true),33);
  await SelectAsync(AutomationPosition(1,1));await SelectAsync(AutomationPosition(1000,10),true);
  FormatPane.IsExpanded=true;UpdateLayout();await RefreshAsync();await LoadFormatting();
  AutomationCheck(formatCombos[0].SelectedIndex==0&&formatCombos[4].SelectedIndex==0,"large selection Mixed includes one hundred exceptions");
  formatCombos[0].SelectedIndex=3;await ApplyFormatting(false);
  var summary=await s.ReadStyleSummaryAsync(AutomationPosition(1,1),AutomationPosition(1000,10));
  AutomationCheck((summary.MixedStyleFields&1)==0&&CellStyle.Decode(summary.SummaryStyle).Numeric==2,"pane Currency replaces all selected exceptions");
  AutomationCheck((summary.MixedStyleFields&32)!=0,"pane preserves unrelated bold");
  formatCombos[4].SelectedIndex=1;await ApplyFormatting(false);
  summary=await s.ReadStyleSummaryAsync(AutomationPosition(1,1),AutomationPosition(1000,10));
  AutomationCheck((summary.MixedStyleFields&32)==0&&!CellStyle.Decode(summary.SummaryStyle).Bold,"pane clears prior individual bold across selection");
  await SelectAsync(AutomationPosition(1,1));await RefreshAsync();
  var mouse=new Pointer(998,PointerType.Mouse,true);
  var held=new PointerPointProperties(global::Avalonia.Input.RawInputModifiers.LeftMouseButton,PointerUpdateKind.LeftButtonPressed);
  async Task Header(Point point,Func<bool> done) {
   Sheet.RaiseEvent(new PointerPressedEventArgs(Sheet,mouse,this,Sheet.TranslatePoint(point,this)!.Value,0,held,KeyModifiers.None,1));
   await AutomationWaitAsync(done);await LoadFormatting();
  }
  var bounds=Sheet.CellBounds(AutomationPosition(1,2));
  await Header(new Point(bounds.Center.X,10),()=>Sheet.SelectionFirst.Column==2&&Sheet.SelectionLast.Row==65536&&Sheet.SelectionLast.Column==2);
  formatCombos[8].SelectedIndex=6;await ApplyFormatting(false);
  AutomationCheck(CellStyle.Decode((await s.ReadStylesAsync(AutomationPosition(65536,2),AutomationPosition(65536,2))).Styles[0].Style).Background==0xc00000,"column header formats bottom absent position");
  bounds=Sheet.CellBounds(AutomationPosition(1,1));
  await Header(new Point(10,bounds.Center.Y),()=>Sheet.SelectionFirst.Row==1&&Sheet.SelectionLast.Column==1024&&Sheet.SelectionLast.Row==1);
  formatCombos[8].SelectedIndex=5;await ApplyFormatting(false);
  AutomationCheck(CellStyle.Decode((await s.ReadStylesAsync(AutomationPosition(1,2),AutomationPosition(1,2))).Styles[0].Style).Background==0x008000,"later row formatting wins at column intersection");
  formatTarget.SelectedIndex=1;await LoadFormatting();formatCombos[5].SelectedIndex=2;await ApplyFormatting(false);
  formatTarget.SelectedIndex=0;await LoadFormatting();await ApplyFormatting(false,true);
  AutomationCheck(CellStyle.Decode((await s.ReadStylesAsync(AutomationPosition(1,1024),AutomationPosition(1,1024))).Styles[0].Style).Italic,"Use worksheet defaults distinct from built-in reset");
  await ApplyFormatting(true);
  AutomationCheck(!CellStyle.Decode((await s.ReadStylesAsync(AutomationPosition(1,1024),AutomationPosition(1,1024))).Styles[0].Style).Italic,"Reset writes explicit built-in default");
  string path=Path.Combine(Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT")??Path.GetDirectoryName(report)!,"regions-ui-"+Guid.NewGuid()+".pivot");
  AutomationCheck(await SaveDocumentAsync(true,path),"region workbook saves");await s.NewAsync(s.Revision,discard:true);await RefreshAsync();await OpenDocumentAsync(path);
  AutomationCheck(CellStyle.Decode((await s.ReadStylesAsync(AutomationPosition(65536,2),AutomationPosition(65536,2))).Styles[0].Style).Background==0xc00000,"WB10 column region reopens");
  AutomationImage(Path.ChangeExtension(report,"regions.png"));
  formatTarget.SelectedIndex=0;FormatPane.IsExpanded=false;await s.NewAsync(s.Revision,discard:true);await RefreshAsync();Sheet.Focus();
 }
}
