using Pivot.Desktop;

internal static class ClipboardReadChecks
{
    private sealed class Transfer:IDisposable {
        internal bool Disposed;
        public void Dispose()=>Disposed=true;
    }
    internal static async Task Run()
    {
        static void Check(bool ok,string why) { if(!ok)throw new Exception("Clipboard read: "+why); }
        static Task NoDelay()=>Task.CompletedTask;
        static ClipboardRead.Snapshot Text(string? value,bool advertised=true)=>new(true,false,false,advertised,null,value);
        static ClipboardRead.Snapshot Native(byte[]? value)=>new(true,true,false,false,value,null);
        async Task Reject(params ClipboardRead.Snapshot[] states) {
            int reads=0;
            try {
                await ClipboardRead.AcquireAsync(()=>Task.FromResult(states[Math.Min(reads++,states.Length-1)]),()=>true,delay:NoDelay);
                throw new Exception("Clipboard read rejection expected");
            }catch(InvalidDataException) {}
        }
        int calls=0;
        var payload=await ClipboardRead.AcquireAsync(()=>Task.FromResult(++calls<3 ? Text(null) : Text("001234")),()=>true,delay:NoDelay);
        Check(calls==3&&payload.Text=="001234"&&payload.Bytes is null,"delayed text succeeds exactly once without numeric interpretation");
        calls=0;
        payload=await ClipboardRead.AcquireAsync(()=>Task.FromResult(++calls<2 ? Native(null) : Native([1,2,3])),()=>true,delay:NoDelay);
        Check(calls==2&&payload.Bytes!.SequenceEqual(new byte[]{1,2,3}),"native retry retains native interpretation");
        calls=0;
        payload=await ClipboardRead.AcquireAsync(()=>Task.FromResult(++calls<2 ? default : Text("")),()=>true,delay:NoDelay);
        Check(calls==2&&payload.Text=="","missing snapshot can become a valid empty String");
        await Reject(Native(null),Text("=1+2"));
        await Reject(new ClipboardRead.Snapshot(true,true,false,false,null,null,6),new ClipboardRead.Snapshot(true,true,false,true,[1],"wrong fallback",5));
        await Reject(Text(null),Native([1]));
        await Reject(new ClipboardRead.Snapshot(true,false,true,true,null,"001234"));
        await Reject(Text(null));
        await Reject(Native(null));
        bool current=true;calls=0;
        payload=await ClipboardRead.AcquireAsync(()=>Task.FromResult(++calls==1 ? Text(null) : Text("stale")),()=>current,
            delay:()=>{current=false;return Task.CompletedTask;});
        Check(calls==1&&payload==default,"changed interaction aborts before another read");
        current=true;calls=0;
        payload=await ClipboardRead.AcquireAsync(()=>{calls++;current=false;return Task.FromResult(Text("stale"));},()=>current,delay:NoDelay);
        Check(calls==1&&payload==default,"changed interaction during acquisition discards data");
        calls=0;
        payload=await ClipboardRead.AcquireAsync(()=>{calls++;return Task.FromResult(Text("wrong"));},()=>false,delay:NoDelay);
        Check(calls==0&&payload==default,"already stale request never reads clipboard");
        var owned=new Transfer();int platformReads=0;
        var snapshot=await ClipboardRead.CaptureAsync(()=>Task.FromResult<Transfer?>(owned),
            ()=>{platformReads++;return Task.FromResult<Transfer?>(new());},
            (data,isOwned)=>{Check(ReferenceEquals(data,owned)&&isOwned,"current ownership selects in-process data");return Task.FromResult(Text("001234"));});
        Check(snapshot.Text=="001234"&&!owned.Disposed&&platformReads==0,"owned clipboard data is neither disposed nor replaced by a native read");
        var external=new Transfer();
        snapshot=await ClipboardRead.CaptureAsync(()=>Task.FromResult<Transfer?>(null),
            ()=>{platformReads++;return Task.FromResult<Transfer?>(external);},
            (data,isOwned)=>{Check(ReferenceEquals(data,external)&&!isOwned,"lost ownership selects current native data");return Task.FromResult(Text("current"));});
        Check(snapshot.Text=="current"&&external.Disposed&&platformReads==1,"native read session is disposed after capture");
        external=new Transfer();
        try {
            await ClipboardRead.CaptureAsync(()=>Task.FromResult<Transfer?>(null),()=>Task.FromResult<Transfer?>(external),
                (data,isOwned)=>Task.FromException<ClipboardRead.Snapshot>(new InvalidDataException("provider failed")));
            throw new Exception("Expected provider failure");
        }catch(InvalidDataException) {}
        Check(external.Disposed,"native read session is disposed on failure");
    }
}
