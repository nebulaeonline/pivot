using System;
using System.Collections.Generic;
using System.Runtime.CompilerServices;
using System.Text.Json.Serialization;

namespace Pivot.Desktop;

// Keep existing report keys stable. Every serialized type has generated metadata;
// the desktop disables reflection-based JSON in managed and Native AOT runs alike.
internal sealed class DesktopReport
{
    public string status { get; init; } = "";
    public string? error { get; init; }
    public string? backend { get; init; }
    public string execution_mode { get; init; } = RuntimeFeature.IsDynamicCodeSupported ? "managed" : "native-aot";
    public string? executable { get; init; } = Environment.ProcessPath;
    public int process_id { get; init; } = Environment.ProcessId;
    public double? startup_to_ready_ms { get; init; }
    public int? viewport_rows { get; init; }
    public int? viewport_columns { get; init; }
    public int? maximum_viewport_cells { get; init; }
    public ulong? remaining_charge { get; init; }
    public string? worksheet_binding {get;init;}
    public string? structured_projection { get; init; }
    public string? worksheet_text { get; init; }
    public string? arrow_edit { get; init; }
    public string? clipboard_checklist { get; init; }
    public string? rectangular_clipboard { get; init; }
    public string? rectangular_selection { get; init; }
    public string? direct_paste { get; init; }
    public string? clipboard_focus { get; init; }
    public string? in_place_editing { get; init; }
    public List<RenderTiming>? timings { get; init; }
    public string? note { get; init; }
}
internal sealed class RenderTiming
{
    public double edit_to_render_ms { get; init; }
    public double ui_apply_ms { get; init; }
    public double paint_delay_ms { get; init; }
    public double refresh_to_render_ms { get; init; }
}
internal sealed class PeerReport
{
    public string status { get; init; } = "";
    public string mode { get; init; } = "";
    public string execution_mode { get; init; } = RuntimeFeature.IsDynamicCodeSupported ? "managed" : "native-aot";
    public string? executable { get; init; } = Environment.ProcessPath;
    public int process_id { get; init; } = Environment.ProcessId;
    public int? rust_process_id { get; init; }
    public ulong? remaining_charge { get; init; }
    public string path { get; init; } = "";
    public bool cancel_checked { get; init; }
    public bool native_received { get; init; }
    public string dialog_input { get; init; } = "programmatic Button.Click routed event";
    public bool os_keyboard_input { get; init; }
}
[JsonSourceGenerationOptions(WriteIndented = true, DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull)]
[JsonSerializable(typeof(DesktopReport))]
[JsonSerializable(typeof(PeerReport))]
internal partial class ReportJsonContext : JsonSerializerContext;
