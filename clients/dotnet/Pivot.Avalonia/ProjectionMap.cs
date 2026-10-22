using System.Globalization;
using Avalonia;
using Avalonia.Media;
using Pivot.Protocol.Application.V1;
namespace Pivot.Avalonia;
public sealed record ProjectionTarget(string Projection,string Producer,string Generation,string Key,string Field,ulong Epoch,ulong Presentation);
public sealed partial class WorksheetView
{
    public ProofFrame? ProjectionFrame {get;private set;}
    public ProjectionTarget? ProjectedSelection {get;private set;}
    public bool MissingProjectedRecord {get;private set;}
    public event Action? ProjectedSelectionChanged;
    public Func<ProjectionTarget,Task>? ProjectionRequested {get;set;}
    public uint TotalVisualRows=>1024+(uint)(ProjectionFrame?.Placements.Count??0)*(Math.Max(1,ProjectionFrame?.RecordCount??0)+2);
    private ProofRow? MappedRow(int row)=>ProjectionFrame is { } f && f.FirstVisualRow==FirstRow && row>=0 && row<f.Rows.Count ? f.Rows[row] : null;
    private int OrdinaryIndex(uint row)=>ProjectionFrame is null ? (int)row-(int)FirstRow : ProjectionFrame.FirstVisualRow==FirstRow ? ProjectionFrame.Rows.ToList().FindIndex(r=>r.Kind==0 && r.OrdinaryRow==row) : -1;
    public uint VisualRow(uint ordinary)=>ordinary+(uint)(ProjectionFrame?.Placements.Count(p=>p.AfterRow<ordinary)??0)*(Math.Max(1,ProjectionFrame?.RecordCount??0)+2);
    public void SetProofFrame(ProofFrame? frame)
    {
        bool mode=ProjectionFrame is not null;
        var old=ProjectionFrame;
        var anchor=old is not null && old.FirstVisualRow==FirstRow ? old.Rows.FirstOrDefault() : old is null && frame is not null ? new ProofRow{Kind=0,OrdinaryRow=FirstRow} : null;
        bool reflow=frame is not null && anchor is not null && (old is null || old.Epoch!=frame.Epoch || old.Presentation!=frame.Presentation);
        ProjectionFrame=frame;
        if(frame is null){ProjectedSelection=null;MissingProjectedRecord=false;if(mode)FirstRow=1;}
        else if(ProjectedSelection is { } removed && !frame.Placements.Any(p=>p.Id==removed.Projection)){
            ProjectedSelection=null;MissingProjectedRecord=false;ProjectedSelectionChanged?.Invoke();
        }
        else if(ProjectedSelection is { } selected){
            bool exists=frame.RecordKeys.Contains(selected.Key);
            MissingProjectedRecord=frame.Generation.Length>0 && (selected.Key.Length>0?!exists:MissingProjectedRecord);
            ProjectedSelection=selected with{Generation=frame.Generation,Epoch=frame.Epoch,Presentation=frame.Presentation,Key=exists?selected.Key:"",Field=exists?selected.Field:""};
            ProjectedSelectionChanged?.Invoke();
        }
        if(reflow && frame is not null && anchor is not null){
            uint next=anchor.Kind==0?VisualRow(anchor.OrdinaryRow):ProjectedVisualRow(anchor.Projection,anchor.Key);
            next=Math.Clamp(next,1,TotalVisualRows+1-(uint)VisibleRows);
            if(next!=FirstRow){FirstRow=next;cells.Clear();ViewportChanged?.Invoke();}
        }
        InvalidateVisual();
    }
    public uint ProjectedVisualRow(string id,string key="") {
        if(ProjectionFrame is not { } f)return FirstRow;
        int index=f.Placements.ToList().FindIndex(p=>p.Id==id);if(index<0)return FirstRow;
        var p=f.Placements[index];uint height=Math.Max(1,f.RecordCount)+2;
        uint header=p.AfterRow+1+(uint)f.Placements.Where((other,i)=>other.AfterRow<p.AfterRow || other.AfterRow==p.AfterRow&&i<index).Count()*height;
        int record=f.RecordKeys.IndexOf(key);if(record<0)return header;
        int ordinal=p.Reverse?f.RecordKeys.Count-1-record:record;return header+1+(uint)ordinal;
    }
    public ProjectionTarget? ProjectionTargetAt(Point point)
    {
        var row=MappedRow((int)Math.Floor(point.Y/HeightPerCell)-1);
        if(row is null || row.Kind==0 || ProjectionFrame is not { } f)return null;
        int column=(int)FirstColumn-1+(int)Math.Floor((point.X-Header)/WidthPerCell);
        return new(row.Projection,f.Producer,f.Generation,row.Key,column>=0&&column<f.FieldIds.Count?f.FieldIds[column]:"",f.Epoch,f.Presentation);
    }
    public bool ApplyProjectionTarget(ProjectionTarget target)
    {
        if(ProjectionFrame is not { } f || f.FirstVisualRow!=FirstRow || target.Producer!=f.Producer || target.Generation!=f.Generation || target.Epoch!=f.Epoch || target.Presentation!=f.Presentation
            || !f.Rows.Any(r=>r.Projection==target.Projection && r.Key==target.Key))return false;
        ProjectedSelection=target;MissingProjectedRecord=false;InvalidateVisual();ProjectedSelectionChanged?.Invoke();return true;
    }
    private bool RenderProjection(DrawingContext context,int row,int col,Rect rect)
    {
        if(ProjectionFrame is null)return false;
        var mapped=MappedRow(row);
        if(mapped is null){context.FillRectangle(Brushes.White,rect);return true;}
        if(mapped.Kind==0)return false;
        bool selected=ProjectedSelection?.Projection==mapped.Projection && ProjectedSelection.Key==mapped.Key && (mapped.Key.Length>0 || mapped.Kind==1);
        if(mapped.Kind!=2){
            if(col<0){var band=new Rect(0,rect.Y,Header+Columns*WidthPerCell,HeightPerCell);context.DrawRectangle(Brushes.LightCyan,new Pen(selected?Brushes.SlateBlue:Brushes.LightGray,selected?2:1),band);
                using(context.PushClip(band.Deflate(3)))context.DrawText(new FormattedText(mapped.Fields.FirstOrDefault()??"",CultureInfo.InvariantCulture,FlowDirection.LeftToRight,new Typeface("Inter"),12,Brushes.DarkSlateBlue),new Point(Header+5,rect.Y+4));}
            return true;
        }
        context.DrawRectangle(selected?Brushes.LightBlue:Brushes.AliceBlue,new Pen(Brushes.LightGray,1),rect);
        int field=(int)FirstColumn+col-1;
        string text=col<0?"#"+mapped.Ordinal:field>=0&&field<mapped.Fields.Count?mapped.Fields[field]:"";
        using(context.PushClip(rect.Deflate(3)))context.DrawText(new FormattedText(text,CultureInfo.InvariantCulture,FlowDirection.LeftToRight,new Typeface("Inter"),13,Brushes.DarkSlateBlue),new Point(rect.X+5,rect.Y+4));
        return true;
    }
}
