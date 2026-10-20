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
public sealed class WorkbookSession : IAsyncDisposable
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
    public string Revision => Volatile.Read(ref revision);
    public TimeSpan LastRoundTrip { get; private set; }
    public TimeSpan LastClientSerialization { get; private set; }
    public TimeSpan LastClientDeserialization { get; private set; }
    public ulong LastRustExecutionNs { get; private set; }
    public ulong LastRustSerializationNs { get; private set; }
    public ulong LastCompletedRequestId { get; private set; }
    public int ProcessId { get; }
    public ulong? FinalCharge { get; private set; }
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
        var request = new Request { Operation = Request.Types.Operation.Apply, Revision = expected };
        request.Edits.AddRange(batch);
        return CallAsync(request, cancellation);
    }
    public Task<Response> ReadAsync(Position first, Position last, CancellationToken cancellation = default) =>
        CallAsync(new Request { Operation = Request.Types.Operation.Read, First = first.Clone(), Last = last.Clone() }, cancellation);
    public Task<Response> CaptureAsync(CancellationToken cancellation = default) =>
        CallAsync(new Request { Operation = Request.Types.Operation.Capture }, cancellation);
    public Task<Response> CalculateAsync(string capture, IEnumerable<Position> outputs, CancellationToken cancellation = default)
    {
        var selected = outputs.Take(257).Select(p => p.Clone()).ToArray();
        if (selected.Length is 0 or > 256) throw new ArgumentOutOfRangeException(nameof(outputs));
        var request = new Request { Operation = Request.Types.Operation.Calculate, Capture = capture };
        request.Outputs.AddRange(selected);
        return CallAsync(request, cancellation);
    }
    public Task<Response> ReleaseAsync(string capture, CancellationToken cancellation = default) =>
        CallAsync(new Request { Operation = Request.Types.Operation.Release, Capture = capture }, cancellation);

    private async Task<Response> CallAsync(Request request, CancellationToken cancellation)
    {
        // Reserve envelope space before accepting a queued request, so the queue
        // cannot retain eight arbitrarily large caller-owned formula buffers.
        if (request.CalculateSize() > MaximumFrame - 256)
            throw new WorkbookSessionException("Request exceeds session frame limit.");
        if (Interlocked.Increment(ref outstanding) > 8 && request.Operation != Request.Types.Operation.Close) {
            Interlocked.Decrement(ref outstanding); throw new WorkbookSessionException("Session queue is full.");
        }
        bool acquired = false;
        string? abandonedCapture = null;
        try {
            await gate.WaitAsync(cancellation).ConfigureAwait(false); acquired = true;
            ObjectDisposedException.ThrowIf(closed, this);
            ObjectDisposedException.ThrowIf(Volatile.Read(ref closing) != 0 && request.Operation != Request.Types.Operation.Close, this);
            cancellation.ThrowIfCancellationRequested();
            request.Version = 1; request.Id = ++sequence;
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
                await process.StandardOutput.BaseStream.ReadExactlyAsync(header, deadline.Token).ConfigureAwait(false);
                int size = BinaryPrimitives.ReadInt32LittleEndian(header);
                if (size is <= 0 or > MaximumFrame) throw new IOException("Invalid Pivot response frame.");
                var payload = new byte[size];
                await process.StandardOutput.BaseStream.ReadExactlyAsync(payload, deadline.Token).ConfigureAwait(false);
                var decode = Stopwatch.StartNew();
                response = Response.Parser.ParseFrom(payload);
                LastClientDeserialization = decode.Elapsed;
                if (response.Version != 1 || response.Id != request.Id) throw new IOException("Mismatched Pivot response.");
            }
            catch (Exception e) when (e is IOException or OperationCanceledException or InvalidProtocolBufferException) {
                closed = true; StopChild();
                throw new IOException("Pivot transport failed. An in-flight edit may have committed; the session is closed and requests are never replayed.", e);
            }
            LastRoundTrip = clock.Elapsed;
            LastCompletedRequestId = response.Id;
            LastRustExecutionNs = response.ExecutionNs;
            LastRustSerializationNs = response.SerializationNs;
            if (request.Operation is Request.Types.Operation.Create or Request.Types.Operation.Apply or Request.Types.Operation.Read)
                Volatile.Write(ref revision, response.Revision);
            if (response.Closed) { FinalCharge = response.RemainingCharge; closed = true; }
            if (cancellation.IsCancellationRequested && response.Capture.Length != 0) abandonedCapture = response.Capture;
            cancellation.ThrowIfCancellationRequested();
            if (response.Error.Length != 0) throw new WorkbookSessionException(response.Error, response.Conflict);
            return response;
        }
        finally {
            if (acquired) gate.Release(); Interlocked.Decrement(ref outstanding);
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
