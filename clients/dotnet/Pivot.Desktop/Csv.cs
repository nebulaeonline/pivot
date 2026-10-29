using System;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Platform.Storage;
using Pivot.Protocol.Application.V1;

namespace Pivot.Desktop;
public partial class MainWindow
{
    private sealed record CsvImportOptions(bool Typed, bool PreserveZeros, bool InferQuoted);
    private static readonly FilePickerFileType CsvFile = new("CSV files") { Patterns = ["*.csv"], MimeTypes = ["text/csv"] };

    private async Task<CsvImportOptions?> CsvOptionsAsync()
    {
        var dialog = new Window { Title = "Import CSV", Width = 580, Height = 450, CanResize = false, WindowStartupLocation = WindowStartupLocation.CenterOwner };
        var panel = new StackPanel { Margin = new Thickness(16), Spacing = 12 };
        var mode = new ComboBox { ItemsSource = new[] { "Text", "Numbers and Booleans" }, SelectedIndex = 0, HorizontalAlignment = global::Avalonia.Layout.HorizontalAlignment.Stretch };
        var zeros = new CheckBox { Content = "Preserve leading-zero integer identifiers", IsChecked = true, IsEnabled = false };
        var quotation = new ComboBox { ItemsSource = new[] { "Preserve quoted fields as Strings", "Allow type inference in quoted fields" }, SelectedIndex = 0, IsEnabled = false, HorizontalAlignment = global::Avalonia.Layout.HorizontalAlignment.Stretch };
        mode.SelectionChanged += (_, _) => zeros.IsEnabled = quotation.IsEnabled = mode.SelectedIndex == 1;
        panel.Children.Add(new TextBlock { Text = "Import into a new, unsaved workbook. UTF-8, comma-separated CSV only.\nLimits: 1,024 live cells; 1,024 × 1,024 positions; 65,536 fields; 4 MiB.", TextWrapping = global::Avalonia.Media.TextWrapping.Wrap });
        panel.Children.Add(mode); panel.Children.Add(quotation); panel.Children.Add(zeros);
        panel.Children.Add(new TextBlock { Text = "Leading zeros in unsigned integers stay text by default. Signed and decimal-point values can become numbers. Quoted empty fields remain empty Strings; unquoted empty fields are Blank. Formulas never execute.", TextWrapping = global::Avalonia.Media.TextWrapping.Wrap });
        var buttons = new StackPanel { Orientation = global::Avalonia.Layout.Orientation.Horizontal, Spacing = 12 };
        var import = new Button { Content = "Choose CSV…" }; var cancel = new Button { Content = "Cancel" };
        import.Click += (_, _) => dialog.Close(new CsvImportOptions(mode.SelectedIndex == 1, zeros.IsChecked == true, quotation.SelectedIndex == 1));
        cancel.Click += (_, _) => dialog.Close(); buttons.Children.Add(import); buttons.Children.Add(cancel); panel.Children.Add(buttons); dialog.Content = panel;
        return await dialog.ShowDialog<CsvImportOptions?>(this);
    }

    private async Task<bool?> CsvAllQuotedAsync()
    {
        var dialog = new Window { Title = "Every CSV field is quoted", Width = 570, Height = 250, CanResize = false, WindowStartupLocation = WindowStartupLocation.CenterOwner };
        var panel = new StackPanel { Margin = new Thickness(16), Spacing = 16 };
        panel.Children.Add(new TextBlock { Text = "Some programs quote every field automatically. Keep everything as text, or recognize numbers and Booleans inside the quotes?\n\nYour leading-zero preservation setting still applies. Empty Strings stay empty Strings. This choice applies only to this import.", TextWrapping = global::Avalonia.Media.TextWrapping.Wrap });
        var buttons = new StackPanel { Orientation = global::Avalonia.Layout.Orientation.Horizontal, Spacing = 8 };
        var keep = new Button { Content = "Keep as text", IsDefault = true };
        var infer = new Button { Content = "Recognize numbers and Booleans" };
        var cancel = new Button { Content = "Cancel", IsCancel = true };
        keep.Click += (_, _) => dialog.Close(false); infer.Click += (_, _) => dialog.Close(true); cancel.Click += (_, _) => dialog.Close();
        buttons.Children.Add(keep); buttons.Children.Add(infer); buttons.Children.Add(cancel); panel.Children.Add(buttons); dialog.Content = panel;
        return await dialog.ShowDialog<bool?>(this);
    }

    private async Task ImportCsvDocumentAsync(string path, CsvImportOptions options, Func<Task<bool?>>? allQuotedChoice = null, Func<Task<bool>>? confirmDiscard = null)
    {
        string? ticket = null;
        try {
            var prepared = await session!.PrepareCsvImportAsync(path); ticket = prepared.CsvTicket;
            if (options.Typed && prepared.CsvAllQuoted && !options.InferQuoted) {
                var infer = await (allQuotedChoice?.Invoke() ?? CsvAllQuotedAsync());
                if (infer is null) return;
                options = options with { InferQuoted = infer.Value };
            }
            if (!await (confirmDiscard?.Invoke() ?? ConfirmDiscardAsync())) return;
            refresh?.Cancel(); ++displayRequest;
            string consumed = ticket; ticket = null;
            await session.ImportCsvAsync(session.Revision, consumed, options.Typed, options.PreserveZeros, options.InferQuoted, discard: true);
            Sheet.Select(new Position { Row = 1, Column = 1 }); await RefreshAsync(); UpdateDocument();
            StatusLabel.Text = "CSV imported into a new workbook. Save as .pivot to preserve your workbook.";
        }
        finally { if (ticket is not null) await session!.CancelCsvImportAsync(ticket); }
    }
    private async void ImportCsvClick(object? sender, RoutedEventArgs e) => await FileActionAsync(async () => {
        var options = await CsvOptionsAsync(); if (options is null) return;
        var files = await StorageProvider.OpenFilePickerAsync(new FilePickerOpenOptions { Title = "Import CSV", AllowMultiple = false, FileTypeFilter = [CsvFile] });
        var path = files.Count == 1 ? files[0].TryGetLocalPath() : null; if (path is null) return;
        await ImportCsvDocumentAsync(path, options);
    });

    private async void ExportCsvClick(object? sender, RoutedEventArgs e) => await FileActionAsync(async () => {
        if (session!.DocumentId.Length == 0) return;
        bool hasContent = (await session.CsvBoundsAsync()).CsvHasContent;
        var dialog = new Window { Title = "Export CSV", Width = 550, Height = 280, CanResize = false, WindowStartupLocation = WindowStartupLocation.CenterOwner };
        var panel = new StackPanel { Margin = new Thickness(16), Spacing = 14 };
        var scope = new ComboBox { SelectedIndex = 0 };
        scope.Items.Add(new ComboBoxItem { Content = "Selected rectangle" });
        scope.Items.Add(new ComboBoxItem { Content = "Active sheet content", IsEnabled = hasContent });
        panel.Children.Add(scope);
        panel.Children.Add(new TextBlock { Text = "Exports exact calculated values. Formulas, styles and other sheets are omitted. Strings are quoted to preserve their types in Pivot; other programs may interpret types or formula-like text differently. Limits: 65,536 fields and 4 MiB.", TextWrapping = global::Avalonia.Media.TextWrapping.Wrap });
        var buttons = new StackPanel { Orientation = global::Avalonia.Layout.Orientation.Horizontal, Spacing = 12 };
        var export = new Button { Content = "Choose destination…" }; var cancel = new Button { Content = "Cancel" };
        export.Click += (_, _) => dialog.Close(scope.SelectedIndex); cancel.Click += (_, _) => dialog.Close();
        buttons.Children.Add(export); buttons.Children.Add(cancel); panel.Children.Add(buttons); dialog.Content = panel;
        var choice = await dialog.ShowDialog<int?>(this); if (choice is null) return;
        var file = await StorageProvider.SaveFilePickerAsync(new FilePickerSaveOptions { Title = "Export CSV", SuggestedFileName = "Worksheet.csv", DefaultExtension = "csv", FileTypeChoices = [CsvFile], ShowOverwritePrompt = true });
        var path = file?.TryGetLocalPath(); if (path is null) return;
        await session.ExportCsvAsync(session.Revision, path, Sheet.SelectionFirst, Sheet.SelectionLast, choice == 1);
        StatusLabel.Text = "CSV exported. Native workbook save state is unchanged.";
    });
}
