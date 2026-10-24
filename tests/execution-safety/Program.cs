// Links the actual pre-Avalonia guard. No Avalonia references or GUI initialization.
return Pivot.Desktop.DesktopExecutionPolicy.CheckBeforeStartup() ? 0 : 2;
