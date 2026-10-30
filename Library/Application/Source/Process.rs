// Process-wide setup, before a program makes its window.

use windows_api::*;
use windows_core::Result;

pub fn prepare_process() -> Result<()> {
    unsafe {
        // So that GetDpiForWindow says the monitor's real DPI, which the
        // window check needs, instead of 96 in a window that Windows stretches.
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2).ok()?;
        // Windows lowers the quality of service of a process whose windows
        // aren't in focus, which on a CPU with efficiency cores can move its
        // threads onto those, at lower clock speeds. Programs ignore input
        // without focus, but keep drawing, so opting out keeps their frames
        // paced the same in the background, like while a coding agent takes
        // a screenshot, as in the foreground.
        let throttling = PROCESS_POWER_THROTTLING_STATE {
            Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION as u32,
            ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED as u32,
            StateMask: 0,
        };
        SetProcessInformation(
            GetCurrentProcess(),
            ProcessPowerThrottling,
            (&raw const throttling).cast(),
            size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        )
        .ok()
    }
}
