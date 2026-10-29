namespace DisplayIdentification;

// Changes the primary display. Windows makes the display at desktop position (0, 0) primary, so the whole arrangement is shifted to
// put the new primary display there. The displays keep their arrangement relative to each other.
internal static unsafe class PrimaryDisplay {
    // With persist, the change is saved like a change in Settings. Without it, the change lasts until Windows restarts or the
    // displays change, which is safer for a temporary change.
    internal static void Set(Display display, bool persist) {
        DisplayConfiguration configuration = CreateConfiguration(display);
        uint flags = NativeMethods.SDC_APPLY | NativeMethods.SDC_USE_SUPPLIED_DISPLAY_CONFIG | NativeMethods.SDC_ALLOW_CHANGES;
        if (persist) {
            flags |= NativeMethods.SDC_SAVE_TO_DATABASE;
        }

        Apply(configuration, flags);
    }

    // Asks Windows whether it would accept the configuration without applying it.
    internal static void Validate(Display display) {
        Apply(CreateConfiguration(display), NativeMethods.SDC_VALIDATE | NativeMethods.SDC_USE_SUPPLIED_DISPLAY_CONFIG);
    }

    private static DisplayConfiguration CreateConfiguration(Display display) {
        DisplayConfiguration configuration = DisplayConfiguration.QueryActive();
        NativeMethods.DisplayConfigPathInfo[] paths = configuration.Paths;
        NativeMethods.DisplayConfigModeInfo[] modes = configuration.Modes;
        int pathIndex = Array.FindIndex(paths, path => path.SourceInfo.AdapterId.Matches(display.SourceAdapterId) && path.SourceInfo.Id == display.SourceId);
        if (pathIndex < 0) {
            throw new CommandException($"The display {display.Key} is no longer active.");
        }

        uint modeIndex = paths[pathIndex].SourceInfo.ModeInfoIndex;
        if (modeIndex >= modes.Length || modes[modeIndex].InfoType != NativeMethods.DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE) {
            throw new CommandException($"Windows didn't report the desktop position of the display {display.Key}.");
        }

        NativeMethods.PointL origin = modes[modeIndex].SourceMode.Position;
        for (int index = 0; index < modes.Length; index++) {
            if (modes[index].InfoType == NativeMethods.DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE) {
                modes[index].SourceMode.Position.X -= origin.X;
                modes[index].SourceMode.Position.Y -= origin.Y;
            }
        }

        return configuration;
    }

    private static void Apply(DisplayConfiguration configuration, uint flags) {
        int result;
        fixed (NativeMethods.DisplayConfigPathInfo* paths = configuration.Paths) {
            fixed (NativeMethods.DisplayConfigModeInfo* modes = configuration.Modes) {
                result = NativeMethods.SetDisplayConfig((uint)configuration.Paths.Length, paths, (uint)configuration.Modes.Length, modes, flags);
            }
        }

        NativeMethods.ThrowIfFailed("SetDisplayConfig", result);
    }
}
