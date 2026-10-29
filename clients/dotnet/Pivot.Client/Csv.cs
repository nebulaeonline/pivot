using Pivot.Protocol.Application.V1;

namespace Pivot.Client;
public sealed partial class WorkbookSession
{
    public Task<Response> CsvBoundsAsync(CancellationToken cancellation = default) =>
        CallAsync(new Request { Operation = Request.Types.Operation.CsvBounds }, cancellation);
    public Task<Response> PrepareCsvImportAsync(string path, CancellationToken cancellation = default) =>
        CallAsync(new Request { Operation = Request.Types.Operation.PrepareCsvImport, Path = Path.GetFullPath(path) }, cancellation);
    public Task<Response> CancelCsvImportAsync(string ticket, CancellationToken cancellation = default) =>
        CallAsync(new Request { Operation = Request.Types.Operation.CancelCsvImport, CsvTicket = ticket }, cancellation);
    public Task<Response> ImportCsvAsync(string expected, string ticket, bool typed = false, bool preserveLeadingZeros = true,
        bool recognizeQuoted = false, bool discard = false, CancellationToken cancellation = default) =>
        CallAsync(new Request { Operation = Request.Types.Operation.ImportCsv, Revision = expected, CsvTicket = ticket,
            CsvTyped = typed, CsvConvertLeadingZeros = !preserveLeadingZeros, CsvRecognizeQuoted = recognizeQuoted, Discard = discard }, cancellation);
    public Task<Response> ExportCsvAsync(string expected, string path, Position first, Position last, bool wholeSheet = false, CancellationToken cancellation = default) =>
        CallAsync(new Request { Operation = Request.Types.Operation.ExportCsv, Revision = expected, Path = Path.GetFullPath(path),
            First = first.Clone(), Last = last.Clone(), CsvWholeSheet = wholeSheet }, cancellation);
}
