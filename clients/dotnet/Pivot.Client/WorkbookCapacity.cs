namespace Pivot.Client;

/// <summary>Production workbook bounds. Session tests verify Rust/C# agreement.</summary>
public static class WorkbookCapacity
{
    public const int MaximumRows = 65_536;
    public const int MaximumColumns = 1_024;
    public const int PositionedCells = 65_536;
    public const int AllocatedIdentities = 262_144;
    public const int StyledPositions = 4_096;
}
