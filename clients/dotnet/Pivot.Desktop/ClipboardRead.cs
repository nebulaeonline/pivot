using System;
using System.IO;
using System.Threading.Tasks;

namespace Pivot.Desktop;

// Read-only acquisition: retries never publish data or dispatch a workbook edit.
internal static class ClipboardRead
{
    internal readonly record struct Snapshot(bool Exists, bool Native, bool Obsolete,
        bool TextAdvertised, byte[]? Bytes, string? Text, uint NativeKind = 0);
    internal readonly record struct Payload(byte[]? Bytes, string? Text);

    internal static async Task<Snapshot> CaptureAsync<T>(Func<Task<T?>> ownedRead,
        Func<Task<T?>> platformRead, Func<T?,bool,Task<Snapshot>> capture) where T:class,IDisposable
    {
        var owned=await ownedRead();
        var data=owned ?? await platformRead();
        try { return await capture(data,owned is not null); }
        finally { if(owned is null)data?.Dispose(); }
    }

    internal static async Task<Payload> AcquireAsync(Func<Task<Snapshot>> read,
        Func<bool> current, Action<string>? diagnose = null, Func<Task>? delay = null)
    {
        bool? native = null;
        uint? nativeKind = null;
        bool textAdvertised = false;
        for (int attempt = 0; attempt < 3; attempt++)
        {
            if (attempt != 0)
                await (delay?.Invoke() ?? Task.Delay(20));
            if (!current())
            {
                diagnose?.Invoke("clipboard read invalidated");
                return default;
            }
            var snapshot = await read();
            if (!current())
            {
                diagnose?.Invoke("clipboard read invalidated");
                return default;
            }
            diagnose?.Invoke($"clipboard read {attempt + 1}: snapshot={snapshot.Exists}; native={snapshot.Native}; textAdvertised={snapshot.TextAdvertised}; bytesAvailable={snapshot.Bytes is not null}; textAvailable={snapshot.Text is not null}");
            if (snapshot.Obsolete && !snapshot.Native)
                throw new InvalidDataException("Unsupported Pivot clipboard version; copy again using this version.");
            if (native.HasValue && native.Value != snapshot.Native)
                throw new InvalidDataException("Native clipboard changed or is unavailable; copy again.");
            native ??= snapshot.Native;
            if(snapshot.Native) {
                if(nativeKind.HasValue&&nativeKind.Value!=snapshot.NativeKind)
                    throw new InvalidDataException("Preferred native clipboard representation changed; no fallback.");
                nativeKind??=snapshot.NativeKind;
            }
            textAdvertised |= snapshot.TextAdvertised;
            if (snapshot.Native && snapshot.Bytes is not null)
                return new(snapshot.Bytes, "");
            if (!snapshot.Native && snapshot.Text is not null)
                return new(null, snapshot.Text);
        }
        if (native == true)
            throw new InvalidDataException("Native clipboard data is unavailable; copy again.");
        if (textAdvertised)
            throw new InvalidDataException("Clipboard text is unavailable; copy again.");
        return default;
    }
}
