using System;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Automation;
using Avalonia;
using Avalonia.Input;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Styling;
using Avalonia.Media.Imaging;
using System.Runtime.InteropServices;
using Pivot.Protocol.Application.V1;

namespace Pivot.Desktop;

public partial class MainWindow
{
    // Runs in the virtual platform and in separately consented native GUI suites.
    private async Task InteriorHeadingsAutomationAsync(string output)
    {
        static Position P(uint r,uint c)=>new(){Row=r,Column=c};
        var owner=session!;await AutomationWaitForGeometryFrameAsync();SetInteriorHeadings(true);
        AutomationCheck(geometryView!.Frame.Geometry.LocalHeaders.Count>0,"Captured local heading descriptors");
        async Task Select(uint row,uint col,bool column,uint index,Position first,Position last,bool corner=false)
        {
            await AutomationWaitForGeometryFrameAsync();
            var view=geometryView!;
            var heading=view.Frame.Geometry.LocalHeaders.First(h=>h.SectionFirst.Row==row&&h.SectionFirst.Column==col&&h.Column==column&&h.Index==index);
            AutomationCheck(await ApplyLocalHeadingAsync(view,heading,false,corner),"Local heading selection accepted");
            await AutomationWaitForGeometryFrameAsync();await Sheet.EnsureSectionSelectionAsync();
            AutomationCheck(Sheet.RequestedFirst.Equals(first)&&Sheet.RequestedLast.Equals(last)&&Sheet.SelectionFirst.Equals(first)&&Sheet.SelectionLast.Equals(last),"Exact section-local axis bounds");
        }
        await Select(5,1,true,3,P(5,3),P(28,3));
        await Select(5,1,false,10,P(10,1),P(10,6));
        await Select(5,7,false,10,P(10,7),P(10,8));
        await Select(5,1,true,3,P(5,1),P(28,6),corner:true);
        await SelectAsync(P(1,1));await AutomationWaitForGeometryFrameAsync();PublishInteriorHeadings();
        AutomationCheck(interiorHeadingTargets.Count>0&&interiorHeadingTargets.Count<=256,"Invoice displays bounded interior headings");
        AutomationCheck(interiorHeadingTargets.Any(t=>t.Corner),"Ordinary invoice exposes a visible section corner target");
        AutomationCheck(interiorHeadingTargets.All(t=>t.Button.Bounds.Width>0&&t.Button.Bounds.Height>0),"Published heading targets have actual arranged visual bounds");
        AutomationCheck(interiorHeadingTargets.All(t=>!t.Bounds.Intersects(Sheet.CellBounds(Sheet.Selected).Inflate(2)))&&interiorHeadingTargets.All(t=>!string.IsNullOrEmpty(AutomationProperties.GetName(t.Button))),"Heading accessibility and unobscured active cell");
        await InteriorHeadingContrastAutomationAsync(output);
        AutomationImage(Path.Combine(output,"interior-headings.png"));
        var from=interiorHeadingTargets.First(t=>!t.Corner&&t.Header.Column&&t.Header.SectionFirst.Row==5&&t.Header.Index==3);
        var to=interiorHeadingTargets.First(t=>!t.Corner&&t.Header.Column&&t.Header.SectionFirst.Row==5&&t.Header.Index==7);
        var pointer=new Pointer(1001,PointerType.Mouse,true);
        var held=new PointerPointProperties(global::Avalonia.Input.RawInputModifiers.LeftMouseButton,PointerUpdateKind.LeftButtonPressed);
        var released=new PointerPointProperties(global::Avalonia.Input.RawInputModifiers.None,PointerUpdateKind.LeftButtonReleased);
        Point WindowPoint(Rect rect)=>InteriorHeadingLayer.TranslatePoint(rect.Center,this)!.Value;
        from.Button.RaiseEvent(new PointerPressedEventArgs(from.Button,pointer,this,WindowPoint(from.Bounds),0,held,KeyModifiers.None,1));
        from.Button.RaiseEvent(new PointerEventArgs(InputElement.PointerMovedEvent,from.Button,pointer,this,WindowPoint(to.Bounds),0,held,KeyModifiers.None));
        from.Button.RaiseEvent(new PointerReleasedEventArgs(from.Button,pointer,this,WindowPoint(to.Bounds),0,released,KeyModifiers.None,MouseButton.Left));
        await AutomationWaitForGeometryFrameAsync();await Sheet.EnsureSectionSelectionAsync();
        AutomationCheck(Sheet.SectionSelectionEnlarged&&Sheet.SelectionFirst.Equals(P(5,1))&&Sheet.SelectionLast.Equals(P(28,8)),"Routed local-heading drag crosses the vertical section boundary");
        await SelectAsync(P(1,1));await AutomationWaitForGeometryFrameAsync();PublishInteriorHeadings();
        var target=interiorHeadingTargets.First(t=>!t.Corner);var oldButton=target.Button;
        var revision=owner.Revision;var selected=Sheet.RequestedFirst;var end=Sheet.RequestedLast;int zoom=Sheet.GeometryZoomPercent;
        InteriorHeadingsButton.IsChecked=false;InteriorHeadingsButton.RaiseEvent(new RoutedEventArgs(CheckBox.ClickEvent));
        AutomationCheck(!showInteriorHeadings&&InteriorHeadingLayer.Children.Count==0&&interiorHeadingTargets.Count==0&&!InteriorSectionMenu.IsVisible,"Hidden headings have no controls or targets");
        oldButton.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
        await Task.Delay(25);
        AutomationCheck(owner.Revision==revision&&Sheet.RequestedFirst.Equals(selected)&&Sheet.RequestedLast.Equals(end)&&Sheet.GeometryZoomPercent==zoom,"Toggle and obsolete heading preserve selection and revisions");
        InteriorHeadingsToggle.IsChecked=true;InteriorHeadingsToggle.RaiseEvent(new RoutedEventArgs(MenuItem.ClickEvent));
        AutomationCheck(showInteriorHeadings&&InteriorHeadingsButton.IsChecked==true&&InteriorHeadingLayer.Children.Count>0,"Master menu and toolbar restore both heading orientations");
        var unfinished=interiorHeadingTargets.First(t=>!t.Corner);
        unfinished.Button.RaiseEvent(new PointerPressedEventArgs(unfinished.Button,pointer,this,WindowPoint(unfinished.Bounds),0,held,KeyModifiers.None,1));
        SetInteriorHeadings(false);await Task.Delay(20);
        var stoppedFirst=Sheet.RequestedFirst;var stoppedLast=Sheet.RequestedLast;
        unfinished.Button.RaiseEvent(new PointerReleasedEventArgs(unfinished.Button,pointer,this,WindowPoint(unfinished.Bounds),0,released,KeyModifiers.None,MouseButton.Left));
        await Task.Delay(20);
        AutomationCheck(headingDragOwner is null&&pointer.Captured is null&&Sheet.RequestedFirst.Equals(stoppedFirst)&&Sheet.RequestedLast.Equals(stoppedLast)&&owner.Revision==revision,"Toggle cancels a captured heading gesture without publishing or replay");
        SetInteriorHeadings(true);await AutomationWaitForGeometryFrameAsync();
        var obsolete=interiorHeadingTargets.First(t=>!t.Corner);
        obsolete.Button.RaiseEvent(new PointerPressedEventArgs(obsolete.Button,pointer,this,WindowPoint(obsolete.Bounds),0,held,KeyModifiers.None,1));
        await RefreshAsync();await AutomationWaitForGeometryFrameAsync();
        var refreshFirst=Sheet.RequestedFirst;var refreshLast=Sheet.RequestedLast;
        AutomationCheck(headingDragOwner is null&&pointer.Captured is null,"Replacement frame cancels the previous heading capture before overlay publication");
        obsolete.Button.RaiseEvent(new PointerReleasedEventArgs(obsolete.Button,pointer,this,WindowPoint(obsolete.Bounds),0,released,KeyModifiers.None,MouseButton.Left));
        obsolete.Button.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));await Task.Delay(20);
        AutomationCheck(Sheet.RequestedFirst.Equals(refreshFirst)&&Sheet.RequestedLast.Equals(refreshLast),"Obsolete overlay cannot replay after a same-revision capture refresh");
        // Route the real Button activation path, rather than only its selection helper.
        var clicked=interiorHeadingTargets.First(t=>!t.Corner);var header=clicked.Header.Clone();
        clicked.Button.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
        await AutomationWaitForGeometryFrameAsync();await Sheet.EnsureSectionSelectionAsync();
        var expectedFirst=header.SectionFirst.Clone();var expectedLast=header.SectionLast.Clone();
        if(header.Column)expectedFirst.Column=expectedLast.Column=header.Index;else expectedFirst.Row=expectedLast.Row=header.Index;
        AutomationCheck(Sheet.RequestedFirst.Equals(expectedFirst)&&Sheet.RequestedLast.Equals(expectedLast),"Actual heading Button selects its own local axis");
        await Select(5,1,true,3,P(5,3),P(28,3));
        await AutomationWaitForGeometryFrameAsync();
        var right=interiorHeadingTargets.First(t=>!t.Corner&&t.Header.Column&&t.Header.Index==7&&t.Header.SectionFirst.Row==5&&t.Header.SectionFirst.Column==7);
        right.Button.RaiseEvent(new KeyEventArgs{RoutedEvent=InputElement.KeyDownEvent,Key=Key.Enter,KeyModifiers=KeyModifiers.Shift});
        await AutomationWaitForGeometryFrameAsync();await Sheet.EnsureSectionSelectionAsync();
        AutomationCheck(Sheet.SectionSelectionEnlarged&&Sheet.SelectionFirst.Equals(P(5,1))&&Sheet.SelectionLast.Equals(P(28,8)),"Heading extension preserves canonical either-boundary closure");
        SetInteriorHeadings(false);AutomationCheck(Sheet.SectionSelectionEnlarged,"Hiding headings retains expanded-operation confirmation");SetInteriorHeadings(true);
        string invoice=owner.WorksheetId;
        double minimumHeight=Sheet.MinHeight,maximumHeight=Sheet.MaxHeight;
        // Reproduce the smaller native viewport instead of assuming row 10 is
        // already materialized in a full-size virtual window.
        Sheet.MinHeight=180;Sheet.MaxHeight=234;
        try{
        foreach(bool horizontal in new[]{true,false}){
            var single=await owner.CreateWindowAsync(owner.Revision,horizontal?"Horizontal headings":"Vertical headings",12,8);owner.SelectWindow(single.Worksheet);
            await owner.EditGeometryAsync(owner.Revision,[new GeometryCommand{Action=GeometryCommand.Types.Action.AddBreak,Horizontal=horizontal,At=horizontal?5u:7u}]);
            await RefreshAsync();await AutomationWaitForGeometryFrameAsync();
            Sheet.Reveal(P(10,horizontal?1u:7u));await AutomationWaitForGeometryFrameAsync();
            var view=geometryView!;
            var row=view.Frame.Geometry.LocalHeaders.First(h=>!h.Column&&h.Index==10&&h.SectionFirst.Row==(horizontal?5u:1u)&&h.SectionFirst.Column==(horizontal?1u:7u));
            AutomationCheck(await ApplyLocalHeadingAsync(view,row,false),"Single-boundary local row accepted");
            await AutomationWaitForGeometryFrameAsync();await Sheet.EnsureSectionSelectionAsync();
            AutomationCheck(Sheet.SelectionFirst.Equals(P(10,horizontal?1u:7u))&&Sheet.SelectionLast.Equals(P(10,8)),"Single horizontal or vertical boundary selects its exact local row");
            if(horizontal){
                Sheet.Reveal(P(5,3));await AutomationWaitForGeometryFrameAsync();
                view=geometryView!;var column=view.Frame.Geometry.LocalHeaders.First(h=>h.Column&&h.Index==3&&h.SectionFirst.Row==5);
                AutomationCheck(await ApplyLocalHeadingAsync(view,column,false),"Horizontal-only local column accepted");
                await AutomationWaitForGeometryFrameAsync();await Sheet.EnsureSectionSelectionAsync();
                AutomationCheck(Sheet.SelectionFirst.Equals(P(5,3))&&Sheet.SelectionLast.Equals(P(12,3)),"Horizontal-only heading clips to the real tail");
            }
        }
        }finally{Sheet.MaxHeight=maximumHeight;Sheet.MinHeight=minimumHeight;}
        owner.SelectWindow(invoice);await RefreshAsync();await AutomationWaitForGeometryFrameAsync();
        Directory.CreateDirectory(output);await File.WriteAllTextAsync(Path.Combine(output,"section-headings.txt"),"passed: local column, left/right local row, section corner, actual Button activation, cross-boundary closure, toggle, obsolete targets, accessibility names and active-cell visibility\n");
        await SelectAsync(P(1,1));await AutomationWaitForGeometryFrameAsync();
    }

    private async Task InteriorHeadingContrastAutomationAsync(string output)
    {
        var original=RequestedThemeVariant;
        try{
            foreach(var theme in new[]{ThemeVariant.Light,ThemeVariant.Dark}){
                RequestedThemeVariant=theme;UpdateLayout();await AutomationWaitForGeometryFrameAsync();PublishInteriorHeadings();
                await global::Avalonia.Threading.Dispatcher.UIThread.InvokeAsync(()=>{},global::Avalonia.Threading.DispatcherPriority.Background);
                var target=interiorHeadingTargets.First(t=>!t.Corner&&t.Header.Column&&t.Header.Index==3&&t.Header.SectionFirst.Row==5);
                var location=target.Button.TranslatePoint(default,this)!.Value;
                int width=(int)Math.Ceiling(Bounds.Width),height=(int)Math.Ceiling(Bounds.Height),stride=checked(width*4),length=checked(stride*height);
                using var bitmap=new RenderTargetBitmap(new PixelSize(width,height));bitmap.Render(this);
                var pixels=new byte[length];var memory=Marshal.AllocHGlobal(length);
                try{bitmap.CopyPixels(new PixelRect(0,0,width,height),memory,length,stride);Marshal.Copy(memory,pixels,0,length);}
                finally{Marshal.FreeHGlobal(memory);}
                int left=Math.Clamp((int)Math.Floor(location.X+target.Button.Bounds.Width/2-10),0,width-1);
                int right=Math.Clamp((int)Math.Ceiling(location.X+target.Button.Bounds.Width/2+10),left+1,width);
                int top=Math.Clamp((int)Math.Floor(location.Y+3),0,height-1),bottom=Math.Clamp((int)Math.Ceiling(location.Y+target.Button.Bounds.Height-3),top+1,height);
                int dark=0,light=0;
                for(int y=top;y<bottom;y++)for(int x=left;x<right;x++){
                    int p=y*stride+x*4;
                    if(pixels[p]<160&&pixels[p+1]<160&&pixels[p+2]<160)dark++;
                    if(pixels[p]>190&&pixels[p+1]>190&&pixels[p+2]>190)light++;
                }
                bitmap.Save(Path.Combine(output,"interior-headings-"+theme.Key+".png"),PngBitmapEncoderOptions.Default);
                await File.WriteAllTextAsync(Path.Combine(output,"heading-contrast-"+theme.Key+".txt"),$"glyphPixels={dark}; lightBackgroundPixels={light}\n");
                AutomationCheck(dark>=6&&light>=30,$"Readable centered heading glyph against worksheet chrome in {theme}; glyphPixels={dark}; lightBackgroundPixels={light}");
            }
        }finally{RequestedThemeVariant=original;UpdateLayout();await AutomationWaitForGeometryFrameAsync();PublishInteriorHeadings();}
    }
}
