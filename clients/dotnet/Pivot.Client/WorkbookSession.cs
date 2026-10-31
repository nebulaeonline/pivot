using System.Buffers.Binary;
using System.Diagnostics;
using Google.Protobuf;
using Pivot.Protocol.Application.V1;
using Pivot.Protocol.V1;

namespace Pivot.Client;

public sealed class WorkbookSessionException(string message, bool conflict = false) : Exception(message)
{
    public bool IsConflict { get; } = conflict;
}

/// <summary>One owned Rust child, one workbook, at most eight outstanding callers.
/// Cancellation before dispatch prevents execution. After dispatch we drain the reply,
/// update known committed state, then discard the cancelled caller's result. No replay.</summary>
public sealed partial class WorkbookSession : IAsyncDisposable
{
    public const int MaximumFrame = 1_048_576;
    private readonly Process process;
    private readonly SemaphoreSlim gate = new(1);
    private int outstanding;
    private ulong sequence;
    private volatile bool closed;
    private int closing;
    private Task? disposal;
    private string revision = "";
    public string DocumentId { get; private set; } = "";
    public uint PositionedCells {get;private set;}
    public uint AllocatedCellIdentities {get;private set;}
    public uint PositionedCellLimit {get;private set;}
    public uint AllocatedCellIdentityLimit {get;private set;}
    public ulong DocumentGeneration {get;private set;}
    public string WorksheetId { get; private set; } = "";
    public IReadOnlyList<WorksheetInfo> Worksheets { get; private set; } = Array.Empty<WorksheetInfo>();
    private readonly System.Collections.Concurrent.ConcurrentDictionary<string,string> capturedSheets = new();
    public void SelectWorksheet(string id) {
        if (!Worksheets.Any(s=>s.Id==id)) throw new ArgumentException("Unknown worksheet ID.",nameof(id));
        WorksheetId=id;
    }
    public Task<Response> ListSheetsAsync(CancellationToken cancellation=default)=>CallAsync(new Request{Operation=Request.Types.Operation.ListSheets},cancellation);
    public Task<Response> AddSheetAsync(string expected,string? name=null,CancellationToken cancellation=default) {
        ArgumentException.ThrowIfNullOrEmpty(expected);
        return CallAsync(new Request{Operation=Request.Types.Operation.AddSheet,Revision=expected,SheetName=name??""},cancellation);
    }
    public Task<Response> RenameSheetAsync(string expected,string id,string name,CancellationToken cancellation=default) {
        ArgumentException.ThrowIfNullOrEmpty(expected);ArgumentException.ThrowIfNullOrEmpty(id);
        return CallAsync(new Request{Operation=Request.Types.Operation.RenameSheet,Revision=expected,Worksheet=id,SheetName=name},cancellation);
    }

    public string DocumentPath { get; private set; } = "";
    public string SavedCheckpoint { get; private set; } = "";
    public bool IsDirty { get; private set; }
    public bool InstallationUncertain { get; private set; }
    public bool IsProjectionProof { get; private set; }
    public bool IsBindingProof {get;private set;}
    public string ComputationalState {get;private set;}="";
    public ulong ProofEpoch { get; private set; }
    public ulong ProofPresentation { get; private set; }
    public string Revision => Volatile.Read(ref revision);
    public TimeSpan LastRoundTrip { get; private set; }
    public TimeSpan LastClientSerialization { get; private set; }
    public TimeSpan LastClientDeserialization { get; private set; }
    public ulong LastRustExecutionNs { get; private set; }
    public ulong LastRustSerializationNs { get; private set; }
    public ulong LastCompletedRequestId { get; private set; }
    public int ProcessId { get; }
    public ulong? FinalCharge { get; private set; }
    // Test-only synchronization after a real request is flushed, before its
    // response is drained. Keeps
    // cancellation assertions independent of host/pipe scheduling speed.
    internal Action? RequestDispatchedForTesting { get; set; }
    private WorkbookSession(Process child) { process = child; ProcessId = child.Id; }

    public static async Task<WorkbookSession> StartAsync(string executable, CancellationToken cancellation = default)
    {
        cancellation.ThrowIfCancellationRequested();
        var child = Process.Start(new ProcessStartInfo(Path.GetFullPath(executable)) {
            UseShellExecute = false, RedirectStandardInput = true, RedirectStandardOutput = true,
            CreateNoWindow = true // stderr inherited: no unbounded captured diagnostic buffer.
        }) ?? throw new IOException("Could not start Pivot Rust session.");
        var session = new WorkbookSession(child);
        try { await session.CallAsync(new Request { Operation = Request.Types.Operation.Create }, cancellation); return session; }
        catch { await session.DisposeAsync(); throw; }
    }

    public Task<Response> ApplyAsync(string expected, IEnumerable<Edit> edits, CancellationToken cancellation = default)
    {
        ArgumentException.ThrowIfNullOrEmpty(expected);
        // Bound enumeration before constructing caller-owned request buffers.
        var batch = edits.Take(257).Select(e => e.Clone()).ToArray();
        if (batch.Length is 0 or > 256) throw new ArgumentOutOfRangeException(nameof(edits));
        var request = new Request { Operation = Request.Types.Operation.Apply, Revision = expected, ComputationalState=ComputationalState };
        request.Edits.AddRange(batch);
        return CallAsync(request, cancellation);
    }
    public Task<Response> CopyBlockAsync(string expected, Position first, Position last, CancellationToken cancellation = default)
    {
        ArgumentException.ThrowIfNullOrEmpty(expected);
        return CallAsync(new Request {Operation=Request.Types.Operation.CopyBlock,Revision=expected,First=first.Clone(),Last=last.Clone()},cancellation);
    }
    public Task<Response> PasteBlockAsync(string expected, Position first, Position last, byte[] payload, bool native, CancellationToken cancellation = default)
    {
        ArgumentException.ThrowIfNullOrEmpty(expected);
        if(payload.Length>262144) throw new WorkbookSessionException("Clipboard payload exceeds 256 KiB.");
        return CallAsync(new Request {Operation=Request.Types.Operation.PasteBlock,Revision=expected,First=first.Clone(),Last=last.Clone(),Clipboard=ByteString.CopyFrom(payload),NativeClipboard=native},cancellation);
    }
    public Task<Response> StartBindingsAsync(CancellationToken cancellation=default)=>CallAsync(new Request{ComputationalState=ComputationalState,Revision=Revision,Operation=Request.Types.Operation.BindingStart},cancellation);
    public Task<Response> ResolveFormulaAsync(string source,CancellationToken cancellation=default)=>CallAsync(new Request{ComputationalState=ComputationalState,Revision=Revision,Operation=Request.Types.Operation.ResolveFormula,FormulaSource=source},cancellation);
    public async Task<Response> ApplyStructuredFormulaAsync(string expected,Position at,string source,CancellationToken cancellation=default){
        Response resolved;
        try {resolved=await ResolveFormulaAsync(source,cancellation);}
        catch(WorkbookSessionException e) when(!e.IsConflict && (e.Message.Contains("InvalidDraft")||e.Message.Contains("InvalidTarget"))) {
            return await ApplyAsync(expected,[new Edit{At=at.Clone(),Formula=source}],cancellation);
        }
        var request=new Request{Operation=Request.Types.Operation.ApplyFormula,Revision=expected,First=at.Clone(),FormulaSource=source,ComputationalState=resolved.ComputationalState};
        request.Occurrences.AddRange(resolved.Occurrences);return await CallAsync(request,cancellation);
    }
    public Task<Response> ApplyResolvedFormulaAsync(string expected,Position at,string source,IEnumerable<TypedOccurrence> occurrences,string computationalState,CancellationToken cancellation=default){
        ArgumentException.ThrowIfNullOrEmpty(expected);ArgumentException.ThrowIfNullOrEmpty(computationalState);
        var entries=occurrences.Take(17).Select(o=>o.Clone()).ToArray();if(entries.Length>16)throw new ArgumentOutOfRangeException(nameof(occurrences));
        var request=new Request{Operation=Request.Types.Operation.ApplyFormula,Revision=expected,First=at.Clone(),FormulaSource=source,ComputationalState=computationalState};request.Occurrences.AddRange(entries);return CallAsync(request,cancellation);
    }
    public Task<Response> FillAsync(string expected, Position first, Position last, Request.Types.FillDirection direction, CancellationToken cancellation=default) =>
        CallAsync(new Request{Operation=Request.Types.Operation.Fill,Revision=expected,ComputationalState=ComputationalState,First=first.Clone(),Last=last.Clone(),FillDirection=direction},cancellation);
    public Task<Response> DuplicateBlockAsync(Position first,Position last,Position destinationFirst,Position destinationLast,CancellationToken cancellation=default)=>CallAsync(new Request{ComputationalState=ComputationalState,Revision=Revision,Operation=Request.Types.Operation.DuplicateBlock,First=first.Clone(),Last=last.Clone(),DestinationFirst=destinationFirst.Clone(),DestinationLast=destinationLast.Clone()},cancellation);
    public Task<Response> RenameProducerAsync(string name,CancellationToken cancellation=default)=>CallAsync(new Request{ComputationalState=ComputationalState,Revision=Revision,Operation=Request.Types.Operation.RenameProducer,ProducerName=name},cancellation);
    public Task<Response> RenameOutputAsync(string name,CancellationToken cancellation=default)=>CallAsync(new Request{ComputationalState=ComputationalState,Revision=Revision,Operation=Request.Types.Operation.RenameOutput,ProducerName=name},cancellation);
    public Task<Response> RetireOutputAsync(bool recreate=false,CancellationToken cancellation=default)=>CallAsync(new Request{ComputationalState=ComputationalState,Revision=Revision,Operation=recreate?Request.Types.Operation.RecreateOutput:Request.Types.Operation.RetireOutput},cancellation);
    public Task<Response> RetireProducerAsync(bool recreate=false,CancellationToken cancellation=default)=>CallAsync(new Request{ComputationalState=ComputationalState,Revision=Revision,Operation=recreate?Request.Types.Operation.RecreateProducer:Request.Types.Operation.RetireProducer},cancellation);
    public Task<Response> PublishUnavailableAsync(CancellationToken cancellation=default)=>CallAsync(new Request{ComputationalState=ComputationalState,Revision=Revision,Operation=Request.Types.Operation.PublishUnavailable,ProofEpoch=ProofEpoch},cancellation);
    public Task<Response> StartProofAsync(CancellationToken cancellation=default)=>CallAsync(new Request{Operation=Request.Types.Operation.ProofStart},cancellation);
    public Task<Response> PublishProofAsync(uint count,string omitKey="",CancellationToken cancellation=default)=>CallAsync(new Request{ComputationalState=ComputationalState,Revision=Revision,Operation=Request.Types.Operation.ProofPublish,ProofCount=count,ProofKey=omitKey,ProofEpoch=ProofEpoch},cancellation);
    public Task<Response> LayoutProofAsync(string id,uint afterRow,bool reverse,CancellationToken cancellation=default)=>CallAsync(new Request{ComputationalState=ComputationalState,Revision=Revision,Operation=Request.Types.Operation.ProofLayout,ProofProjection=id,ProofAfterRow=afterRow,ProofReverse=reverse,ProofPresentation=ProofPresentation},cancellation);
    public Task<Response> ReadProofAsync(Position first,Position last,CancellationToken cancellation=default)=>CallAsync(new Request{ComputationalState=ComputationalState,Revision=Revision,Operation=Request.Types.Operation.ProofRead,First=first.Clone(),Last=last.Clone()},cancellation);
    public Task<Response> EvaluateProofAsync(string capture,string key="",CancellationToken cancellation=default)=>CallAsync(new Request{ComputationalState=ComputationalState,Revision=Revision,Operation=Request.Types.Operation.ProofEvaluate,Capture=capture,ProofKey=key},cancellation);
    public Task<Response> ReadAsync(Position first, Position last, CancellationToken cancellation = default) =>
        CallAsync(new Request { Operation = Request.Types.Operation.Read, First = first.Clone(), Last = last.Clone() }, cancellation);
    public Task<Response> CaptureAsync(CancellationToken cancellation = default) =>
        CallAsync(new Request { Operation = Request.Types.Operation.Capture }, cancellation);
    public Task<Response> CalculateAsync(string capture, IEnumerable<Position> outputs, CancellationToken cancellation = default)
    {
        var selected = outputs.Take(257).Select(p => p.Clone()).ToArray();
        if (selected.Length is 0 or > 256) throw new ArgumentOutOfRangeException(nameof(outputs));
        var request = new Request { Operation = Request.Types.Operation.Calculate, Capture = capture, Worksheet=capturedSheets.TryGetValue(capture,out var sheet)?sheet:WorksheetId };
        request.Outputs.AddRange(selected);
        return CallAsync(request, cancellation);
    }
    public Task<Response> ReleaseAsync(string capture, CancellationToken cancellation = default) =>
        CallAsync(new Request { Operation = Request.Types.Operation.Release, Capture = capture }, cancellation);

    public Task<Response> NewAsync(string expected, bool discard = false, CancellationToken cancellation = default) =>
        LifecycleAsync(Request.Types.Operation.New, expected, "", discard, cancellation);
    public Task<Response> OpenAsync(string expected, string path, bool discard = false, CancellationToken cancellation = default) =>
        LifecycleAsync(Request.Types.Operation.Open, expected, Path.GetFullPath(path), discard, cancellation);
    public Task<Response> CloseDocumentAsync(string expected, bool discard = false, CancellationToken cancellation = default) =>
        LifecycleAsync(Request.Types.Operation.CloseDocument, expected, "", discard, cancellation);
    public Task<Response> PrepareSaveAsync(string expected, string? path = null, CancellationToken cancellation = default) =>
        LifecycleAsync(Request.Types.Operation.PrepareSave, expected, path is null ? "" : Path.GetFullPath(path), false, cancellation);
    private Task<Response> LifecycleAsync(Request.Types.Operation operation, string expected, string path, bool discard, CancellationToken cancellation)
    {
        ArgumentException.ThrowIfNullOrEmpty(expected);
        return CallAsync(new Request { Operation = operation, Revision = expected, Path = path, Discard = discard }, cancellation);
    }
    public Task<Response> CommitSaveAsync(string expected, string ticket, bool abandon = false, CancellationToken cancellation = default)
    {
        ArgumentException.ThrowIfNullOrEmpty(expected); ArgumentException.ThrowIfNullOrEmpty(ticket);
        return CallAsync(new Request { Operation = Request.Types.Operation.Save, Revision = expected, SaveTicket = ticket, Discard = abandon }, cancellation);
    }
    public async Task<Response> SaveAsync(string expected, string? path = null, CancellationToken cancellation = default)
    {
        var prepared = await PrepareSaveAsync(expected, path, cancellation);
        try { return await CommitSaveAsync(prepared.Revision, prepared.SaveTicket, cancellation: cancellation); }
        catch {
            // If cancellation or a stale revision prevented commit dispatch, release the retained export.
            try { await CommitSaveAsync(Revision, prepared.SaveTicket, abandon: true); }
            catch (WorkbookSessionException) { } // Already consumed by a drained commit.
            throw;
        }
    }

    private async Task<Response> CallAsync(Request request, CancellationToken cancellation)
    {
        // Capture sheet context before queueing; tab switches cannot redirect this request.
        if(request.Worksheet.Length==0)request.Worksheet=WorksheetId;
        // Reserve envelope space before accepting a queued request, so the queue
        // cannot retain eight arbitrarily large caller-owned formula buffers.
        if (request.CalculateSize() > MaximumFrame - 256)
            throw new WorkbookSessionException("Request exceeds session frame limit.");
        if (Interlocked.Increment(ref outstanding) > 8 && request.Operation != Request.Types.Operation.Close) {
            Interlocked.Decrement(ref outstanding); throw new WorkbookSessionException("Session queue is full.");
        }
        bool acquired = false;
        string? abandonedCapture = null;
        string? abandonedSave = null;
        string? abandonedCsv = null;
        try {
            await gate.WaitAsync(cancellation).ConfigureAwait(false); acquired = true;
            ObjectDisposedException.ThrowIf(closed, this);
            ObjectDisposedException.ThrowIf(Volatile.Read(ref closing) != 0 && request.Operation != Request.Types.Operation.Close, this);
            cancellation.ThrowIfCancellationRequested();
            if(request.Operation==Request.Types.Operation.Capture) request.ProofEpoch=ProofEpoch;
            if(request.ComputationalState.Length==0)request.ComputationalState=ComputationalState;
            request.Version = 9; request.Id = ++sequence;
            if (request.Revision.Length == 0) request.Revision = Revision;
            var serialization = Stopwatch.StartNew();
            if (request.CalculateSize() is <= 0 or > MaximumFrame) throw new WorkbookSessionException("Request exceeds session frame limit.");
            var bytes = request.ToByteArray();
            LastClientSerialization = serialization.Elapsed;
            var header = new byte[4]; BinaryPrimitives.WriteInt32LittleEndian(header, bytes.Length);
            // A deadline breaks a stalled or dead child; never abandon a half-read response.
            using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(15));
            var clock = Stopwatch.StartNew();
            Response response;
            try {
                await process.StandardInput.BaseStream.WriteAsync(header, deadline.Token).ConfigureAwait(false);
                await process.StandardInput.BaseStream.WriteAsync(bytes, deadline.Token).ConfigureAwait(false);
                await process.StandardInput.BaseStream.FlushAsync(deadline.Token).ConfigureAwait(false);
                RequestDispatchedForTesting?.Invoke();
                await process.StandardOutput.BaseStream.ReadExactlyAsync(header, deadline.Token).ConfigureAwait(false);
                int size = BinaryPrimitives.ReadInt32LittleEndian(header);
                if (size is <= 0 or > MaximumFrame) throw new IOException("Invalid Pivot response frame.");
                var payload = new byte[size];
                await process.StandardOutput.BaseStream.ReadExactlyAsync(payload, deadline.Token).ConfigureAwait(false);
                var decode = Stopwatch.StartNew();
                response = Response.Parser.ParseFrom(payload);
                LastClientDeserialization = decode.Elapsed;
                if (response.Version != 9 || response.Id != request.Id) throw new IOException("Mismatched Pivot response.");
                foreach(var cell in response.Cells) ValidateResult(cell);
            }
            catch (Exception e) when (e is IOException or OperationCanceledException or InvalidProtocolBufferException) {
                closed = true; StopChild();
                throw new IOException("Pivot transport failed. An in-flight edit may have committed; the session is closed and requests are never replayed.", e);
            }
            LastRoundTrip = clock.Elapsed;
            LastCompletedRequestId = response.Id;
            LastRustExecutionNs = response.ExecutionNs;
            LastRustSerializationNs = response.SerializationNs;
            if (request.Operation is not (Request.Types.Operation.Calculate or Request.Types.Operation.Release or Request.Types.Operation.ProofEvaluate))
                Volatile.Write(ref revision, response.Revision);
            IsProjectionProof=response.ProofMode;IsBindingProof=response.BindingMode;ComputationalState=response.ComputationalState;ProofEpoch=response.ProofEpoch;ProofPresentation=response.ProofPresentation;
            bool documentChanged=DocumentId!=response.DocumentId || (response.Error.Length==0 && request.Operation is (Request.Types.Operation.Create or Request.Types.Operation.New or Request.Types.Operation.Open or Request.Types.Operation.CloseDocument));
            if(documentChanged)DocumentGeneration++;
            Worksheets=response.Worksheets.Select(s=>s.Clone()).ToArray();
            if(documentChanged || !Worksheets.Any(s=>s.Id==WorksheetId)) {WorksheetId=Worksheets.FirstOrDefault()?.Id??"";capturedSheets.Clear();}
            if(request.Operation==Request.Types.Operation.Capture && response.Capture.Length!=0)capturedSheets[response.Capture]=request.Worksheet;
            if(request.Operation==Request.Types.Operation.Release)capturedSheets.TryRemove(request.Capture,out _);
            DocumentId = response.DocumentId; DocumentPath = response.Path;
            PositionedCells=response.PositionedCells;AllocatedCellIdentities=response.AllocatedCellIdentities;
            PositionedCellLimit=response.PositionedCellLimit;AllocatedCellIdentityLimit=response.AllocatedCellIdentityLimit;
            SavedCheckpoint = response.SavedCheckpoint; IsDirty = response.Dirty;
            InstallationUncertain = response.Uncertain;
            if (cancellation.IsCancellationRequested && response.SaveTicket.Length != 0) abandonedSave = response.SaveTicket;
            if (cancellation.IsCancellationRequested && response.CsvTicket.Length != 0) abandonedCsv = response.CsvTicket;
            if (response.Closed) { FinalCharge = response.RemainingCharge; closed = true; }
            if (cancellation.IsCancellationRequested && response.Capture.Length != 0) abandonedCapture = response.Capture;
            cancellation.ThrowIfCancellationRequested();
            if (response.Error.Length != 0) throw new WorkbookSessionException(response.Error, response.Conflict);
            return response;
        }
        finally {
            if (acquired) gate.Release(); Interlocked.Decrement(ref outstanding);
            if (abandonedCsv is not null && !closed) {
                try { await CancelCsvImportAsync(abandonedCsv).ConfigureAwait(false); }
                catch (Exception e) when (e is IOException or WorkbookSessionException or ObjectDisposedException) { closed = true; StopChild(); }
            }
            if (abandonedSave is not null && !closed) {
                try { await CommitSaveAsync(Revision, abandonedSave, abandon: true).ConfigureAwait(false); }
                catch (Exception e) when (e is IOException or WorkbookSessionException or ObjectDisposedException) { closed = true; StopChild(); }
            }
            if (abandonedCapture is not null && !closed) {
                try { await ReleaseAsync(abandonedCapture).ConfigureAwait(false); }
                catch (Exception e) when (e is IOException or WorkbookSessionException or ObjectDisposedException) { closed = true; StopChild(); }
            }
        }
    }
    private void StopChild()
    {
        try { if (!process.HasExited) process.Kill(entireProcessTree: true); }
        catch (InvalidOperationException) { }
    }
    public ValueTask DisposeAsync()
    {
        lock (gate) { return new ValueTask(disposal ??= DisposeCoreAsync()); }
    }
    private async Task DisposeCoreAsync()
    {
        Interlocked.Exchange(ref closing, 1);
        // Serialized close clears captures/book and joins Rust runtime workers.
        if (!closed) {
            try { await CallAsync(new Request { Operation = Request.Types.Operation.Close }, CancellationToken.None).ConfigureAwait(false); }
            catch (Exception e) when (e is IOException or WorkbookSessionException or ObjectDisposedException) { closed = true; StopChild(); }
        }
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(5));
        try { await process.WaitForExitAsync(deadline.Token).ConfigureAwait(false); }
        catch (OperationCanceledException) { StopChild(); await process.WaitForExitAsync().ConfigureAwait(false); }
        process.Dispose();
    }
    private static void ValidateResult(Cell cell) {
        if(cell.Error.Length!=0) {if(cell.Result is not null || cell.Value is not null)throw new IOException("Fault has a successful value.");return;}
        var result=cell.Result??throw new IOException("Missing typed result.");
        switch(result.KindCase) {
            case TypedValue.KindOneofCase.Decimal:
                _=Decode(result.Decimal);
                if(cell.Value is null || !cell.Value.Equals(result.Decimal))throw new IOException("Mismatched Decimal mirror.");break;
            case TypedValue.KindOneofCase.Boolean: case TypedValue.KindOneofCase.Text: case TypedValue.KindOneofCase.Blank:
                if(cell.Value is not null || (result.KindCase==TypedValue.KindOneofCase.Blank && !result.Blank) || (result.KindCase==TypedValue.KindOneofCase.Text && System.Text.Encoding.UTF8.GetByteCount(result.Text)>4096))throw new IOException("Invalid typed result.");break;
            default:throw new IOException("Unsupported typed result.");
        }
    }
    public static string DisplayValue(Cell cell) => cell.Result?.KindCase switch {
        TypedValue.KindOneofCase.Boolean => cell.Result.Boolean?"TRUE":"FALSE",
        TypedValue.KindOneofCase.Text => cell.Result.Text,
        TypedValue.KindOneofCase.Decimal => Decode(cell.Result.Decimal).ToString(System.Globalization.CultureInfo.InvariantCulture),
        TypedValue.KindOneofCase.Blank => "",
        _ => cell.Value is not null ? Decode(cell.Value).ToString(System.Globalization.CultureInfo.InvariantCulture) : ""
    };
    public static Decimal128 Encode(decimal value)
    {
        var bits = decimal.GetBits(value);
        return new Decimal128 { Lo = (uint)bits[0], Mid = (uint)bits[1], Hi = (uint)bits[2], Scale = (uint)((bits[3] >> 16) & 255), Negative = bits[3] < 0 };
    }
    public static decimal Decode(Decimal128 value)
    {
        if (value.Scale > 28) throw new IOException("Invalid decimal scale.");
        return new decimal((int)value.Lo, (int)value.Mid, (int)value.Hi, value.Negative, (byte)value.Scale);
    }
}
