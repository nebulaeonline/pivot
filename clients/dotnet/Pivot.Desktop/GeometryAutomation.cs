using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Input;
using Avalonia.Controls;
using Avalonia.Input.Platform;
using Avalonia.Interactivity;
using Avalonia.LogicalTree;
using Pivot.Client;
using Pivot.Protocol.Application.V1;
namespace Pivot.Desktop;
public partial class MainWindow
{
    private async Task AutomationWaitForGeometryFrameAsync()
    {
        var clock=System.Diagnostics.Stopwatch.StartNew();
        while(clock.Elapsed<TimeSpan.FromSeconds(15)){
            var owner=session;var view=geometryView;long request=displayRequest;
            bool Current()=>owner is not null&&owner==session&&view is not null&&view==geometryView&&!view.IsDisposed
                &&request==displayRequest&&frameReady&&Sheet.GeometryInputReady&&Sheet.GeometryHeaderInputReady
                &&!Sheet.NeedsGeometryHeaders&&!editing&&!fileAction&&activeGeometryEditor is null
                &&view.DocumentId==owner.DocumentId&&view.DocumentGeneration==owner.DocumentGeneration
                &&view.Frame.Revision==owner.Revision&&view.Frame.Worksheet==owner.WorksheetId&&view.Frame.WindowScope==owner.IsWindowScope;
            if(Current()&&(await Sheet.EnsureSectionSelectionAsync()||(Sheet.SelectionNeedsReview&&Sheet.SectionSelectionError.Length==0))){
                try{await Sheet.PaintCompletion.WaitAsync(TimeSpan.FromMilliseconds(250));}
                catch(OperationCanceledException){continue;}
                catch(TimeoutException){continue;}
                await global::Avalonia.Threading.Dispatcher.UIThread.InvokeAsync(()=>{},global::Avalonia.Threading.DispatcherPriority.Background);
                if(Current())return;
            }
            await Task.Delay(10);
        }
        throw new TimeoutException($"No stable current geometry frame; frame={frameReady}; geometry={Sheet.GeometryInputReady}; headers={Sheet.GeometryHeaderInputReady}; editing={editing}; pendingHeaders={Sheet.NeedsGeometryHeaders}; qualified={displayedRevision==session?.Revision}; selection={Sheet.SectionSelectionError}; status={StatusLabel.Text}");
    }
    // Shared by the virtual controls gate and the separately consented native GUI suites.
    private async Task GeometryAutomationAsync(string output)
    {
        Directory.CreateDirectory(output);
        var Evidence=new List<string>();
        var editorTasks=new Dictionary<Window,Task>();
        static Position P(uint row,uint column=1)=>new(){Row=row,Column=column};
        static void Check(bool condition,string message)=>AutomationCheck(condition,message);
        static T Control<T>(Window dialog,string name) where T:Control=>dialog.GetLogicalDescendants().OfType<T>().SingleOrDefault(c=>c.Name==name)??throw new InvalidOperationException("Missing geometry control "+name);
        static void Click(Window dialog,string name)=>Control<Button>(dialog,name).RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
        static async Task Wait(Func<bool> condition,string message){var clock=System.Diagnostics.Stopwatch.StartNew();while(!condition()){if(clock.Elapsed>TimeSpan.FromSeconds(15))throw new TimeoutException(message);await Task.Delay(10);}}
        var previousText=Clipboard is null?null:await Clipboard.TryGetTextAsync();
        try {
            var owner=session!;var sheet=Sheet;var window=this;
            await owner.NewAsync(owner.Revision,discard:true);await RefreshAsync();
            var created=await owner.CreateWindowAsync(owner.Revision,"Invoice",36,8);owner.SelectWindow(created.Worksheet);
            await owner.ApplyAsync(owner.Revision,[new Edit{At=P(1),Text="000042"},new Edit{At=P(2),Text="001234"}]);
            await RefreshAsync();sheet.Select(P(1));await sheet.EnsureSectionSelectionAsync();
            async Task<Window> Open(){await AutomationWaitForGeometryFrameAsync();var task=EditGeometryAsync();await Wait(()=>{if(task.IsCompleted){task.GetAwaiter().GetResult();throw new InvalidOperationException("Geometry editor refused: "+window.FindControl<TextBlock>("StatusLabel")!.Text+"; ready="+sheet.GeometryInputReady+"; selection="+sheet.SectionSelectionError+"; editing="+editing);}return activeGeometryEditor?.IsVisible==true;},"Geometry editor opened");var dialog=activeGeometryEditor!;editorTasks.Add(dialog,task);return dialog;}
            void Break(Window dialog,bool horizontal,uint at){Control<ComboBox>(dialog,"SectionOrientation").SelectedIndex=horizontal?0:1;Control<NumericUpDown>(dialog,"SectionAt").Value=at;Click(dialog,"SectionAddBreak");}
            void Size(Window dialog,bool horizontal,uint at,uint first,uint last,decimal points,bool shared=false,bool paired=false){
                Control<ComboBox>(dialog,"SectionOrientation").SelectedIndex=horizontal?0:1;
                Control<NumericUpDown>(dialog,"SectionAt").Value=at;Control<NumericUpDown>(dialog,"SectionFirst").Value=first;Control<NumericUpDown>(dialog,"SectionLast").Value=last;Control<NumericUpDown>(dialog,"SectionSize").Value=points;
                Control<CheckBox>(dialog,"SectionShared").IsChecked=shared;Control<CheckBox>(dialog,"SectionPaired").IsChecked=paired;Click(dialog,"SectionQueueSize");
            }
            async Task Apply(Window dialog,string step){string before=owner.Revision;Click(dialog,"SectionPreview");await Wait(()=>Control<Button>(dialog,"SectionApply").IsEnabled,"Valid batch preview: "+step+"; "+Control<TextBlock>(dialog,"SectionPreviewStatus").Text);Check(owner.Revision==before,"Preview cannot publish");Click(dialog,"SectionApply");await editorTasks[dialog];await AutomationWaitForGeometryFrameAsync();Check(!dialog.IsVisible&&activeGeometryEditor==null,"Applied and refreshed "+step);Evidence.Add(step);}
            var cancelled=await Open();Break(cancelled,true,5);string revision=owner.Revision;Click(cancelled,"SectionCancel");await editorTasks[cancelled];await Wait(()=>!cancelled.IsVisible,"Cancelled geometry dialog closes");await AutomationWaitForGeometryFrameAsync();Check(owner.Revision==revision,"Cancel preserves workbook revision after the current frame settles");Evidence.Add("cancel queued boundary without publication");
            var dialog=await Open();Break(dialog,true,5);Break(dialog,true,29);await Apply(dialog,"add independent horizontal sections from defaults");
            dialog=await Open();
            decimal[][] widths=[[160,32,32,64,64,128,60,36],[240,48,48,48,48,48,60,36],[96,96,96,64,64,64,60,36]];
            uint[] starts=[1,5,29];
            for(int band=0;band<3;band++)for(uint column=1;column<=8;column++)Size(dialog,true,starts[band],column,column,widths[band][column-1]);
            Size(dialog,false,1,1,36,18);await Apply(dialog,"set all three width profiles and shared row heights in one ordinary numeric batch");
            dialog=await Open();Break(dialog,false,7);await Apply(dialog,"add vertical section after local widths already exist");
            dialog=await Open();Size(dialog,false,7,1,1,36);Size(dialog,false,7,2,3,9);Size(dialog,false,7,29,29,36);Size(dialog,false,7,30,31,9);await Apply(dialog,"revise right section heights while preserving authored section seams");
            await using(var capture=await owner.LoadGeometryViewportAsync(P(1),new GeometryRect{Right=576*64,Bottom=648*64})){
                var mapped=capture.Frame.Geometry.Cells;
                Check(mapped.Count==288,"Complete 36 by 8 invoice");
                Check(mapped.Single(c=>c.At.Equals(P(5))).Rectangle.Top==72*64,"First horizontal seam");
                Check(mapped.Single(c=>c.At.Equals(P(29))).Rectangle.Top==504*64,"Totals seam");
                Check(mapped.Single(c=>c.At.Equals(P(1,7))).Rectangle.Left==480*64,"Aligned vertical seam");
                Check(capture.Frame.Cells.Single(c=>c.At.Equals(P(1))).Result.Text=="000042"&&capture.Frame.Cells.Single(c=>c.At.Equals(P(2))).Result.Text=="001234","Invoice geometry preserves leading-zero Strings");
            }
            dialog=await Open();Size(dialog,true,5,2,2,52,paired:true);await Apply(dialog,"revise existing table divider using balanced adjacent sizes");
            await SelectAsync(P(5,2));await RefreshAsync();await AutomationWaitForGeometryFrameAsync();
            var divider=sheet.CellBounds(P(5,2));var pointer=new Pointer(994,PointerType.Mouse,true);
            var held=new PointerPointProperties(global::Avalonia.Input.RawInputModifiers.LeftMouseButton,PointerUpdateKind.LeftButtonPressed);
            var released=new PointerPointProperties(global::Avalonia.Input.RawInputModifiers.None,PointerUpdateKind.LeftButtonReleased);
            var from=new Point(divider.Right,13);var to=from+new Vector(4,0);
            var resize=sheet.GeometryResizeRequested!;Task? resizeTask=null;
            sheet.GeometryResizeRequested=(command,description)=>resizeTask=resize(command,description);
            try {
            sheet.RaiseEvent(new PointerPressedEventArgs(sheet,pointer,this,sheet.TranslatePoint(from,this)!.Value,0,held,KeyModifiers.None,1));
            Check(sheet.IsSelecting,"Header divider begins a qualified pointer gesture");
            sheet.RaiseEvent(new PointerEventArgs(InputElement.PointerMovedEvent,sheet,pointer,this,sheet.TranslatePoint(to,this)!.Value,0,held,KeyModifiers.None));
            sheet.RaiseEvent(new PointerReleasedEventArgs(sheet,pointer,this,sheet.TranslatePoint(to,this)!.Value,0,released,KeyModifiers.None,MouseButton.Left));
            await Wait(()=>{if(resizeTask?.IsCompleted==true&&activeGeometryEditor?.IsVisible!=true){resizeTask.GetAwaiter().GetResult();throw new InvalidOperationException($"Divider editor refused; frame={frameReady}; geometry={sheet.GeometryInputReady}; headers={sheet.GeometryHeaderInputReady}; editing={editing}; qualified={displayedRevision==owner.Revision}; selection={sheet.SectionSelectionError}; status={StatusLabel.Text}");}return activeGeometryEditor?.IsVisible==true;},"Divider opens exact adjacent-size preview; resize callback "+(resizeTask is null?"not invoked":"pending"));dialog=activeGeometryEditor!;
            editorTasks.Add(dialog,resizeTask??throw new InvalidOperationException("Divider did not invoke the resize command"));
            await Apply(dialog,"drag the contextual divider and preview both adjacent sizes before publication");
            }finally{sheet.GeometryResizeRequested=resize;}
            foreach(int zoom in new[]{25,75,125,200,100}){
                sheet.SetGeometryZoom(zoom);await Wait(()=>sheet.GeometryInputReady&&sheet.PaintedGeometryZoom==zoom,"Coherent zoom frame");
                var cell=sheet.CellBounds(P(1));Check(Math.Abs(cell.Width-160*4.0/3*zoom/100)<0.00001,"Zoom uses canonical fixed-unit rectangles");
                if(zoom==25){
                    await SelectAsync(P(1));sheet.RaiseEvent(new KeyEventArgs{RoutedEvent=InputElement.KeyDownEvent,Key=Key.PageDown});
                    await Wait(()=>sheet.Selected.Row==36,"At 25 percent the complete 36-row invoice fits one canonical page");
                    await SelectAsync(P(1));
                }
            }
            Evidence.Add("painted geometry remains coherent at 25, 75, 100, 125 and 200 percent zoom");
            dialog=await Open();Size(dialog,true,1,6,6,132,shared:true,paired:true);await Apply(dialog,"explicit shared seam batch updates all perpendicular profiles");
            await SelectAsync(P(2,2));await SelectWholeScopeAxisAsync(false);Check(sheet.RequestedFirst.Column==1&&sheet.RequestedLast.Column==8,"Explicit whole-scope row action");Check(sheet.SelectionFirst.Row==1&&sheet.SelectionLast.Row==4,"Whole-scope row action uses the same section closure");
            await SelectAsync(P(2,2));await SelectWholeScopeAxisAsync(true);Check(sheet.RequestedFirst.Row==1&&sheet.RequestedLast.Row==36,"Explicit whole-scope column action: "+sheet.RequestedFirst+":"+sheet.RequestedLast+"; "+StatusLabel.Text);Check(sheet.SelectionFirst.Column==1&&sheet.SelectionLast.Column==6,"Whole-scope column action uses the same section closure");
            Evidence.Add("contextual rulers have explicit whole-scope row and column selection actions");
            // Confirm the actual expanded Clear invocation, including stale acceptance.
            async Task<Window> ExpandedClear(){
                await Wait(()=>sheet.GeometryInputReady&&!editing,"Clear ready");sheet.Select(P(4));sheet.Select(P(5,3),true);Check(await sheet.EnsureSectionSelectionAsync(),"Qualified expanded selection");Check(sheet.SectionSelectionEnlarged,"Crossing either boundary expands selection");
                _=ClearSelectionAsync();await Wait(()=>activeSectionConfirmation?.IsVisible==true,"Expanded Clear confirmation");return activeSectionConfirmation!;
            }
            var confirmation=await ExpandedClear();string unchanged=owner.Revision;confirmation.Close(false);
            await Wait(()=>activeSectionConfirmation==null&&!editing,"Cancelled Clear");Check(owner.Revision==unchanged,"Cancel expanded operation preserves state");Evidence.Add("cancel explicit expanded Clear confirmation");
            confirmation=await ExpandedClear();await owner.ApplyAsync(owner.Revision,[new Edit{At=P(1),Text="000042"},new Edit{At=P(3),Text="background change"}]);string current=owner.Revision;confirmation.Close(true);
            await Wait(()=>activeSectionConfirmation==null&&!editing,"Stale Clear refused");Check(owner.Revision==current,"Stale confirmation cannot dispatch destructive work");Evidence.Add("refuse stale expanded Clear acceptance after workbook publication");
            await RefreshAsync();confirmation=await ExpandedClear();confirmation.Close(true);
            await Wait(()=>activeSectionConfirmation==null&&!editing,"Confirmed Clear completed");
            Check(owner.Revision!=current,"Confirmed Clear changes the workbook");
            await using(var cleared=await owner.LoadGeometryViewportAsync(P(1),new GeometryRect{Right=576*64,Bottom=648*64}))Check(cleared.Frame.Cells.All(c=>c.Result?.KindCase is TypedValue.KindOneofCase.Blank or null),"Clear applies to all complete touched sections");
            Evidence.Add("apply Clear to the complete expanded sections only after explicit confirmation");
            dialog=await Open();Click(dialog,"SectionResetSizes");await Apply(dialog,"reset sizes while retaining breaks");
            dialog=await Open();Click(dialog,"SectionResetAll");await Apply(dialog,"reset all geometry");
            // Repeat in a different normal construction order, without a preinstalled fixture.
            dialog=await Open();Break(dialog,false,7);Size(dialog,false,1,1,36,18,shared:true);Break(dialog,true,5);Break(dialog,true,29);
            for(int band=0;band<3;band++)for(uint column=1;column<=8;column++)Size(dialog,true,starts[band],column,column,widths[band][column-1]);
            Size(dialog,false,7,1,1,36);Size(dialog,false,7,2,3,9);Size(dialog,false,7,29,29,36);Size(dialog,false,7,30,31,9);await Apply(dialog,"construct invoice with vertical-first order in one normal batch");
            // Every expanded bulk operation uses the same real Desktop modal guard.
            var guardScope=await owner.CreateWindowAsync(owner.Revision,"Operation guards",16,16);owner.SelectWindow(guardScope.Worksheet);
            await owner.EditGeometryAsync(owner.Revision,[new GeometryCommand{Action=GeometryCommand.Types.Action.AddBreak,Horizontal=true,At=5},new GeometryCommand{Action=GeometryCommand.Types.Action.AddBreak,Horizontal=true,At=9},new GeometryCommand{Action=GeometryCommand.Types.Action.AddBreak,Horizontal=false,At=5},new GeometryCommand{Action=GeometryCommand.Types.Action.AddBreak,Horizontal=false,At=9}]);
            string tsv=string.Join("\n",Enumerable.Repeat(string.Join("\t",Enumerable.Repeat("'001234",4)),8));
            byte[] payload=Encoding.UTF8.GetBytes(tsv);
            var operations=new (string Name,Func<Task<Response>> Run,string? Review)[]{
                ("Fill",()=>owner.FillAsync(owner.Revision,P(1),P(8,4),Request.Types.FillDirection.Down),null),
                ("Format",()=>owner.FormatRegionAsync(owner.Revision,P(1),P(8,4),new CellStyle(Bold:true),32),null),
                ("Move",()=>owner.MoveRegionAsync(owner.Revision,P(1),P(8,4),P(9,9)),null),
                ("Paste",()=>owner.PasteBlockAsync(owner.Revision,P(1),P(8,4),payload,false),null),
                ("Clear Visible",()=>owner.ClearVisibleAsync(owner.Revision,P(1),P(8,4),Request.Types.ClearMode.Contents),"activeVisibleEditDialog"),
                ("Fill Visible",()=>owner.FillVisibleAsync(owner.Revision,P(1),P(8,4),Request.Types.FillDirection.Down),"activeVisibleEditDialog"),
                ("Paste Visible",()=>owner.PasteVisibleAsync(owner.Revision,P(1),P(8,4),payload,false),"activeVisiblePasteDialog")
            };
            foreach(var operation in operations)foreach(string decision in operation.Name.Contains("Paste")?new[]{"cancel","stale","payload","apply"}:new[]{"cancel","stale","apply"}){
                await File.AppendAllTextAsync(Path.Combine(output,"progress.txt"),operation.Name+" / "+decision+" started\n");
                await owner.ClearRegionAsync(owner.Revision,P(1),P(16,16));
                await owner.ApplyAsync(owner.Revision,[new Edit{At=P(1),Text="001234"}]);
                await RefreshAsync();await Wait(()=>sheet.GeometryInputReady&&frameReady,"Operation guard frame");
                sheet.Select(P(4));sheet.Select(P(5,3),true);Check(await sheet.EnsureSectionSelectionAsync(),"Bulk selection resolved");
                await window.Clipboard!.SetTextAsync(tsv); // Virtual platform clipboard only.
                var factory=await BeginSectionCommandAsync();
                using var guard=factory?.Invoke();Check(guard is not null,"Qualified invocation guard");
                string before=owner.Revision;var action=operation.Run();
                await Wait(()=>{if(action.IsCompleted){action.GetAwaiter().GetResult();throw new InvalidOperationException(operation.Name+" skipped expanded confirmation");}return activeSectionConfirmation?.IsVisible==true;},operation.Name+" section confirmation");
                var confirm=activeSectionConfirmation!;
                if(decision=="stale")sheet.Select(P(1));
                if(decision=="payload")await window.Clipboard!.SetTextAsync("changed clipboard");
                confirm.Close(decision!="cancel");
                if(decision=="apply"&&operation.Review is not null){await Wait(()=>(operation.Review=="activeVisibleEditDialog"?activeVisibleEditDialog:activeVisiblePasteDialog)?.IsVisible==true,operation.Name+" exact visible-row review");(operation.Review=="activeVisibleEditDialog"?activeVisibleEditDialog:activeVisiblePasteDialog)!.Close(true);}
                try{await action.WaitAsync(TimeSpan.FromSeconds(15));if(decision!="apply")throw new InvalidOperationException(operation.Name+" accepted an invalid confirmation");}
                catch(OperationCanceledException)when(decision!="apply"){}
                if(decision!="apply")Check(owner.Revision==before,operation.Name+" rejected operation cannot publish");
                Evidence.Add(operation.Name+": "+decision+" expanded confirmation");
            }
            var producer=await owner.CreateFixtureProducerAsync(owner.Revision,"ledger");await owner.RefreshProducerAsync(owner.Revision,producer.ProducerId);
            var bound=await owner.CreateBoundWindowAsync(owner.Revision,"Positional review",producer.ProducerId);owner.SelectWindow(bound.Worksheet);
            await RefreshAsync();await Wait(()=>sheet.GeometryInputReady&&frameReady,"Bound frame ready");
            dialog=await Open();Break(dialog,true,5);await Apply(dialog,"customize positional presentation through the normal editor");
            await owner.RefreshProducerAsync(owner.Revision,producer.ProducerId,1);await RefreshAsync();
            Check(window.FindControl<StackPanel>("GeometryLayoutReviewActions")!.IsVisible,"Reordered source offers persistent layout review");
            var keep=window.GetLogicalDescendants().OfType<Button>().Single(b=>b.Content?.ToString()=="Keep positional settings");keep.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
            await Wait(()=>!window.FindControl<StackPanel>("GeometryLayoutReviewActions")!.IsVisible&&!editing,"Keep positional settings");Evidence.Add("explicitly keep positional presentation after source reorder");
            await owner.RefreshProducerAsync(owner.Revision,producer.ProducerId,2);await RefreshAsync();
            Check(window.FindControl<StackPanel>("GeometryLayoutReviewActions")!.IsVisible,"Source membership change needs review");
            var reset=window.GetLogicalDescendants().OfType<Button>().Single(b=>b.Content?.ToString()=="Reset positional geometry");reset.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
            await Wait(()=>!window.FindControl<StackPanel>("GeometryLayoutReviewActions")!.IsVisible&&!editing,"Reset positional geometry");
            Check(geometryView!.Frame.Geometry.HorizontalStarts.Count==1,"Explicit positional reset removes breaks");Evidence.Add("explicitly reset positional presentation after source membership change");
            var collisionScope=await owner.CreateWindowAsync(owner.Revision,"Filter geometry",3,2);owner.SelectWindow(collisionScope.Worksheet);
            await owner.ApplyAsync(owner.Revision,Enumerable.Range(1,3).Select(row=>new Edit{At=P((uint)row),Boolean=true}));
            GeometryCommand Profile(bool horizontal,uint at,params int[] values){var command=new GeometryCommand{Action=GeometryCommand.Types.Action.SetProfile,Horizontal=horizontal,At=at};for(int i=0;i<values.Length;i++)command.Sizes.Add(new GeometrySizeRun{First=(uint)i+1,Last=(uint)i+1,Size=values[i]*64});return command;}
            await owner.EditGeometryAsync(owner.Revision,[new GeometryCommand{Action=GeometryCommand.Types.Action.AddBreak,Horizontal=true,At=3},new GeometryCommand{Action=GeometryCommand.Types.Action.AddBreak,Horizontal=false,At=2},Profile(true,1,1,1),Profile(true,3,2,1),Profile(false,1,1,2,1),Profile(false,2,2,1,1)]);
            await owner.SetFilterAsync(owner.Revision,new FilterDefinition{First=P(1),Last=P(3,2),Conditions={new FilterCondition{Column=1,Predicates={new FilterPredicate{Operator=0,Boolean=true}}}}});
            await owner.ApplyAsync(owner.Revision,[new Edit{At=P(2),Boolean=false}]);await RefreshAsync();
            await Wait(()=>owner.FilterStatus==2&&sheet.GeometryInputReady,"Data-driven geometry suspension");
            Check(sheet.RowIsShown(2)&&FilterStatusLabel.Text?.Contains("geometry",StringComparison.OrdinalIgnoreCase)==true&&ReapplyFilterButton.IsEnabled,"Current values remain visible with an explicit geometry suspension and recovery control");
            var blocked=owner.Revision;await EditVisibleAsync(Request.Types.ClearMode.Contents,Request.Types.FillDirection.FillUnspecified);Check(owner.Revision==blocked&&activeVisibleEditDialog is null,"Visible-only editing is blocked during geometry suspension");
            await owner.ApplyAsync(owner.Revision,[new Edit{At=P(2),Boolean=true}]);await RefreshAsync();Check(owner.FilterStatus==2,"Correcting data does not silently resume the filter");
            ReapplyFilterButton.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));await Wait(()=>owner.FilterStatus==1&&!editing&&sheet.GeometryInputReady,"Explicit reapply restores valid geometry");
            Evidence.Add("data-driven compaction collision suspends presentation, blocks visible-only editing and requires explicit Reapply");
            var dense=await owner.CreateWindowAsync(owner.Revision,"Dense recovery",65536,1024);owner.SelectWindow(dense.Worksheet);
            await owner.EditGeometryAsync(owner.Revision,[new GeometryCommand{Action=GeometryCommand.Types.Action.SetSizes,Horizontal=true,At=1,First=1,Last=1024,Size=64},new GeometryCommand{Action=GeometryCommand.Types.Action.SetSizes,Horizontal=false,At=1,First=1,Last=65536,Size=64}]);
            uint admittedIdentities=owner.AllocatedCellIdentities;
            await RefreshAsync();
            Check(!frameReady&&geometryView is null&&!sheet.GeometryInputReady&&StatusLabel.Text?.Contains("zoom",StringComparison.OrdinalIgnoreCase)==true,"A dense new scope refuses atomically with a zoom diagnostic and no stale scope frame");
            GeometryZoom800.RaiseEvent(new RoutedEventArgs(MenuItem.ClickEvent));
            await Wait(()=>frameReady&&sheet.GeometryInputReady&&geometryView?.Frame.Worksheet==dense.Worksheet,"Zoom menu recovers a dense scope without a preexisting frame");
            Check(geometryView!.Frame.Geometry.Cells.Count>256&&!geometryView.Frame.Geometry.HasNext&&owner.AllocatedCellIdentities==admittedIdentities,"Recovered multi-page viewport is complete and allocates no Blank cell identities");
            Evidence.Add("recover an initially over-dense scope using the independent zoom menu; no stale or partial frame is interactive");
            sheet.SetGeometryZoom(100);
            Directory.CreateDirectory(output);await File.WriteAllLinesAsync(Path.Combine(output,"editing-sequence.txt"),Evidence);await owner.NewAsync(owner.Revision,discard:true);await RefreshAsync();
        } catch(Exception error){await File.WriteAllTextAsync(Path.Combine(output,"failure-before-cleanup.txt"),error.ToString());throw;}
        finally{if(Clipboard is not null){if(previousText is null)await Clipboard.ClearAsync();else await Clipboard.SetTextAsync(previousText);}}
    }
}
