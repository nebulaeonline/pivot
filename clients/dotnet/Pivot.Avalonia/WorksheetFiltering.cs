using Avalonia;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
namespace Pivot.Avalonia;
public sealed partial class WorksheetView {
 private uint[]? filterRows;
 private string filterGeneration="";
 public FilterSummary? FilterSummary {get;private set;}
 public bool HasVisibleRows=>filterRows is null||filterRows.Length!=0;
 public bool HasFilterMap=>filterRows is not null;
 public event Action<uint>? FilterColumnRequested;
 public uint? WorksheetRowAt(int offset){
  if(filterRows is null)return (uint)Math.Clamp((int)FirstRow+offset,1,WorkbookCapacity.MaximumRows);
  long index=(long)FirstRow-1+offset;return index>=0&&index<filterRows.Length?filterRows[index]:null;
 }
 public bool RowIsShown(uint row)=>filterRows is null||Array.BinarySearch(filterRows,row)>=0;
 public uint ClosestVisibleRow(uint row,bool after=true){
  if(filterRows is null)return row;if(filterRows.Length==0)return 1;
  int index=Array.BinarySearch(filterRows,row);if(index>=0)return filterRows[index];index=~index;
  if(!after)index--;return filterRows[Math.Clamp(index,0,filterRows.Length-1)];
 }
 public uint StepVisibleRow(uint row,int delta){
  if(filterRows is null)return (uint)Math.Clamp((long)row+delta,1,WorkbookCapacity.MaximumRows);
  if(filterRows.Length==0)return 1;int index=Array.BinarySearch(filterRows,row);if(index<0)index=~index;
  return filterRows[Math.Clamp((long)index+delta,0,filterRows.Length-1)];
 }
 public uint HiddenRows(uint first,uint last){
  if(filterRows is null)return 0;
  int lo=Array.BinarySearch(filterRows,first);if(lo<0)lo=~lo;
  int hi=Array.BinarySearch(filterRows,last);hi=hi<0?~hi:hi+1;
  return last-first+1-(uint)(hi-lo);
 }
 private int FilterIndex(uint row){if(filterRows is null)return (int)row-(int)FirstRow;int index=Array.BinarySearch(filterRows,row);return index<0?-1:index-(int)FirstRow+1;}
 private uint FilterVisualRow(uint row){if(filterRows is null)return row;int index=Array.BinarySearch(filterRows,row);return (uint)((index<0?~index:index)+1);}
 public void ClearFilterFrame(){filterRows=null;FilterSummary=null;filterGeneration="";}
 public void SetFilterFrame(Response response){
  if(response.FilterVisibleMask.Length!=8192)throw new InvalidDataException("Invalid filter row mask.");
  if(response.FilterStatus>4||(response.FilterStatus==0)!=(response.FilterSummary is null))throw new InvalidDataException("Invalid filter presentation state.");
  if((response.FilterStatus==2)!=(response.FilterSummary?.NeedsReview==true)||response.FilterShownDataRows>response.FilterTotalDataRows)throw new InvalidDataException("Invalid filter counts or review state.");
  var frameRows=response.FilterRows.ToHashSet();if(response.Cells.Any(c=>c.At is null||!frameRows.Contains(c.At.Row))||response.Styles.Any(s=>s.At is null||!frameRows.Contains(s.At.Row)))throw new InvalidDataException("Filtered cell/style addresses do not match the mapped rows.");
  if(response.FilterStatus!=1&&response.FilterVisibleMask.Span.ContainsAnyExcept((byte)255))throw new InvalidDataException("Pending or suspended filtering must show all rows.");
  if(filterGeneration!=response.FilterGeneration){EndPointerSelection(false);filterGeneration=response.FilterGeneration;}
  if(response.FilterStatus!=1){
   if(response.FilterTotalVisibleRows!=WorkbookCapacity.MaximumRows||response.FilterFirstVisible==0||response.FilterFirstVisible>WorkbookCapacity.MaximumRows||!response.FilterRows.SequenceEqual(Enumerable.Range((int)response.FilterFirstVisible,response.FilterRows.Count).Select(r=>(uint)r)))throw new InvalidDataException("Invalid unfiltered viewport mapping.");
   filterRows=null;FilterSummary=response.FilterSummary?.Clone();FirstRow=response.FilterFirstVisible;InvalidateVisual();return;
  }
  var rows=new List<uint>();for(uint row=1;row<=WorkbookCapacity.MaximumRows;row++)if((response.FilterVisibleMask.Span[(int)(row-1)/8]&(1<<((int)(row-1)%8)))!=0)rows.Add(row);
  if(rows.Count!=response.FilterTotalVisibleRows||response.FilterFirstVisible==0)throw new InvalidDataException("Invalid filtered viewport mapping.");
  var expected=rows.Skip((int)response.FilterFirstVisible-1).Take(response.FilterRows.Count);if(!expected.SequenceEqual(response.FilterRows))throw new InvalidDataException("Incoherent filtered viewport rows.");
  FilterSummary=response.FilterSummary?.Clone();filterRows=FilterSummary is null||FilterSummary.NeedsReview?null:rows.ToArray();FirstRow=response.FilterFirstVisible;
  if(HasVisibleRows&&!RowIsShown(Selected.Row))Select(new Position{Row=ClosestVisibleRow(Selected.Row),Column=Selected.Column});
  InvalidateVisual();
 }
 private bool FilterHeaderClick(Point point){
  if(FilterSummary is not {ExcludeHeader:true,NeedsReview:false,First:{ } first}||point.X<Header||point.Y<HeightPerCell)return false;
  int offset=(int)Math.Floor(point.Y/HeightPerCell)-1;uint? row=WorksheetRowAt(offset);
  uint column=FirstColumn+(uint)((point.X-Header)/WidthPerCell);
  if(row!=first.Row||column<first.Column||column>FilterSummary.Last.Column||(point.X-Header)%WidthPerCell<WidthPerCell-18)return false;
  FilterColumnRequested?.Invoke(column);return true;
 }
}
