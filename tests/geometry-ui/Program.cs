using System.Reflection;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Interactivity;
using Avalonia.Input;
using Avalonia.Input.Platform;
using System.Text;
using Avalonia.Threading;
using Avalonia.VisualTree;
using Avalonia.LogicalTree;
using Pivot.Client;
using Pivot.Desktop;
using Pivot.Avalonia;
using Pivot.Protocol.Application.V1;

internal static class Program
{
    private static readonly BindingFlags Private=BindingFlags.Instance|BindingFlags.NonPublic;
    [STAThread] public static int Main(string[] args)
    {
        if(args.Length!=2){Console.Error.WriteLine("Expected session executable and output directory.");return 2;}
        // This process never initializes a native platform, OS input or system clipboard.
        Environment.SetEnvironmentVariable("PIVOT_SESSION_PATH",Path.GetFullPath(args[0]));
        foreach(var key in new[]{"PIVOT_UI_SMOKE_REPORT","PIVOT_UI_PEER_REPORT","PIVOT_PROJECTION_PROOF","PIVOT_BINDING_PROOF"})Environment.SetEnvironmentVariable(key,null);
        AppBuilder.Configure<App>().UseSkia().UseHeadless(new AvaloniaHeadlessPlatformOptions{UseHeadlessDrawing=false}).WithInterFont().SetupWithoutStarting();
        using var stop=new CancellationTokenSource();int code=1;
        var renderTimer=new DispatcherTimer{Interval=TimeSpan.FromMilliseconds(16)};renderTimer.Tick+=(_,_)=>AvaloniaHeadlessPlatform.ForceRenderTimerTick();renderTimer.Start();
        Dispatcher.UIThread.Post(async()=>{
            try{await Run(args[1]);code=0;}catch(Exception error){Console.Error.WriteLine(error);Directory.CreateDirectory(args[1]);await File.WriteAllTextAsync(Path.Combine(args[1],"failure.txt"),error.ToString());}
            finally{renderTimer.Stop();stop.Cancel();}
        });
        Dispatcher.UIThread.MainLoop(stop.Token);return code;
    }
    private static T Field<T>(object owner,string name)=>(T)(owner.GetType().GetField(name,Private)??throw new InvalidOperationException("Missing field "+name)).GetValue(owner)!;
    private static Task Invoke(object owner,string name,params object?[] args)=>(Task)(owner.GetType().GetMethod(name,Private)?.Invoke(owner,args)??throw new InvalidOperationException("Missing method "+name));
    private static async Task Wait(Func<bool> condition,string message)
    {
        var until=DateTime.UtcNow.AddSeconds(15);
        while(!condition()){if(DateTime.UtcNow>until)throw new TimeoutException(message);AvaloniaHeadlessPlatform.ForceRenderTimerTick();await Task.Delay(10);}
    }
    private static async Task Run(string output)
    {
        Directory.CreateDirectory(output);
        var window=new MainWindow();window.Show();WorkbookSession? owner=null;
        try {
            await Wait(()=>{try{owner=Field<WorkbookSession>(window,"session");return owner.DocumentId.Length>0&&Field<bool>(window,"frameReady");}catch{return false;}},"Initial canonical frame");
            var superseded=Invoke(window,"RefreshAsync");var current=Invoke(window,"RefreshAsync");
            await superseded;
            bool observedGap=!Field<bool>(window,"frameReady");
            await Invoke(window,"AutomationWaitForGeometryFrameAsync");
            if(!Field<bool>(window,"frameReady"))throw new InvalidOperationException("Current painted frame must settle after a superseded refresh completes");
            await current;
            Console.WriteLine("Superseded refresh and latest painted-frame regression passed; unready gap observed="+observedGap+".");
            await FramePublication(window);
            await Invoke(window,"GeometryAutomationAsync",output);
            await Invoke(window,"TextAutomationAsync",Path.Combine(output,"text-report.json"));
            await Invoke(window,"VisibleClipboardAutomationAsync",Path.Combine(output,"visible-report.json"));
            await PendingDrags(window);
            await Invoke(window,"InputAutomationAsync",Path.Combine(output,"input-report.json"));
            Console.WriteLine("Virtual-window geometry, text-paste, visible-clipboard and complete input editing sequences passed.");
        } catch(Exception error){await File.WriteAllTextAsync(Path.Combine(output,"failure-before-cleanup.txt"),error.ToString());throw;}
        finally {
            foreach(string field in new[]{"activeGeometryEditor","activeSectionConfirmation","activeVisibleEditDialog","activeVisiblePasteDialog"}){
                if(Field<Window?>(window,field) is {IsVisible:true} modal){if(field=="activeGeometryEditor")modal.Close();else modal.Close(false);}
            }
            if(owner is not null){await Invoke(window,"ReleaseGeometryViewAsync",false);await owner.DisposeAsync();if(owner.FinalCharge!=0)throw new InvalidOperationException("All view/capture charges released");}
            window.GetType().GetField("allowClose",Private)!.SetValue(window,true);window.Close();
        }
    }
    private static async Task FramePublication(MainWindow window)
    {
        var owner=Field<WorkbookSession>(window,"session");var sheet=window.FindControl<WorksheetView>("Sheet")!;
        Position P(uint r,uint c=1)=>new(){Row=r,Column=c};
        await owner.NewAsync(owner.Revision,discard:true);await Invoke(window,"RefreshAsync");await Invoke(window,"AutomationWaitForGeometryFrameAsync");
        var scope=await owner.CreateWindowAsync(owner.Revision,"Frame publication",3,2);owner.SelectWindow(scope.Worksheet);
        await owner.ApplyAsync(owner.Revision,Enumerable.Range(1,3).Select(r=>new Edit{At=P((uint)r),Boolean=true}));
        GeometryCommand Profile(bool horizontal,uint at,params int[] sizes){var q=new GeometryCommand{Action=GeometryCommand.Types.Action.SetProfile,Horizontal=horizontal,At=at};for(int i=0;i<sizes.Length;i++)q.Sizes.Add(new GeometrySizeRun{First=(uint)i+1,Last=(uint)i+1,Size=sizes[i]*64});return q;}
        await owner.EditGeometryAsync(owner.Revision,[new GeometryCommand{Action=GeometryCommand.Types.Action.AddBreak,Horizontal=true,At=3},new GeometryCommand{Action=GeometryCommand.Types.Action.AddBreak,Horizontal=false,At=2},Profile(true,1,1,1),Profile(true,3,2,1),Profile(false,1,1,2,1),Profile(false,2,2,1,1)]);
        await owner.SetFilterAsync(owner.Revision,new FilterDefinition{First=P(1),Last=P(3,2),Conditions={new FilterCondition{Column=1,Predicates={new FilterPredicate{Operator=0,Boolean=true}}}}});
        await owner.ApplyAsync(owner.Revision,[new Edit{At=P(2),Boolean=false}]);
        var original=sheet.ResolveSectionSelection!;var entered=new TaskCompletionSource();var release=new TaskCompletionSource();bool held=false;
        sheet.ResolveSectionSelection=async(view,first,last)=>{
            var result=await original(view,first,last);
            if(!held&&view.Frame.FilterStatus==2){held=true;entered.SetResult();await release.Task;}
            return result;
        };
        var refresh=Invoke(window,"RefreshAsync");
        try{
            await entered.Task.WaitAsync(TimeSpan.FromSeconds(15));
            bool currentValue=sheet.CellAt(P(2)) is {ContentCase:Cell.ContentOneofCase.Boolean,Boolean:false};
            bool warning=window.FindControl<TextBlock>("FilterStatusLabel")!.Text?.Contains("geometry",StringComparison.OrdinalIgnoreCase)==true;
            bool recovery=window.FindControl<Button>("ReapplyFilterButton")!.IsEnabled;
            if(!currentValue||!sheet.RowIsShown(2)||!warning||!recovery)throw new InvalidOperationException($"Frame publication must pair current values with suspension controls before deferred cleanup; value={currentValue}; shown={sheet.RowIsShown(2)}; warning={warning}; recovery={recovery}");
        }finally{release.TrySetResult();sheet.ResolveSectionSelection=original;await refresh;}
        await Invoke(window,"AutomationWaitForGeometryFrameAsync");
        await owner.NewAsync(owner.Revision,discard:true);await Invoke(window,"RefreshAsync");await Invoke(window,"AutomationWaitForGeometryFrameAsync");
        Console.WriteLine("Current data and filter recovery controls publish together before deferred cleanup.");
    }
    private static async Task PendingDrags(MainWindow window)
    {
        var sheet=window.FindControl<WorksheetView>("Sheet")!;
        var original=sheet.SelectionRequested!;
        Position P(uint r,uint c)=>new(){Row=r,Column=c};
        for(int scenario=0;scenario<4;scenario++){
            await Invoke(window,"SelectAsync",P(2,2),false,null);
            await Wait(()=>Field<bool>(window,"frameReady")&&sheet.GeometryInputReady,"Pending drag initial frame");
            var entered=new TaskCompletionSource();var gate=new TaskCompletionSource();var completed=new TaskCompletionSource();
            sheet.SelectionRequested=async(at,extend,edit,current)=>{
                entered.SetResult();await gate.Task;
                try{return await original(at,extend,edit,current);}finally{completed.SetResult();}
            };
            var mouse=new global::Avalonia.Input.Pointer(871+scenario,PointerType.Mouse,true);
            var held=new PointerPointProperties(global::Avalonia.Input.RawInputModifiers.LeftMouseButton,PointerUpdateKind.LeftButtonPressed);
            var released=new PointerPointProperties(global::Avalonia.Input.RawInputModifiers.None,PointerUpdateKind.LeftButtonReleased);
            Point ToWindow(Position at)=>sheet.TranslatePoint(sheet.CellBounds(at).Center,window)!.Value;
            try{
                sheet.RaiseEvent(new PointerPressedEventArgs(sheet,mouse,window,ToWindow(P(2,2)),0,held,KeyModifiers.None,1));
                await entered.Task;
                sheet.RaiseEvent(new PointerEventArgs(InputElement.PointerMovedEvent,sheet,mouse,window,ToWindow(P(5,4)),0,held,KeyModifiers.None));
                sheet.RaiseEvent(new PointerReleasedEventArgs(sheet,mouse,window,ToWindow(P(5,4)),0,released,KeyModifiers.None,MouseButton.Left));
                if(sheet.IsSelecting)throw new InvalidOperationException("Pending released drag retains no capture");
                if(scenario==1)sheet.RaiseEvent(new KeyEventArgs{RoutedEvent=InputElement.KeyDownEvent,Key=Key.Escape});
                if(scenario==2){
                    sheet.SelectionRequested=original;
                    sheet.RaiseEvent(new PointerPressedEventArgs(sheet,mouse,window,ToWindow(P(3,3)),0,held,KeyModifiers.None,1));
                    sheet.RaiseEvent(new PointerReleasedEventArgs(sheet,mouse,window,ToWindow(P(3,3)),0,released,KeyModifiers.None,MouseButton.Left));
                    await Wait(()=>sheet.Selected.Equals(P(3,3)),"New gesture completes before old request");
                }
                if(scenario==3){
                    var owner=Field<WorkbookSession>(window,"session");
                    await owner.ApplyAsync(owner.Revision,[new Edit{At=P(1,1),Text="background update"}]);
                    await Invoke(window,"RefreshAsync");
                }
                gate.SetResult();await completed.Task;
                await Dispatcher.UIThread.InvokeAsync(()=>{},DispatcherPriority.Background);
                var expected=scenario==0?P(5,4):scenario==2?P(3,3):P(2,2);
                await Wait(()=>sheet.Selected.Equals(expected),"Pending released drag preserves endpoint, cancellation and newer gesture");
                if(!await sheet.EnsureSectionSelectionAsync()||sheet.IsSelecting)throw new InvalidOperationException("Pending drag canonical bounds settle without capture");
            }finally{gate.TrySetResult();sheet.SelectionRequested=original;}
        }
        var session=Field<WorkbookSession>(window,"session");
        await session.ApplyAsync(session.Revision,[new Edit{At=P(1,1),Clear=true}]);await Invoke(window,"RefreshAsync");
        Console.WriteLine("Pending drag release, Escape, superseding gesture and external-frame cancellation regressions passed.");
    }
}
