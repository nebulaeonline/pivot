using Pivot.Client;
using Pivot.Protocol.Application.V1;
internal static class GeometryChecks
{
    private static void Check(bool value,string message){if(!value)throw new InvalidOperationException(message);}
    private static Position P(uint row,uint column=1)=>new(){Row=row,Column=column};
    internal static async Task Run(string executable,string output)
    {
        await using var session=await WorkbookSession.StartAsync(executable);
        await session.NewAsync(session.Revision,discard:true);
        string worksheet=session.WorksheetId;
        var created=await session.CreateWindowAsync(session.Revision,"Invoice",36,8);
        session.SelectWindow(created.Worksheet);
        await session.ApplyAsync(session.Revision,[new Edit{At=P(1),Text="001234"}]);
        var old=await session.CaptureAsync();
        try {
            await session.EditGeometryAsync(session.Revision,[new GeometryCommand{Action=GeometryCommand.Types.Action.AddBreak,Horizontal=true,At=5}]);
            var current=await session.CaptureAsync();
            try {
                await session.ApplyAsync(session.Revision,[new Edit{At=P(1),Text="changed after capture"}]);
                session.SelectWorksheet(worksheet);
                string revision=session.Revision;
                var oldSelection=await session.ResolveSectionSelectionAsync(old.Capture,P(4),P(5,3));
                Check(!oldSelection.Geometry.Enlarged,"Old capture keeps its original geometry");
                Check(session.Revision==revision,"Reading old geometry cannot roll back the current revision");
                var selection=await session.ResolveSectionSelectionAsync(current.Capture,P(4),P(5,3));
                Check(selection.Geometry.Enlarged&&selection.Geometry.Sections==2&&selection.Geometry.EffectiveLast.Equals(P(36,8)),"Crossing one boundary closes both touched sections");
                var page=await session.ReadGeometryAsync(current.Capture,new GeometryRect{Right=4512*8,Bottom=1248*20});
                Check(page.Geometry.Cells.Count==160&&page.Geometry.Total==160&&!page.Geometry.HasNext,"Complete canonical rectangles for a bounded viewport");
                Check(page.Geometry.Cells.Any(c=>c.At.Equals(P(1))&&c.Rectangle.Right==4512),"Exact fixed-point rectangle");
                Check(page.Cells.Single(c=>c.At.Equals(P(1))).Result.Text=="001234","Geometry/value frame preserves identifier Strings");
                Check(page.Capture.Length==0&&page.CaptureIdentity==current.Capture,"Geometry reads borrow their capture");
                using var cancelled=new CancellationTokenSource();
                session.RequestDispatchedForTesting=()=>{session.RequestDispatchedForTesting=null;cancelled.Cancel();};
                try {
                    await session.ReadGeometryAsync(current.Capture,new GeometryRect{Right=4512*8,Bottom=1248*20},cancellation:cancelled.Token);
                    throw new InvalidOperationException("Expected cancelled borrowed geometry read");
                } catch(OperationCanceledException) { }
                finally {session.RequestDispatchedForTesting=null;}
                Check((await session.ResolveSectionSelectionAsync(current.Capture,P(4),P(5,3))).Geometry.Enlarged,"Cancelling a borrowed read never releases its owner's capture");
                uint identities=session.AllocatedCellIdentities;
                await using(var owned=await session.LoadGeometryViewportAsync(P(10,3),new GeometryRect{Right=4512*20,Bottom=1248*20})) {
                    Check(owned.Frame.Geometry.Cells.Count==400&&owned.Frame.Cells.Count==400&&owned.Frame.Styles.Count==400,"Complete multi-page frame set");
                    Check(owned.Frame.Geometry.Viewport.Left==4512*2&&owned.Frame.Geometry.Viewport.Top==1248*9,"Logical anchor uses canonical document origin");
                    Check(!owned.Frame.Geometry.HasNext&&owned.Frame.Cells.All(c=>c.CellId.Length==0),"Complete frame does not manufacture Blank identities");
                    Check(session.AllocatedCellIdentities==identities,"Viewport demand does not allocate cell identities");
                }

            } finally {await session.ReleaseAsync(current.Capture);}
        } finally {await session.ReleaseAsync(old.Capture);}
        await session.DisposeAsync();Check(session.FinalCharge==0,"Geometry capture charges released on close");
        Directory.CreateDirectory(output);
        await File.WriteAllTextAsync(Path.Combine(output,"geometry-session.json"),"{\"status\":\"passed\",\"final_charge\":0}");
    }
}
