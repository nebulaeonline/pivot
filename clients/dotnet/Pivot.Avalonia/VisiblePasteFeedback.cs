using Avalonia.Threading;
namespace Pivot.Avalonia;
public sealed partial class WorksheetView {
 private readonly HashSet<(uint Row,uint Column)> visiblePasteFeedback=[];
 private long visiblePasteFeedbackEpoch;
 public void HighlightPastedRows(IEnumerable<uint> source,uint firstColumn,uint width){
  var rows=source.ToArray();
  if(rows.Length==0||width==0||(ulong)rows.Length*width>256||firstColumn==0||firstColumn+width-1>1024||rows.Any(r=>r==0||r>65536))throw new ArgumentOutOfRangeException(nameof(source));
  ClearPasteFeedback();foreach(uint row in rows)for(uint column=firstColumn;column<firstColumn+width;column++)visiblePasteFeedback.Add((row,column));
  long epoch=visiblePasteFeedbackEpoch;InvalidateVisual();_ = ExpirePasteFeedbackAsync(epoch);
 }
 private async Task ExpirePasteFeedbackAsync(long epoch){await Task.Delay(1500);Dispatcher.UIThread.Post(()=>{if(epoch==visiblePasteFeedbackEpoch)ClearPasteFeedback();});}
 private void ClearPasteFeedback(){++visiblePasteFeedbackEpoch;visiblePasteFeedback.Clear();InvalidateVisual();}
}
