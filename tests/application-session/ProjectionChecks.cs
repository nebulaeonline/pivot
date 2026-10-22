using Pivot.Client;
using Pivot.Protocol.Application.V1;
internal static class ProjectionChecks
{
    private static void Check(bool ok,string message) {if(!ok)throw new Exception("Projection: "+message);}
    private static Position P(uint r,uint c=1)=>new(){Row=r,Column=c};
    private static async Task Reject(Func<Task> action) {
        try {await action();} catch(WorkbookSessionException){return;}
        throw new Exception("Projection operation should be rejected");
    }
    internal static async Task Run(string executable) {
        var s=await WorkbookSession.StartAsync(executable);
        try {
            await s.ApplyAsync(s.Revision,[new Edit{At=P(8),Number=WorkbookSession.Encode(8)},new Edit{At=P(9),Number=WorkbookSession.Encode(9)},new Edit{At=P(9,2),Formula="=SUM(A8:A9)"}]);
            var ordinary=await s.ReadAsync(P(8),P(9,2));
            await s.StartProofAsync();
            var view=(await s.ReadProofAsync(P(1),P(20,10))).ProofView;
            Check(view.RecordCount==3 && view.Placements.Count==2 && view.Placements[0].Id!=view.Placements[1].Id,"two projections share one generation");
            Check(view.Rows.Single(r=>r.Kind==0&&r.OrdinaryRow==9) is not null,"ordinary row 9 remains addressable after bands");
            var old=await s.CaptureAsync();
            await s.ApplyAsync(s.Revision,[new Edit{At=P(8),Number=WorkbookSession.Encode(80)}]);
            await s.PublishProofAsync(5);
            var current=await s.CaptureAsync();await Reject(()=>s.CaptureAsync());
            var first=(await s.EvaluateProofAsync(old.Capture,"L2")).ProofValue;
            var next=(await s.EvaluateProofAsync(current.Capture)).ProofValue;
            Check(WorkbookSession.Decode(first.Total)==60 && WorkbookSession.Decode(first.Taxed)==66,"old detached numeric invocation retains result generation");
            Check(WorkbookSession.Decode(next.Total)==150 && WorkbookSession.Decode(next.Taxed)==165 && next.Generation!=first.Generation,"new coherent capture sees new generation");
            Check(WorkbookSession.Decode((await s.CalculateAsync(old.Capture,[P(9,2)])).Cells.Single().Value)==17,"old workbook root captured with old result root");
            Check(WorkbookSession.Decode((await s.CalculateAsync(current.Capture,[P(9,2)])).Cells.Single().Value)==89,"new workbook root captured with new result root");
            await s.ReleaseAsync(old.Capture);await s.ReleaseAsync(current.Capture);
            string live=s.Revision;
            await s.PublishProofAsync(3,"L2");
            var missing=await s.CaptureAsync();
            Check((await s.EvaluateProofAsync(missing.Capture,"L2")).ProofValue.Error.Length>0,"missing key never resolves by ordinal");
            await s.ReleaseAsync(missing.Capture);
            await s.PublishProofAsync(3);view=(await s.ReadProofAsync(P(1),P(20,10))).ProofView;
            Check(view.RecordKeys.Contains("L2") && s.Revision==live,"key returns without worksheet mutation");
            ulong epoch=s.ProofEpoch,presentation=s.ProofPresentation;
            await s.LayoutProofAsync(view.Placements[0].Id,4,true);
            Check(s.ProofEpoch==epoch && s.ProofPresentation!=presentation,"layout changes presentation only");
            var after=await s.ReadAsync(P(8),P(9,2));
            Check(after.Cells.Single(c=>c.At.Equals(P(9))).CellId==ordinary.Cells.Single(c=>c.At.Equals(P(9))).CellId,"projection does not materialize ordinary identities");
            string path=Path.Combine(Path.GetTempPath(),"pivot-proof-"+Guid.NewGuid()+".pivot");
            await Reject(()=>s.SaveAsync(s.Revision,path));Check(!File.Exists(path),"save refuses before filesystem write");
            await Reject(()=>s.NewAsync(s.Revision));Check(s.IsProjectionProof,"dirty lifecycle rejection preserves proof");
            await s.PublishProofAsync(0);var empty=await s.CaptureAsync();
            Check(WorkbookSession.Decode((await s.EvaluateProofAsync(empty.Capture)).ProofValue.Total)==0,"empty result checked total is zero");
            await s.ReleaseAsync(empty.Capture);
            await s.NewAsync(s.Revision,discard:true);Check(!s.IsProjectionProof&&!s.IsDirty,"explicit discard leaves normal clean workbook");
        } finally {await s.DisposeAsync();}
        Check(s.FinalCharge==0,"all ownership charges released");
    }
}
