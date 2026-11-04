using System;
using System.Linq;
using System.Security.Cryptography;
using System.Threading;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Layout;
using Avalonia.Threading;
using Google.Protobuf;
using Pivot.Client;
using Pivot.Protocol.Application.V1;

namespace Pivot.Desktop;
public partial class MainWindow
{
    private GeometryViewport? geometryView;
    private Window? activeSectionConfirmation;
    private void ConfigureGeometrySession(WorkbookSession owner)
    {
        Sheet.GeometryResizeRequested=(command,description)=>EditGeometryAsync(command,description);
        Sheet.ResolveSectionSelection=async(view,first,last)=>{
            if(owner!=session||view.IsDisposed)throw new OperationCanceledException("The view was replaced.");
            return (await owner.ResolveSectionSelectionAsync(view.Capture,first,last)).Geometry;
        };
    }
    private async Task ReleaseGeometryViewAsync(bool loading=false)
    {
        var old=geometryView;geometryView=null;ClearInteriorHeadings();GeometryLayoutBar.IsVisible=false;if(Sheet.GeometryMode)Sheet.SetGeometryViewport(null);if(loading)Sheet.EnterGeometryMode();if(old is not null)await old.DisposeAsync();
    }
    private async Task RefreshGeometryAsync(bool keepReady=false)
    {
        var owner=session;if(owner is null||shuttingDown)return;
        if(geometryView is { } previousScope&&(previousScope.DocumentId!=owner.DocumentId||previousScope.DocumentGeneration!=owner.DocumentGeneration||previousScope.Frame.Worksheet!=owner.WorksheetId||previousScope.Frame.WindowScope!=owner.IsWindowScope))await ReleaseGeometryViewAsync(true);
        Sheet.EnterGeometryMode();
        bool preserve=keepReady&&geometryView is {IsDisposed:false} old&&old.Frame.Revision==owner.Revision&&old.Frame.Worksheet==owner.WorksheetId;
        if(!preserve){frameReady=false;Sheet.GeometryInputReady=false;}
        Sheet.GeometryHeaderInputReady=false;
        refresh?.Cancel();refresh?.Dispose();refresh=new CancellationTokenSource();var cancellation=refresh.Token;long request=++displayRequest;
        EnsureWorksheetViews();string document=owner.DocumentId,scope=owner.WorksheetId;ulong generation=owner.DocumentGeneration;bool window=owner.IsWindowScope;long epoch=worksheetViewEpoch;
        var origin=new Position{Row=Sheet.FirstRow,Column=Sheet.FirstColumn};var active=Sheet.Selected;int zoom=Sheet.GeometryZoomPercent;var viewport=Sheet.RequestedGeometryViewport();
        GeometryViewport? incoming=null;
        try {
            await Task.Delay(35,cancellation);
            incoming=await owner.LoadGeometryViewportAsync(origin,viewport,cancellation,active);
            var frame=incoming.Frame;
            if(owner!=session||shuttingDown||request!=displayRequest||epoch!=worksheetViewEpoch||document!=owner.DocumentId||generation!=owner.DocumentGeneration
                ||scope!=owner.WorksheetId||window!=owner.IsWindowScope||frame.Worksheet!=scope||frame.WindowScope!=window||frame.Revision!=owner.Revision||zoom!=Sheet.GeometryZoomPercent)return;
            if(draftCell is not null&&!editing&&frame.FilterVisibleMask.Length==8192&&(frame.FilterVisibleMask.Span[(int)(draftCell.Row-1)/8]&(1<<((int)(draftCell.Row-1)%8)))==0){filterPublicationDeferred=true;filterViewPending=true;FilterStatusLabel.Text="Finish or cancel the editor before filter visibility changes.";return;}
            if(displayedFilterGeneration!=frame.FilterGeneration){++interactionVersion;++clipboardRequest;displayedFilterGeneration=frame.FilterGeneration;}
            var previous=geometryView;geometryView=incoming;incoming=null;
            Sheet.SetProofFrame(null);Sheet.SetGridDimensions(Math.Max(1,frame.Geometry.Rows),Math.Max(1,frame.Geometry.Columns));
            // A pointer selection may be waiting for its own inline edit to commit.
            // Keep that gesture only across this edit's same-scope/filter frame.
            Sheet.SetGeometryViewport(geometryView,zoom,preservePendingSelection:editing&&draftCell is not null);Sheet.GeometryHeaderInputReady=true;
            frameReady=true;displayedRevision=frame.Revision;displayedGeneration=frame.Generation;
            // Publish frame-dependent controls in the same UI turn as the cells.
            // Cleanup and section resolution can yield or be superseded.
            ShowFilterStatus(frame);ShowGeometryLayout(frame);PublishInteriorHeadings();
            if(previous is not null)await previous.DisposeAsync();
            await Sheet.EnsureSectionSelectionAsync();
            if(request!=displayRequest||owner!=session||geometryView?.Frame!=frame)return;
            UpdateWorksheetTabs();SelectCell();UpdateDocument();UpdateBoundControls();
            StatusLabel.Text=$"Ready · {frame.Geometry.Cells.Count} mapped cells · Zoom {zoom}%";
            if(Sheet.SelectionNeedsReview)StatusLabel.Text="Section bounds changed. Select the intended range again before changing contents.";
            if(frame.FilterStatus==3)_=RefreshPendingFilterAsync(owner,scope,generation,request);
            await Sheet.PaintCompletion.WaitAsync(TimeSpan.FromSeconds(5),cancellation);
        } catch(OperationCanceledException) { }
          catch(Exception error){if(request==displayRequest&&!shuttingDown){StatusLabel.Text=error.Message;Sheet.GeometryInputReady=preserve;}}
        finally {if(incoming is not null)await incoming.DisposeAsync();}
    }
    private static string OperationDigest(Request operation)
    {
        var canonical=operation.Clone();canonical.Id=0;canonical.Version=0;canonical.FilterConfirmation="";
        return Convert.ToHexString(SHA256.HashData(canonical.ToByteArray()));
    }
    private async Task<Func<IDisposable>?> BeginSectionCommandAsync()
    {
        var owner=session;if(owner is null)return null;
        if(!Sheet.GeometryMode)return ()=>owner.BeginSelectionOperation((_,_)=>Task.FromResult(true),(_,_)=>Task.FromResult(true));
        string requestedScope=owner.WorksheetId,requestedDocument=owner.DocumentId;ulong requestedGeneration=owner.DocumentGeneration;
        var originalFirst=Sheet.RequestedFirst;var originalLast=Sheet.RequestedLast;
        if(!await Sheet.EnsureSectionSelectionAsync()){StatusLabel.Text=Sheet.SelectionNeedsReview?"Section bounds changed; select the intended range again.":"Wait for a qualified section selection. "+Sheet.SectionSelectionError;return null;}
        if(Sheet.NeedsGeometryHeaders){await RefreshGeometryAsync();if(!await Sheet.EnsureSectionSelectionAsync())return null;}
        if(owner!=session||owner.WorksheetId!=requestedScope||owner.DocumentId!=requestedDocument||owner.DocumentGeneration!=requestedGeneration||!Sheet.RequestedFirst.Equals(originalFirst)||!Sheet.RequestedLast.Equals(originalLast)){StatusLabel.Text="Selection context changed while preparing the section operation.";return null;}
        var view=geometryView;if(view is null||view.IsDisposed||view.Frame.Revision!=owner.Revision)return null;
        refresh?.Cancel();++displayRequest;
        string revision=owner.Revision,scope=owner.WorksheetId,document=owner.DocumentId,visibility=displayedFilterGeneration;
        bool window=owner.IsWindowScope;ulong generation=owner.DocumentGeneration;
        var requestedFirst=Sheet.RequestedFirst;var requestedLast=Sheet.RequestedLast;var first=Sheet.SelectionFirst;var last=Sheet.SelectionLast;
        bool enlarged=Sheet.SectionSelectionEnlarged;string? digest=null;
        bool Current()=>session==owner&&!shuttingDown&&geometryView==view&&!view.IsDisposed&&owner.Revision==revision&&owner.WorksheetId==scope&&owner.IsWindowScope==window
            &&owner.DocumentId==document&&owner.DocumentGeneration==generation&&displayedFilterGeneration==visibility
            &&Sheet.RequestedFirst.Equals(requestedFirst)&&Sheet.RequestedLast.Equals(requestedLast)&&Sheet.SelectionFirst.Equals(first)&&Sheet.SelectionLast.Equals(last)&&!Sheet.SelectionNeedsReview;
        // Activate in the caller's execution context, after this async method returns.
        return ()=>owner.BeginSelectionOperation((operation,cancel)=>OnUiAsync(async()=>{
            cancel.ThrowIfCancellationRequested();if(!Current()||operation.Revision!=revision||operation.Worksheet!=scope||operation.WindowScope!=window)return false;
            digest=OperationDigest(operation);
            if(operation.Operation is Request.Types.Operation.PasteBlock or Request.Types.Operation.PasteVisible){
                var shape=(await owner.ResolvePasteSectionsAsync(view.Capture,operation,cancel)).Geometry;
                if(!Current())return false;
                bool exact=operation.First.Equals(first)&&operation.Last.Equals(last)&&!first.Equals(last);
                if(shape.Sections>1&&(!exact||(operation.Operation==Request.Types.Operation.PasteBlock&&(!shape.RequestedFirst.Equals(first)||!shape.RequestedLast.Equals(last)||shape.Enlarged)))){
                    StatusLabel.Text="Paste crosses section boundaries. Select the complete destination sections with an exactly matching payload.";return false;
                }
            }
            bool changing=operation.Operation is not (Request.Types.Operation.CopyBlock or Request.Types.Operation.CopyValues or Request.Types.Operation.CopyScope or Request.Types.Operation.CopyVisible);
            if(!changing||!enlarged||operation.WorksheetDefaults)return true;
            ulong Count(Position a,Position b)=>(ulong)(b.Row-a.Row+1)*(b.Column-a.Column+1);
            string action=operation.Operation.ToString();bool visible=operation.Operation is Request.Types.Operation.ClearVisible or Request.Types.Operation.FillVisible or Request.Types.Operation.PasteVisible;
            var panel=new StackPanel{Margin=new Thickness(20),Spacing=12};
            panel.Children.Add(new TextBlock{Text=$"{action}\nRequested: {SortDialog.RangeText(requestedFirst,requestedLast)}\nComplete sections: {SortDialog.RangeText(first,last)}\nAdded positions: {Count(first,last)-Count(requestedFirst,requestedLast):N0}\nHidden-row policy: "+(visible?"visible rows only; hidden rows retained":"all selected rows; existing hidden-row review also applies"),TextWrapping=global::Avalonia.Media.TextWrapping.Wrap});
            if(operation.Operation==Request.Types.Operation.MoveRegion)panel.Children.Add(new TextBlock{Text="Destination starts at "+Pivot.Avalonia.WorksheetView.Address(operation.DestinationFirst)});
            var dialog=new Window{Title="Confirm expanded section operation",Width=620,SizeToContent=SizeToContent.Height,WindowStartupLocation=WindowStartupLocation.CenterOwner};activeSectionConfirmation=dialog;
            var buttons=new StackPanel{Orientation=Orientation.Horizontal,Spacing=12};var cancelButton=new Button{Content="Cancel",IsCancel=true};var apply=new Button{Content="Apply to these sections"};
            cancelButton.Click+=(_,_)=>dialog.Close(false);apply.Click+=(_,_)=>dialog.Close(true);buttons.Children.Add(cancelButton);buttons.Children.Add(apply);panel.Children.Add(buttons);dialog.Content=panel;
            try{
                using var registration=cancel.Register(()=>Dispatcher.UIThread.Post(()=>{if(dialog.IsVisible)dialog.Close(false);}));
                if(!await dialog.ShowDialog<bool>(this)||!Current())return false;
                // Reject changed clipboard bytes before opening any subsequent visible-row review.
                if((operation.Operation is Request.Types.Operation.PasteBlock or Request.Types.Operation.PasteVisible)&&owner.RevalidateFilteredOperationAsync is { } validate)
                    return await validate(operation,cancel)&&Current();
                return true;
            }
            finally{activeSectionConfirmation=null;}
        }),(operation,cancel)=>OnUiAsync(async()=>{
            cancel.ThrowIfCancellationRequested();if(!Current()||digest!=OperationDigest(operation))return false;
            if((operation.Operation is Request.Types.Operation.PasteBlock or Request.Types.Operation.PasteVisible)&&owner.RevalidateFilteredOperationAsync is { } validate)
                return await validate(operation,cancel)&&Current();
            return true;
        }));
    }
    private async Task ClearSelectionAsync()
    {
        if(session is null||session.IsReadOnly||Sheet.ProjectedSelection is not null||!await CommitAsync())return;
        using var context=(await BeginSectionCommandAsync())?.Invoke();if(context is null)return;
        if(!Sheet.GeometryMode){await EditAsync(new Edit{At=Sheet.Selected,Clear=true});return;}
        editing=true;SetEditing(false);
        try{await session.ClearRegionAsync(displayedRevision,Sheet.SelectionFirst,Sheet.SelectionLast);await RefreshAsync();}
        catch(OperationCanceledException){}catch(Exception error){StatusLabel.Text=error.Message;}
        finally{editing=false;SetEditing(session.DocumentId.Length!=0);}
    }
    private void ShowGeometryLayout(Response frame)
    {
        bool positional=frame.Geometry?.Positional==true;
        GeometryLayoutBar.IsVisible=positional;
        GeometryLayoutReviewActions.IsVisible=positional&&frame.Geometry!.LayoutNeedsReview;
        GeometryLayoutStatus.Text=positional?(frame.Geometry!.LayoutNeedsReview
            ?"Positional presentation — source schema or record order changed. Review, keep, or reset these displayed-position settings. Source updates remain live."
            :"Positional presentation — sizes belong to displayed row and column positions, independently of source-field formatting."):"";
    }
    private async void KeepGeometryLayoutClick(object? sender,global::Avalonia.Interactivity.RoutedEventArgs args)=>await AcknowledgeGeometryLayoutAsync(false);
    private async void ResetGeometryLayoutClick(object? sender,global::Avalonia.Interactivity.RoutedEventArgs args)=>await AcknowledgeGeometryLayoutAsync(true);
    private async Task AcknowledgeGeometryLayoutAsync(bool reset)
    {
        var owner=session;var view=geometryView;
        if(owner is null||view is null||view.IsDisposed||editing||fileAction||shuttingDown||!view.Frame.Geometry.Positional||view.Frame.Revision!=owner.Revision||!await CommitAsync())return;
        var stamp=view.Frame.Geometry.LayoutStamp;
        if(stamp.Length!=32)return;
        var commands=new System.Collections.Generic.List<GeometryCommand>();
        if(reset)commands.Add(new GeometryCommand{Action=GeometryCommand.Types.Action.ResetAll});
        commands.Add(new GeometryCommand{Action=GeometryCommand.Types.Action.AcknowledgeLayout,LayoutStamp=stamp});
        editing=true;SetEditing(false);
        try{await owner.EditGeometryAsync(view.Frame.Revision,commands);await RefreshAsync();}
        catch(Exception error){StatusLabel.Text=error.Message;}
        finally{editing=false;SetEditing(owner.DocumentId.Length!=0);}
    }

    private async void WholeScopeRowsClick(object? sender,global::Avalonia.Interactivity.RoutedEventArgs args)=>await SelectWholeScopeAxisAsync(false);
    private async void WholeScopeColumnsClick(object? sender,global::Avalonia.Interactivity.RoutedEventArgs args)=>await SelectWholeScopeAxisAsync(true);
    private async Task SelectWholeScopeAxisAsync(bool columns)
    {
        var owner=session;
        if(owner is null||!Sheet.GeometryMode||editing||fileAction||shuttingDown||!Sheet.HasVisibleRows||!await CommitAsync())return;
        string scope=owner.WorksheetId,revision=owner.Revision,document=owner.DocumentId;ulong generation=owner.DocumentGeneration;
        var requestedFirst=Sheet.RequestedFirst;var requestedLast=Sheet.RequestedLast;var focus=Sheet.Selected;
        if(!frameReady||!Sheet.GeometryInputReady||Sheet.NeedsGeometryHeaders)await RefreshGeometryAsync();
        // A layout change can coalesce a newer viewport while the first one completes.
        // Wait for that read only; never repeat a selection edit or adopt another context.
        var waiting=System.Diagnostics.Stopwatch.StartNew();
        while((!frameReady||!Sheet.GeometryInputReady)&&owner==session&&owner.Revision==revision&&owner.WorksheetId==scope
            &&Sheet.RequestedFirst.Equals(requestedFirst)&&Sheet.RequestedLast.Equals(requestedLast)&&waiting.Elapsed<TimeSpan.FromSeconds(1))await Task.Delay(10);
        if(owner!=session||owner.Revision!=revision||owner.WorksheetId!=scope||owner.DocumentId!=document||owner.DocumentGeneration!=generation||!frameReady||!Sheet.GeometryInputReady
            ||!Sheet.RequestedFirst.Equals(requestedFirst)||!Sheet.RequestedLast.Equals(requestedLast)){StatusLabel.Text="Selection changed while refreshing the section map. Select the intended axes again.";return;}
        var first=columns?new Position{Row=1,Column=requestedFirst.Column}:new Position{Row=requestedFirst.Row,Column=1};
        var last=columns?new Position{Row=(uint)Sheet.RowLimit,Column=requestedLast.Column}:new Position{Row=requestedLast.Row,Column=(uint)Sheet.ColumnLimit};
        Sheet.SelectStructuralBand(first,last,focus);
        await Sheet.EnsureSectionSelectionAsync();SelectCell();
    }

    private void GeometryZoomClick(object? sender,global::Avalonia.Interactivity.RoutedEventArgs args)
    {
        if(session is null||session.DocumentId.Length==0||session.IsBindingProof||session.IsProjectionProof||editing||fileAction||shuttingDown)return;
        if(sender is MenuItem item&&int.TryParse(item.Tag?.ToString(),out int zoom))Sheet.SetGeometryZoom(zoom);
    }

}
