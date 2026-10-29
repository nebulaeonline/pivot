using System;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.VisualTree;
using Pivot.Protocol.Application.V1;

namespace Pivot.Desktop;
public partial class MainWindow
{
    private async Task CsvAutomationAsync(string report)
    {
        var s = session!;
        string root = Path.Combine(Path.GetDirectoryName(Path.GetFullPath(report))!, "csv-fixtures");
        Directory.CreateDirectory(root);
        string input = Path.Combine(root, "all-quoted.csv"), output = Path.Combine(root, "export.csv");
        async Task<T> Choose<T>(Func<Task<T>> show, string title, string button, Action<Window>? setup = null)
        {
            var pending = show();
            await AutomationWaitAsync(() => OwnedWindows.Any(w => w.Title == title));
            var dialog = OwnedWindows.Single(w => w.Title == title);
            setup?.Invoke(dialog); dialog.UpdateLayout();
            var chosen = dialog.GetVisualDescendants().OfType<Button>().Single(b => Equals(b.Content, button));
            chosen.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
            return await pending;
        }
        async Task<bool?> AllQuoted(string choice) => await Choose(CsvAllQuotedAsync, "Every CSV field is quoted", choice);
        await s.NewAsync(s.Revision, discard: true); await RefreshAsync();
        var options = await Choose(CsvOptionsAsync, "Import CSV", "Choose CSV…", dialog => {
            var combos = dialog.GetVisualDescendants().OfType<ComboBox>().ToArray();
            AutomationCheck(combos[0].SelectedIndex == 0 && !combos[1].IsEnabled, "CSV defaults to Text");
            var zeros = dialog.GetVisualDescendants().OfType<CheckBox>().Single();
            AutomationCheck(zeros.IsChecked == true && !zeros.IsEnabled, "CSV preserves zeros by default");
            combos[0].SelectedIndex = 1;
            AutomationCheck(zeros.IsEnabled && combos[1].IsEnabled && combos[1].SelectedIndex == 0, "typed mode exposes independent quotation and zero settings");
        });
        AutomationCheck(options is { Typed: true, PreserveZeros: true, InferQuoted: false }, "CSV options reflect actual controls");
        await File.WriteAllTextAsync(input, "\"001234\",\"1234\",\"TRUE\",\"\",\"+001234\",\"00.50\",\"=1+1\"", new UTF8Encoding(false));
        string before = s.Revision;
        await ImportCsvDocumentAsync(input, options!, () => AllQuoted("Cancel"), () => Task.FromResult(true));
        AutomationCheck(s.Revision == before && !s.IsDirty, "all-quoted Cancel keeps workbook");
        await ImportCsvDocumentAsync(input, options!, () => AllQuoted("Keep as text"), () => Task.FromResult(true));
        AutomationCheck(Sheet.CellAt(new Position { Row = 1, Column = 2 })?.Result.KindCase == TypedValue.KindOneofCase.Text, "Keep as text reaches grid");
        await ImportCsvDocumentAsync(input, options!, () => AllQuoted("Recognize numbers and Booleans"), () => Task.FromResult(true));
        AutomationCheck(Sheet.CellAt(new Position { Row = 1, Column = 1 })?.Result.Text == "001234", "identifier survives quoted inference");
        AutomationCheck(Sheet.CellAt(new Position { Row = 1, Column = 2 })?.Result.KindCase == TypedValue.KindOneofCase.Decimal, "quoted inference reaches grid");
        AutomationCheck(Sheet.CellAt(new Position { Row = 1, Column = 4 })?.Result.KindCase == TypedValue.KindOneofCase.Text && Sheet.CellAt(new Position { Row = 1, Column = 7 })?.Result.Text == "=1+1", "empty String and non-executable formula text");
        AutomationCheck(s.DocumentPath.Length == 0 && s.IsDirty && Sheet.Selected.Row == 1 && Sheet.Selected.Column == 1, "import lifecycle and selection");
        await s.ExportCsvAsync(s.Revision, output, new Position { Row = 1, Column = 1 }, new Position { Row = 1, Column = 7 });
        AutomationCheck(await File.ReadAllTextAsync(output) == "\"001234\",1234,TRUE,\"\",1234,0.50,\"=1+1\"\r\n", "typed export bytes");
        await ImportCsvDocumentAsync(input, new CsvImportOptions(false, true, false), () => throw new Exception("Text mode must not prompt"), () => Task.FromResult(true));
        await ImportCsvDocumentAsync(input, new CsvImportOptions(true, false, true), () => throw new Exception("Explicit inference must not prompt"), () => Task.FromResult(true));
        AutomationCheck(Sheet.CellAt(new Position { Row = 1, Column = 1 })?.Result.KindCase == TypedValue.KindOneofCase.Decimal, "explicit zero conversion reaches grid");
        before = s.Revision;
        await ImportCsvDocumentAsync(input, new CsvImportOptions(false, true, false), confirmDiscard: () => Task.FromResult(false));
        AutomationCheck(s.Revision == before, "discard cancellation keeps workbook and releases ticket");
        await File.WriteAllTextAsync(input, "001234,\"1234\",,\"\"");
        await ImportCsvDocumentAsync(input, options!, () => throw new Exception("Mixed CSV must not prompt"), () => Task.FromResult(true));
        AutomationCheck(Sheet.CellAt(new Position { Row = 1, Column = 3 }) is null && Sheet.CellAt(new Position { Row = 1, Column = 4 })?.Result.KindCase == TypedValue.KindOneofCase.Text, "Blank versus empty String in grid");
        await ImportCsvDocumentAsync(input, new CsvImportOptions(true, true, true), () => throw new Exception("Mixed CSV explicit inference must not prompt"), () => Task.FromResult(true));
        AutomationCheck(Sheet.CellAt(new Position { Row = 1, Column = 2 })?.Result.KindCase == TypedValue.KindOneofCase.Decimal, "mixed quotation supports inference override");
        await SaveDocumentAsync(true, Path.Combine(root, "imported.pivot"));
        await OpenDocumentAsync(Path.Combine(root, "imported.pivot"));
        AutomationCheck(!s.IsDirty && Sheet.CellAt(new Position { Row = 1, Column = 1 })?.Result.Text == "001234", "import native save/reopen in actual window");
        AutomationImage(Path.ChangeExtension(report, "csv.png"));
        await s.NewAsync(s.Revision, discard: true); CancelDraft(); await RefreshAsync(); Sheet.Focus();
    }
}
