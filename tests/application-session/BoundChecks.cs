using Pivot.Client;
using Pivot.Protocol.Application.V1;
using System.Text.Json;
static class BoundChecks {
    static Position P(uint row,uint column=1)=>new(){Row=row,Column=column};
    static Edit E(uint row,uint column,string text)=>new(){At=P(row,column),Entry=text};
    static void Check(bool yes,string text){if(!yes)throw new Exception("Bound producers: "+text);}
    static async Task Fails(Func<Task> action){try{await action();throw new Exception("Expected refusal");}catch(WorkbookSessionException){}}
    static decimal Number(Response r,uint row,uint column=1)=>WorkbookSession.Decode(r.Cells.Single(c=>c.At.Row==row&&c.At.Column==column).Result.Decimal);
    public static async Task Run(string binary,string output){
        var s=await WorkbookSession.StartAsync(binary);
        try {
            string sheet=s.WorksheetId;
            await s.ApplyAsync(s.Revision,[E(2,2,"2"),E(3,2,"5"),E(4,2,"3"),E(2,3,"'0012")]);
            var producer=await s.CreateWorksheetProducerAsync(s.Revision,sheet,P(2,2),P(4,3));string worksheetProducer=producer.ProducerId;
            var window=await s.CreateBoundWindowAsync(s.Revision,"Items",worksheetProducer);string items=window.Worksheet;s.SelectWindow(items);
            Check(s.IsReadOnly&&s.Windows.Single().Bound,"read-only ownership metadata");
            var frame=await s.ReadFilteredAsync(1,3,1,2);
            Check(frame.BoundProjection&&frame.Cells.Count==6&&frame.Cells.All(c=>c.CellId.Length==0&&c.NodeId.Length==0&&c.Projected is not null),"tagged projected locations, no manufactured cells");
            Check(Number(frame,2)==5m&&frame.Cells.Single(c=>c.At.Row==1&&c.At.Column==2).Result.Text=="0012","physical mapping and exact text");
            Check(frame.Cells.Single(c=>c.At.Row==2&&c.At.Column==2).Result.Blank,"physical holes retained");
            string before=s.Revision;await Fails(()=>s.ApplyAsync(before,[E(1,1,"7")]));await Fails(()=>s.SetFilterAsync(s.Revision,new FilterDefinition{First=P(1),Last=P(3)}));Check(s.Revision==before,"read-only refusal rollback");
            var values=await s.CopyValuesAsync(s.Revision,P(1,1),P(1,2));Check(values.Clipboard.Length==0&&values.ClipboardText=="2\t'0012","explicit values export preserves identifiers");await Fails(()=>s.CopyBlockAsync(s.Revision,P(1),P(1)));
            var old=await s.CaptureAsync();s.SelectWorksheet(sheet);
            await s.ApplyAsync(s.Revision,[E(10,1,"=Items|A2"),E(11,1,"=SUM(Items|A1:A3)"),E(12,1,"=ALLOCATEWEIGHTEDRANGE(10,2,2,Items|A1:A3)")]);
            Check(Number(await s.ReadAsync(P(10),P(12)),10)==5m,"canonical scalar alias");
            before=s.Revision;await Fails(()=>s.ApplyAsync(before,[E(2,2,"=A10")]));Check(s.Revision==before,"producer-aware feedback refusal");
            await s.StructureAsync(s.Revision,false,false,3,1);
            var after=await s.ReadAsync(P(11),P(13));Check(Number(after,11)==5m&&Number(after,12)==10m&&after.Cells.Single(c=>c.At.Row==13).Error.Length!=0,"structural affinity and strict blank weight");
            var oldValues=await s.CalculateAsync(old.Capture,[P(2)]);Check(Number(oldValues,2)==5m,"old projected capture retains original geometry");await s.ReleaseAsync(old.Capture);
            var external=await s.CreateFixtureProducerAsync(s.Revision,"ledger");string externalProducer=external.ProducerId;
            var externalWindow=await s.CreateBoundWindowAsync(s.Revision,"External",externalProducer);string externalId=externalWindow.Worksheet;s.SelectWindow(externalId);
            var unavailable=await s.ReadAsync(P(1),P(1));Check(unavailable.BoundRows==0&&unavailable.Cells.Count==0,"no implicit first refresh");
            await s.RefreshProducerAsync(s.Revision,externalProducer);Check(Number(await s.ReadAsync(P(1),P(1,2)),1)==1m,"explicit fixture refresh");
            var first=await s.BeginProducerRefreshAsync(s.Revision,externalProducer);var second=await s.BeginProducerRefreshAsync(s.Revision,externalProducer);
            before=s.Revision;await Fails(()=>s.CompleteProducerRefreshAsync(before,externalProducer,first.ProducerTicket,2));Check(s.Revision==before,"obsolete completion cannot change newer status");
            s.SelectWorksheet(sheet);await s.ApplyAsync(s.Revision,[E(20,1,"=External|A1")]);await s.CompleteProducerRefreshAsync(s.Revision,externalProducer,second.ProducerTicket,1);Check(Number(await s.ReadAsync(P(20),P(20)),20)==2m,"durable key follows reorder across unrelated edit");
            var failure=await s.BeginProducerRefreshAsync(s.Revision,externalProducer);string computation=s.ComputationalState;await s.FailProducerRefreshAsync(s.Revision,externalProducer,failure.ProducerTicket);Check(s.ComputationalState==computation&&Number(await s.ReadAsync(P(20),P(20)),20)==2m,"failed status retains selected successful values without calculation revision");
            s.SelectWindow(externalId);before=s.Revision;await Fails(()=>s.ApplyAsync(before,[E(1,1,"9")]));
            var detached=await s.DetachBoundWindowAsync(s.Revision,externalId,"Detached");s.SelectWindow(detached.Worksheet);Check(!s.IsReadOnly&&Number(await s.ReadAsync(P(128),P(128)),128)==2m,"detach creates independent values without rebinding source references");
            string path=Path.Combine(output,"bound-producers.pivot");await s.SaveAsync(s.Revision,path);Check((await File.ReadAllBytesAsync(path)).AsSpan(0,9).SequenceEqual("PIVOTWB15"u8),"WB15 envelope");
            await s.NewAsync(s.Revision,discard:true);await s.OpenAsync(s.Revision,path);Check(s.Producers.Single(p=>p.Id==externalProducer).RefreshStatus=="Disconnected","reopen has no refresh capability");
            s.SelectWorksheet(sheet);Check(Number(await s.ReadAsync(P(20),P(20)),20)==2m,"saved selected context remains calculable");
            var preview=await s.PrepareWindowDeleteAsync(s.Revision,externalId);Check(preview.WindowInboundReferences==1,"exact bound Window inbound review");await s.DeleteWindowAsync(s.Revision,externalId,preview.WindowDeleteTicket);
            Check((await s.ReadAsync(P(20),P(20))).Cells.Single().ErrorCode=="ReferenceDeleted","Window lifetime guard survives source values");
            var replacement=await s.CreateBoundWindowAsync(s.Revision,"External",externalProducer);Check(replacement.Worksheet!=externalId,"Window identity never reused");
            s.SelectWorksheet(sheet);Check((await s.ReadAsync(P(20),P(20))).Cells.Single().ErrorCode=="ReferenceDeleted","name reuse cannot repair guarded reference");
            await s.DisposeAsync();Check(s.FinalCharge==0,"all producer/capture/task charges released");
            await File.WriteAllTextAsync(Path.Combine(output,"bound.json"),JsonSerializer.Serialize(new{status="passed",remaining_charge=s.FinalCharge}));
        } finally {await s.DisposeAsync();}
    }
}
