using System;
using System.Linq;
using System.Collections.Generic;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Automation;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Media;
using Pivot.Client;
using Pivot.Protocol.Application.V1;

namespace Pivot.Desktop;

public partial class MainWindow
{
    private bool showInteriorHeadings=true;
    private long headingEpoch;
    private GeometryLocalHeader? headingDragAnchor;
    private GeometryViewport? headingDragOwner;
    private Button? headingDragButton;
    private IPointer? headingPointer;
    private GeometryLocalHeader? headingDragLast;
    private Task<bool>? headingPress;
    private readonly List<(Button Button,GeometryLocalHeader Header,Rect Bounds,bool Corner)> interiorHeadingTargets=[];
    private void ClearInteriorHeadings()
    {
        ++headingEpoch;headingDragAnchor=null;headingDragOwner=null;headingDragLast=null;
        var button=headingDragButton;headingDragButton=null;
        var pointer=headingPointer;headingPointer=null;pointer?.Capture(null);
        if(InteriorHeadingLayer.IsKeyboardFocusWithin)Sheet.Focus();
        InteriorHeadingLayer.Children.Clear();interiorHeadingTargets.Clear();
    }
    private void InteriorHeadingsClick(object? sender,RoutedEventArgs args)
    {
        SetInteriorHeadings(sender is CheckBox check?check.IsChecked==true:InteriorHeadingsToggle.IsChecked==true);
    }
    private void SetInteriorHeadings(bool visible)
    {
        if(showInteriorHeadings==visible)return;
        showInteriorHeadings=visible;InteriorHeadingsToggle.IsChecked=visible;InteriorHeadingsButton.IsChecked=visible;
        Sheet.CancelHeadingGesture();ClearInteriorHeadings();PublishInteriorHeadings();
    }
    private bool HeadingCurrent(GeometryViewport owner,long epoch)=>showInteriorHeadings&&epoch==headingEpoch&&geometryView==owner
        &&!owner.IsDisposed&&frameReady&&Sheet.GeometryInputReady&&session?.Revision==owner.Frame.Revision;
    private async Task<bool> ApplyLocalHeadingAsync(GeometryViewport owner,GeometryLocalHeader h,bool extend,bool corner=false)
    {
        bool held=headingDragOwner is not null;
        if(!held)headingDragOwner=owner;
        try{return await Sheet.SelectLocalHeadingAsync(owner,h,extend,corner);}
        finally{if(!held){headingDragOwner=null;PublishInteriorHeadings();if(Sheet.NeedsGeometryHeaders&&!editing&&!fileAction)_=RefreshGeometryAsync(true);}}
    }
    private void PublishInteriorHeadings()
    {
        if(headingDragOwner is { } held){
            if(held==geometryView&&!held.IsDisposed&&held.Frame.Revision==session?.Revision&&Sheet.GeometryInputReady)return;
            // A replacement capture cancels the old overlay gesture before the
            // new frame publishes, including a same-revision viewport refresh.
            Sheet.CancelHeadingGesture();
        }
        ClearInteriorHeadings();
        InteriorSectionMenu.Items.Clear();
        InteriorSectionMenu.IsVisible=showInteriorHeadings;
        if(geometryView is not {IsDisposed:false} owner||!frameReady||!Sheet.GeometryInputReady)return;
        var frame=owner.Frame.Geometry;var origin=frame.Viewport;double scale=Sheet.PaintedGeometryZoom/4800.0;
        var sections=frame.LocalHeaders.GroupBy(h=>(h.SectionFirst.Row,h.SectionFirst.Column)).ToArray();
        InteriorSectionMenu.Header=sections.Length>256?"Section selection — dense view; select a cell or zoom in":"Section selection";
        ToolTip.SetTip(InteriorHeadingsButton,"Interior headings prioritize the active section. In dense views, select a cell to activate its section and use the outer rulers, or zoom in.");
        foreach(var section in sections.Take(256)){
            var sample=section.First();var item=new MenuItem{Header="Section "+SortDialog.RangeText(sample.SectionFirst,sample.SectionLast)};
            var whole=new MenuItem{Header="Select whole section"};whole.Click+=async(_,_)=>await ApplyLocalHeadingAsync(owner,sample,false,true);item.Items.Add(whole);
            foreach(bool column in new[]{false,true}){
                uint index=column?Math.Clamp(Sheet.Selected.Column,sample.SectionFirst.Column,sample.SectionLast.Column):Math.Clamp(Sheet.Selected.Row,sample.SectionFirst.Row,sample.SectionLast.Row);
                var local=sample.Clone();local.Column=column;local.Index=index;
                var axis=new MenuItem{Header=column?"Select local column "+Pivot.Avalonia.WorksheetView.Address(new Position{Row=1,Column=index}).TrimEnd('1'):"Select local row "+index};
                axis.Click+=async(_,_)=>await ApplyLocalHeadingAsync(owner,local,false);item.Items.Add(axis);
            }
            InteriorSectionMenu.Items.Add(item);
        }
        if(!showInteriorHeadings)return;
        var content=new Rect(48,26,Math.Max(0,Sheet.Bounds.Width-48),Math.Max(0,Sheet.Bounds.Height-26));
        var protectedCell=Sheet.CellBounds(Sheet.Selected).Inflate(2);
        var protectedEditor=CellEditor.IsVisible?new Rect(Canvas.GetLeft(CellEditor),Canvas.GetTop(CellEditor),CellEditor.Bounds.Width,CellEditor.Bounds.Height).Inflate(2):default;
        var used=new List<Rect>();
        // Active section receives priority under density. No declared-grid enumeration.
        bool Active(GeometryLocalHeader h)=>Sheet.Selected.Row>=h.SectionFirst.Row&&Sheet.Selected.Row<=h.SectionLast.Row
            &&Sheet.Selected.Column>=h.SectionFirst.Column&&Sheet.Selected.Column<=h.SectionLast.Column;
        long epoch=headingEpoch;
        // Reserve the section corners before row labels. At a scope edge the
        // outer ruler supplies the corner gutter; cell-label centers stay put.
        foreach(var section in sections.OrderByDescending(g=>Active(g.First())).Take(256)){
            var sample=section.First();if(sample.SectionFirst.Row==1&&sample.SectionFirst.Column==1)continue;
            var column=section.FirstOrDefault(h=>h.Column);var row=section.FirstOrDefault(h=>!h.Column);
            if(column is null||row is null)continue;
            double x=48+(row.Edge-origin.Left)*scale,y=26+(column.Edge-origin.Top)*scale;
            var corner=new Rect(Math.Max(24,x-24),Math.Max(4,y-22),24,22);
            if(new Rect(Sheet.Bounds.Size).Contains(corner)&&!corner.Intersects(protectedCell)
                &&(!CellEditor.IsVisible||!corner.Intersects(protectedEditor))&&!used.Any(r=>r.Intersects(corner))){Add(sample,corner,"▦",true);used.Add(corner);}
        }
        foreach(var h in frame.LocalHeaders.OrderByDescending(Active)){
            if(interiorHeadingTargets.Count>=256)break;
            bool interior=h.Column?h.SectionFirst.Row>1:h.SectionFirst.Column>1;
            if(!interior)continue;
            double first=(h.First-(h.Column?origin.Left:origin.Top))*scale+(h.Column?48:26);
            double last=(h.Last-(h.Column?origin.Left:origin.Top))*scale+(h.Column?48:26);
            double edge=(h.Edge-(h.Column?origin.Top:origin.Left))*scale+(h.Column?26:48);
            // Leading-edge overlays become sticky at the body edge. They never
            // change canonical cell rectangles or the outer geometry dividers.
            var bounds=h.Column?new Rect(first,Math.Max(26,edge-22),last-first,22):new Rect(Math.Max(48,edge-40),first,40,last-first);
            bounds=bounds.Intersect(content);
            if(bounds.Width<18||bounds.Height<16||bounds.Intersects(protectedCell)||(CellEditor.IsVisible&&bounds.Intersects(protectedEditor))||used.Any(r=>r.Intersects(bounds)))continue;
            string axis=h.Column?Pivot.Avalonia.WorksheetView.Address(new Position{Row=1,Column=h.Index}).TrimEnd('1'):h.Index.ToString(System.Globalization.CultureInfo.InvariantCulture);
            Add(h,bounds,axis,false);used.Add(bounds);
        }
        // Arrange this bounded overlay before a consumer can render the new
        // cell frame. Fresh buttons must not exist only as unlaid-out targets.
        InteriorHeadingLayer.UpdateLayout();
        void Add(GeometryLocalHeader h,Rect bounds,string label,bool corner)
        {
            var button=new Button{Content=label,Width=bounds.Width,Height=bounds.Height,MinWidth=0,MinHeight=0,Padding=new Thickness(1),FontSize=11,Focusable=true,
                Background=Brushes.WhiteSmoke,Foreground=Brushes.Black,BorderBrush=Brushes.SlateGray,BorderThickness=new Thickness(1),
                HorizontalContentAlignment=global::Avalonia.Layout.HorizontalAlignment.Center,VerticalContentAlignment=global::Avalonia.Layout.VerticalAlignment.Center};
            string range=SortDialog.RangeText(h.SectionFirst,h.SectionLast);
            string name=$"{(corner?"Select section":h.Column?"Select local column "+label:"Select local row "+label)} · {range}"+(frame.Positional?" · Positional presentation, read-only":"");
            AutomationProperties.SetName(button,name);ToolTip.SetTip(button,name);
            Canvas.SetLeft(button,bounds.Left);Canvas.SetTop(button,bounds.Top);
            button.Click+=async(_,_)=>{if(HeadingCurrent(owner,epoch))await ApplyLocalHeadingAsync(owner,h,false,corner);};
            button.AddHandler(InputElement.PointerPressedEvent,(_,e)=>{
                if(!e.GetCurrentPoint(button).Properties.IsLeftButtonPressed||!HeadingCurrent(owner,epoch))return;
                e.Handled=true;headingDragOwner=owner;headingDragAnchor=h;headingDragLast=h;headingDragButton=button;headingPointer=e.Pointer;
                e.Pointer.Capture(button);
                headingPress=ApplyLocalHeadingAsync(owner,h,e.KeyModifiers.HasFlag(KeyModifiers.Shift),corner);
            },RoutingStrategies.Tunnel);
            button.PointerMoved+=(_,e)=>{
                if(headingDragButton!=button||corner)return;
                var point=e.GetPosition(InteriorHeadingLayer);
                var target=interiorHeadingTargets.LastOrDefault(t=>!t.Corner&&t.Header.Column==h.Column&&t.Bounds.Contains(point));
                if(target.Button is not null)headingDragLast=target.Header;
            };
            button.PointerReleased+=async(_,e)=>{
                if(headingDragButton!=button)return;
                e.Handled=true;var last=headingDragLast;var start=headingDragAnchor;var press=headingPress;
                try{
                    if(press is not null&&await press&&last is not null&&start is not null&&!last.Equals(start)&&HeadingCurrent(owner,epoch))
                        await ApplyLocalHeadingAsync(owner,last,true);
                }finally{
                    if(headingDragButton==button){headingDragButton=null;headingPointer=null;headingDragAnchor=null;headingDragLast=null;headingDragOwner=null;e.Pointer.Capture(null);PublishInteriorHeadings();if(Sheet.NeedsGeometryHeaders&&!editing&&!fileAction)_=RefreshGeometryAsync(true);}
                }
            };
            button.PointerCaptureLost+=(_,_)=>{if(headingDragButton==button){headingDragButton=null;headingPointer=null;headingDragAnchor=null;headingDragLast=null;headingDragOwner=null;Sheet.CancelHeadingGesture();PublishInteriorHeadings();}};
            button.AddHandler(InputElement.KeyDownEvent,async(_,e)=>{
                if(e.Key==Key.Escape){e.Handled=true;Sheet.CancelHeadingGesture();ClearInteriorHeadings();PublishInteriorHeadings();Sheet.Focus();}
                if((e.Key is Key.Enter or Key.Space)&&e.KeyModifiers.HasFlag(KeyModifiers.Shift)){
                    e.Handled=true;if(HeadingCurrent(owner,epoch))await ApplyLocalHeadingAsync(owner,h,true,corner);
                }
            },RoutingStrategies.Tunnel);
            InteriorHeadingLayer.Children.Add(button);interiorHeadingTargets.Add((button,h,bounds,corner));
        }
    }
}
