using Pivot.Client;
using Pivot.Protocol.Application.V1;
internal static class StructuredBindingChecks {
    private static Position P(uint r,uint c=1)=>new(){Row=r,Column=c};
    private static void Check(bool ok,string message){if(!ok)throw new Exception("Worksheet binding: "+message);}
    internal static async Task Run(string executable){
        var s=await WorkbookSession.StartAsync(executable);
        try {
            await s.StartBindingsAsync();Check(s.IsBindingProof,"explicit binding mode");
            await s.ApplyAsync(s.Revision,[new Edit{At=P(2),Number=WorkbookSession.Encode(1)}]);
            const string source="=@{InvoiceLines.TotalAmount}*1.1+A2";
            var resolved=await s.ResolveFormulaAsync(source);
            Check(resolved.Occurrences.Count==1 && resolved.Occurrences[0].StartByte==1,"typed UTF-8 occurrence spans");
            async Task Reject(TypedOccurrence[] entries,string state,bool conflict=false){
                string before=s.Revision;
                try{await s.ApplyResolvedFormulaAsync(before,P(10),source,entries,state);throw new Exception("Invalid annotation unexpectedly accepted");}
                catch(WorkbookSessionException e){Check(!conflict||e.IsConflict,"stale computation token is a conflict");}
                Check(s.Revision==before,"rejected apply leaves root unchanged");
            }
            foreach(string field in new[]{"span","ordinal","document","producer","output","type"}){
                var bad=resolved.Occurrences[0].Clone();
                switch(field){case "span":bad.EndByte++;break;case "ordinal":bad.Ordinal++;break;case "document":bad.Document=Guid.NewGuid().ToString();break;case "producer":bad.Producer=Guid.NewGuid().ToString();break;case "output":bad.Output=Guid.NewGuid().ToString();break;default:bad.ExpectedType=2;break;}
                await Reject([bad],resolved.ComputationalState);
            }
            await Reject([],resolved.ComputationalState);
            await Reject([resolved.Occurrences[0],resolved.Occurrences[0]],resolved.ComputationalState);
            await s.PublishProofAsync(3);
            await Reject(resolved.Occurrences.ToArray(),resolved.ComputationalState,true);
            await s.ApplyStructuredFormulaAsync(s.Revision,P(1),source);
            await s.ApplyAsync(s.Revision,[new Edit{At=P(1,2),Formula="=A1+1"},new Edit{At=P(2,2),Formula="=SUM(A1:B1)"}]);
            var first=await s.CaptureAsync();string revision=s.Revision,token=s.ComputationalState;
            await s.PublishProofAsync(5);Check(s.Revision==revision&&s.ComputationalState!=token,"source publication rotates computational authority only");
            var frame=await s.ReadAsync(P(1),P(2,2));
            Check(WorkbookSession.Decode(frame.Cells.Single(c=>c.At.Equals(P(1))).Value)==166,"ordinary formula consumes new publication");
            Check(WorkbookSession.Decode(frame.Cells.Single(c=>c.At.Equals(P(2,2))).Value)==333,"dependent SUM consumes bound worksheet cells");
            var old=await s.CalculateAsync(first.Capture,[P(1)]);
            Check(WorkbookSession.Decode(old.Cells.Single().Value)==67 && old.CaptureState==token,"old capture retains old source context and computational identity");
            await s.ReleaseAsync(first.Capture);
            await s.RenameProducerAsync("RenamedLines");
            await s.DuplicateBlockAsync(P(1),P(1),P(3),P(3));
            frame=await s.ReadAsync(P(3),P(3));Check(WorkbookSession.Decode(frame.Cells.Single().Value)==165,"duplicate after rename preserves output and translates A2 to A4");
            try {await s.CopyBlockAsync(s.Revision,P(1),P(1));throw new Exception("Structured OS copy should reject");}catch(WorkbookSessionException){}
            await s.PublishUnavailableAsync();frame=await s.ReadAsync(P(1),P(1));Check(frame.Cells.Single().Error.Contains("UnavailableResult"),"unavailable head never falls back to old total");
            await s.PublishProofAsync(0);frame=await s.ReadAsync(P(1),P(1));Check(WorkbookSession.Decode(frame.Cells.Single().Value)==1,"valid empty source is exact zero");
            await s.RenameOutputAsync("RenamedTotal");
            frame=await s.ReadAsync(P(1),P(1));Check(frame.Cells.Single().SourceOutputName=="RenamedTotal","output rename changes label without rebinding");
            await s.RetireOutputAsync();frame=await s.ReadAsync(P(1),P(1));Check(frame.Cells.Single().SourceFaultCode=="MissingOutput","removed output is explicit");
            await s.RetireOutputAsync(recreate:true);frame=await s.ReadAsync(P(1),P(1));Check(frame.Cells.Single().SourceFaultCode=="MissingOutput","same-label recreated output never rebinds old operand");
            await s.RetireProducerAsync();frame=await s.ReadAsync(P(1),P(1));Check(frame.Cells.Single().Error.Contains("MissingProducer"),"deleted identity is explicit");
            await s.NewAsync(s.Revision,discard:true);Check(!s.IsBindingProof,"discard restores normal workbook");
        }finally{await s.DisposeAsync();}
        Check(s.FinalCharge==0,"zero charge");
    }
}
