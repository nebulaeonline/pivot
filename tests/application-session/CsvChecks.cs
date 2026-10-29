using System.Text;
using System.Text.Json;
using Pivot.Client;
using Pivot.Protocol.Application.V1;

internal static class CsvChecks
{
    static Position P(uint row, uint column = 1) => new() { Row = row, Column = column };
    static void Check(bool value, string message) { if (!value) throw new Exception("CSV: " + message); }
    static async Task Reject(Func<Task> action, string message)
    {
        try { await action(); throw new Exception("CSV unexpectedly accepted: " + message); }
        catch (WorkbookSessionException e) { Check(e.Message.Contains(message, StringComparison.OrdinalIgnoreCase), e.Message); }
    }
    public static async Task Run(string binary, string output)
    {
        Directory.CreateDirectory(output);
        string root = Environment.GetEnvironmentVariable("PIVOT_FILE_TEST_ROOT") ?? Path.GetTempPath();
        root = Path.Combine(root, "csv-" + Guid.NewGuid()); Directory.CreateDirectory(root);
        string source = Path.Combine(root, "input.csv"), destination = Path.Combine(root, "export.csv"), native = Path.Combine(root, "saved.pivot");
        var s = await WorkbookSession.StartAsync(binary);
        try {
            await s.ApplyAsync(s.Revision, [new Edit { At = P(1), Entry = "99" }]);
            await s.SaveAsync(s.Revision, native);
            string originalId = s.DocumentId, originalRevision = s.Revision, savedPath = s.DocumentPath;
            await File.WriteAllTextAsync(source, "\"001234\",\"1234\",\"TRUE\",\"\",\"+001234\",\"00.50\",\"=1+1\"", new UTF8Encoding(false));
            var prepare = await s.PrepareCsvImportAsync(source);
            Check(prepare.CsvAllQuoted && prepare.CsvTicket.Length > 0, "all-quoted detection");
            Check(s.DocumentId == originalId && s.Revision == originalRevision && !s.IsDirty && s.DocumentPath == savedPath, "prepare is read-only");
            await Reject(() => s.PrepareCsvImportAsync(source), "already prepared");
            // A changing source must not change the bytes the user reviewed.
            await File.WriteAllTextAsync(source, "altered");
            await s.ImportCsvAsync(s.Revision, prepare.CsvTicket, typed: true, recognizeQuoted: true);
            var frame = await s.ReadAsync(P(1), P(1, 7)); Cell At(uint c) => frame.Cells.Single(x => x.At.Column == c);
            Check(At(1).Result.Text == "001234" && At(2).Result.KindCase == TypedValue.KindOneofCase.Decimal && At(3).Result.Boolean, "explicit quoted inference keeps leading-zero identifier");
            Check(At(4).Result.KindCase == TypedValue.KindOneofCase.Text && At(4).Result.Text == "", "quoted empty remains empty String");
            Check(WorkbookSession.Decode(At(5).Result.Decimal) == 1234m && WorkbookSession.Decode(At(6).Result.Decimal) == .5m && At(7).Result.Text == "=1+1", "signs decimals and no formula execution");
            Check(s.DocumentId != originalId && s.IsDirty && s.DocumentPath == "" && s.Worksheets.Count == 1 && s.Worksheets[0].Name == "Sheet1", "new unsaved single-sheet lifecycle");
            await s.SaveAsync(s.Revision, native); await s.OpenAsync(s.Revision, native);
            Check(!s.IsDirty && (await s.ReadAsync(P(1), P(1))).Cells.Single().Result.Text == "001234", "native reopen preserves identifier");

            // Mixed quoting, exact Unicode/newlines, Blank versus empty String.
            await File.WriteAllTextAsync(source, "001234,\"001234\",TRUE,\"TRUE\",,\"\",\"hé\r\nλ\"\r\n+001234,\"+001234\",00.50,\"00.50\",,\"\",\"a,\"\"b\"\"\"\r\n", new UTF8Encoding(false));
            prepare = await s.PrepareCsvImportAsync(source); Check(!prepare.CsvAllQuoted, "mixed quoting detected");
            await s.ImportCsvAsync(s.Revision, prepare.CsvTicket, typed: true, discard: true);
            frame = await s.ReadAsync(P(1), P(2, 7));
            Check(frame.Cells.Single(x => x.At.Row == 1 && x.At.Column == 4).Result.Text == "TRUE", "quoted Boolean preserved by default");
            Check(!frame.Cells.Any(x => x.At.Row == 1 && x.At.Column == 5), "unquoted empty remains graph-free Blank");
            await s.SaveAsync(s.Revision, native); await s.OpenAsync(s.Revision, native);
            Check((await s.ReadAsync(P(1, 7), P(1, 7))).Cells.Single().Result.Text == "hé\r\nλ", "multiline native reopen");
            var copy = await s.CopyBlockAsync(s.Revision, P(1, 7), P(1, 7));
            await s.PasteBlockAsync(s.Revision, P(3, 7), P(3, 7), copy.Clipboard.ToByteArray(), true);
            Check((await s.ReadAsync(P(3, 7), P(3, 7))).Cells.Single().Result.Text == "hé\r\nλ", "native clipboard retains multiline String");
            await s.ApplyAsync(s.Revision, [new Edit { At = P(3, 7), Clear = true }, new Edit { At = P(3), Formula = "=ROUND(12.345,2)" }]);
            string revision = s.Revision, id = s.DocumentId, path = s.DocumentPath; bool dirty = s.IsDirty;
            await s.ExportCsvAsync(revision, destination, P(1), P(3, 7));
            var bytes = await File.ReadAllBytesAsync(destination); string csv = Encoding.UTF8.GetString(bytes);
            Check(csv.StartsWith("\"001234\",\"001234\",TRUE,\"TRUE\",,\"\",\"hé\r\nλ\"\r\n") && csv.Contains("12.35,,,,,,\r\n"), "typed calculated CSV and trailing blanks");
            Check(!bytes.AsSpan().StartsWith(new byte[] { 0xef, 0xbb, 0xbf }), "UTF-8 without BOM");
            Check(s.Revision == revision && s.DocumentId == id && s.DocumentPath == path && s.IsDirty == dirty, "export does not acknowledge a native save");
            await Reject(() => s.ExportCsvAsync(s.Revision, native, P(1), P(1)), "native workbook");
            await s.ApplyAsync(s.Revision, [new Edit { At = P(2, 5), Formula = "=1/0" }, new Edit { At = P(3, 2), Formula = "=2/0" }]);
            await Reject(() => s.ExportCsvAsync(s.Revision, destination, P(1), P(3, 7)), "E2");
            Check(Enumerable.SequenceEqual(bytes, await File.ReadAllBytesAsync(destination)), "evaluation failure preserves destination");
            await Reject(() => s.ExportCsvAsync(s.Revision, destination, P(1), P(1025, 1024)), "logical-field");
            Check(Enumerable.SequenceEqual(bytes, await File.ReadAllBytesAsync(destination)), "quota failure preserves destination");
            await Reject(() => s.ExportCsvAsync(s.Revision, Path.Combine(root, "missing", "out.csv"), P(1), P(1)), "destination");

            revision = s.Revision; id = s.DocumentId;
            foreach (var invalid in new[] { "a,b\nc", "\"unclosed", "a\rb", "\"x\"oops" }) {
                await File.WriteAllTextAsync(source, invalid); await Reject(() => s.PrepareCsvImportAsync(source), "CSV");
                Check(s.Revision == revision && s.DocumentId == id && s.DocumentPath == path && s.IsDirty, "bad CSV keeps previous workbook");
            }
            await File.WriteAllBytesAsync(source, [0xff]); await Reject(() => s.PrepareCsvImportAsync(source), "UTF-8");
            await File.WriteAllBytesAsync(source, new byte[16 * 1024 * 1024 + 1]); await Reject(() => s.PrepareCsvImportAsync(source), "file-size");
            await File.WriteAllTextAsync(source, "a,b"); prepare = await s.PrepareCsvImportAsync(source);
            await s.CancelCsvImportAsync(prepare.CsvTicket); Check(s.Revision == revision, "cancel has no mutation");
            await Reject(() => s.ImportCsvAsync(s.Revision, prepare.CsvTicket, discard: true), "ticket");
            await File.WriteAllTextAsync(source, "79228162514264337593543950336"); prepare = await s.PrepareCsvImportAsync(source);
            await Reject(() => s.ImportCsvAsync(s.Revision, prepare.CsvTicket, typed: true, discard: true), "numeric conversion");
            Check(s.Revision == revision && s.DocumentId == id, "conversion failure atomic");
            await File.WriteAllTextAsync(source, string.Join('\n', Enumerable.Repeat(string.Join(',',Enumerable.Repeat("1",256)),257))); prepare = await s.PrepareCsvImportAsync(source);
            await Reject(() => s.ImportCsvAsync(s.Revision, prepare.CsvTicket, typed: true, discard: true), "live-cell limit");
            Check(s.Revision == revision && s.DocumentId == id, "multi-batch candidate failure atomic");

            await File.WriteAllTextAsync(source, "001234,\"001234\",TRUE,\"TRUE\",,\"\""); prepare = await s.PrepareCsvImportAsync(source);
            await s.ImportCsvAsync(s.Revision, prepare.CsvTicket, typed: true, preserveLeadingZeros: false, recognizeQuoted: true, discard: true);
            frame = await s.ReadAsync(P(1), P(1, 6)); Check(frame.Cells.Take(2).All(x => x.Result.KindCase == TypedValue.KindOneofCase.Decimal), "explicit leading-zero conversion applies to quoted and unquoted");
            await s.ExportCsvAsync(s.Revision, destination, P(1), P(1), wholeSheet: true);
            Check(await File.ReadAllTextAsync(destination) == "1234,1234,TRUE,TRUE,,\"\"\r\n", "whole-sheet content bounds");
            await File.WriteAllTextAsync(source, string.Join('\n', Enumerable.Repeat("1", 1024))); prepare = await s.PrepareCsvImportAsync(source);
            await s.ImportCsvAsync(s.Revision, prepare.CsvTicket, typed: true, discard: true);
            await s.ExportCsvAsync(s.Revision, destination, P(1), P(1), wholeSheet: true);
            Check((await File.ReadAllTextAsync(destination)).Split("\r\n").Length == 1025, "1024 live cells imported/exported in bounded batches");
            // Cancellation after dispatch must reclaim the retained parsed table.
            using var cancel = new CancellationTokenSource();
            s.RequestDispatchedForTesting = () => { s.RequestDispatchedForTesting = null; cancel.Cancel(); };
            try { await s.PrepareCsvImportAsync(source, cancel.Token); throw new Exception("Expected cancellation"); } catch (OperationCanceledException) { }
            prepare = await s.PrepareCsvImportAsync(source); await s.CancelCsvImportAsync(prepare.CsvTicket);
        }
        finally { await s.DisposeAsync(); }
        Check(s.FinalCharge == 0, "zero remaining runtime charge");
        await File.WriteAllTextAsync(Path.Combine(output, "csv.json"), JsonSerializer.Serialize(new { status = "passed", remaining_charge = s.FinalCharge, fixtures = root }));
    }
}
