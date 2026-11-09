// The GPU: its adapter, device, command queue and fence, and a command list
// for setup work, like copying meshes into GPU memory.

use windows_api::*;
use windows_core::{BOOL, Error, Interface, Result};

// Which GPU draws.
#[derive(Clone, Copy, Debug)]
pub enum AdapterChoice {
    // The GPU that the monitor is an output of. On a laptop whose screen is
    // wired to the integrated GPU, drawing with the discrete one would add a
    // copy between the GPUs to every frame, and keep Windows from flipping
    // frames straight to the screen.
    Monitor(HMONITOR),
    // The fastest hardware GPU, or WARP if there's none, like for tests.
    HighPerformance,
    // Microsoft's software rasterizer.
    Warp,
}

// Where the GPU's scheduler puts the process's work among other processes'.
// Windows' job objects can't limit the GPU, so a process that must stay out
// of a game's way lowers its own priority: RunLightly asks for Idle through
// KGAMES_GPU_PRIORITY, which Gpu::new reads, for coding agents' tests and
// test runs while someone plays a game, whose frames then go first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuPriority {
    Idle,
    BelowNormal,
    Normal,
    AboveNormal,
    High,
}

impl GpuPriority {
    // The priority of the name, like "Idle".
    pub fn from_name(name: &str) -> Option<GpuPriority> {
        match name {
            "Idle" => Some(GpuPriority::Idle),
            "BelowNormal" => Some(GpuPriority::BelowNormal),
            "Normal" => Some(GpuPriority::Normal),
            "AboveNormal" => Some(GpuPriority::AboveNormal),
            "High" => Some(GpuPriority::High),
            _ => None,
        }
    }

    // What KGAMES_GPU_PRIORITY asks for, or Normal when it's unset.
    pub fn from_environment() -> Result<GpuPriority> {
        match std::env::var("KGAMES_GPU_PRIORITY") {
            Err(_) => Ok(GpuPriority::Normal),
            Ok(value) => GpuPriority::from_name(&value).ok_or_else(|| {
                Error::new(
                    E_INVALIDARG,
                    format!("KGAMES_GPU_PRIORITY must be Idle, BelowNormal, Normal, AboveNormal or High, not {value}"),
                )
            }),
        }
    }

    fn class(self) -> D3DKMT_SCHEDULINGPRIORITYCLASS {
        match self {
            GpuPriority::Idle => D3DKMT_SCHEDULINGPRIORITYCLASS_IDLE,
            GpuPriority::BelowNormal => D3DKMT_SCHEDULINGPRIORITYCLASS_BELOW_NORMAL,
            GpuPriority::Normal => D3DKMT_SCHEDULINGPRIORITYCLASS_NORMAL,
            GpuPriority::AboveNormal => D3DKMT_SCHEDULINGPRIORITYCLASS_ABOVE_NORMAL,
            GpuPriority::High => D3DKMT_SCHEDULINGPRIORITYCLASS_HIGH,
        }
    }
}

// Sets where the GPU's scheduler puts this process's work, for every device
// it has and makes. Raising it above Normal needs a privilege that only
// administrators have.
pub fn set_gpu_priority(priority: GpuPriority) -> Result<()> {
    unsafe { D3DKMTSetProcessSchedulingPriorityClass(GetCurrentProcess(), priority.class()) }.ok()
}

// Where the GPU's scheduler puts this process's work now.
pub fn gpu_priority() -> Result<GpuPriority> {
    let mut class = D3DKMT_SCHEDULINGPRIORITYCLASS_NORMAL;
    unsafe { D3DKMTGetProcessSchedulingPriorityClass(GetCurrentProcess(), &mut class) }.ok()?;
    Ok([GpuPriority::Idle, GpuPriority::BelowNormal, GpuPriority::Normal, GpuPriority::AboveNormal, GpuPriority::High]
        .into_iter()
        .find(|priority| priority.class() == class)
        .unwrap_or(GpuPriority::Normal))
}

// What a GPU memory segment holds for this process: the bytes of it that the
// process has reserved, and the bytes that the GPU's scheduler is letting it
// have, which depends on what else is running.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VideoMemory {
    pub used: u64,
    pub budget: u64,
}

pub struct Gpu {
    pub factory: IDXGIFactory6,
    pub adapter: IDXGIAdapter1,
    // Like "NVIDIA GeForce RTX 5090 Laptop GPU".
    pub name: String,
    pub device: ID3D12Device,
    pub queue: ID3D12CommandQueue,
    // Whether presenting may tear, which needs both the display driver's and
    // Windows' support.
    pub tearing_supported: bool,
    fence: ID3D12Fence,
    fence_event: HANDLE,
    last_signal: u64,
    setup_allocator: ID3D12CommandAllocator,
    setup_list: ID3D12GraphicsCommandList,
    // With the debug layer, which only debug builds turn on.
    info_queue: Option<ID3D12InfoQueue>,
}

impl Gpu {
    pub fn new(choice: AdapterChoice) -> Result<Self> {
        // Before the device, so that the GPU's scheduler has the priority
        // for all of the process's devices.
        let priority = GpuPriority::from_environment()?;
        if priority != GpuPriority::Normal {
            set_gpu_priority(priority)?;
        }
        unsafe {
            let debug = cfg!(debug_assertions) && enable_debug_layer();
            let flags = if debug { DXGI_CREATE_FACTORY_DEBUG as u32 } else { 0 };
            let factory: IDXGIFactory6 = CreateDXGIFactory2(flags)?;
            let (adapter, device) = create_device(&factory, choice)?;
            let mut description = DXGI_ADAPTER_DESC1::default();
            adapter.GetDesc1(&mut description).ok()?;
            let name = String::from_utf16_lossy(&description.Description)
                .trim_end_matches('\0')
                .to_string();

            let queue: ID3D12CommandQueue = device.CreateCommandQueue(&D3D12_COMMAND_QUEUE_DESC {
                Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
                ..Default::default()
            })?;
            let fence: ID3D12Fence = device.CreateFence(0, D3D12_FENCE_FLAG_NONE)?;
            let fence_event = CreateEventW(None, false, false, None);
            check_handle(fence_event.0)?;
            let setup_allocator: ID3D12CommandAllocator =
                device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)?;
            let setup_list: ID3D12GraphicsCommandList =
                device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &setup_allocator, None)?;
            setup_list.Close().ok()?;

            let mut tearing = BOOL(0);
            let tearing_supported = factory
                .CheckFeatureSupport(
                    DXGI_FEATURE_PRESENT_ALLOW_TEARING,
                    &mut tearing as *mut BOOL as *mut _,
                    size_of::<BOOL>() as u32,
                )
                .is_ok()
                && tearing.as_bool();

            let info_queue = if debug { device.cast::<ID3D12InfoQueue>().ok() } else { None };
            Ok(Self {
                factory,
                adapter,
                name,
                device,
                queue,
                tearing_supported,
                fence,
                fence_event,
                last_signal: 0,
                setup_allocator,
                setup_list,
                info_queue,
            })
        }
    }

    // How much GPU memory this process has reserved, and how much the GPU's
    // scheduler is letting it have, for both of the GPU's segments: local,
    // which on a discrete GPU is its own memory, and non-local, which is
    // system memory that it reads across PCIe. Every committed resource
    // counts, and so does what the driver allocates for the process.
    pub fn video_memory(&self) -> Result<[VideoMemory; 2]> {
        let adapter = self.adapter.cast::<IDXGIAdapter3>()?;
        let mut segments = [VideoMemory::default(); 2];
        for (segment, group) in [DXGI_MEMORY_SEGMENT_GROUP_LOCAL, DXGI_MEMORY_SEGMENT_GROUP_NON_LOCAL].into_iter().enumerate() {
            let mut info = DXGI_QUERY_VIDEO_MEMORY_INFO::default();
            unsafe { adapter.QueryVideoMemoryInfo(0, group, &mut info).ok()? };
            segments[segment] = VideoMemory {
                used: info.CurrentUsage,
                budget: info.Budget,
            };
        }
        Ok(segments)
    }

    // The fence that `signal` has the queue signal, for what waits on its
    // own, like the presenter as it goes.
    pub(crate) fn fence(&self) -> ID3D12Fence {
        self.fence.clone()
    }

    // Has the queue signal the fence once the GPU finishes all the work
    // submitted so far, and returns the value that it will signal.
    pub fn signal(&mut self) -> Result<u64> {
        self.last_signal += 1;
        unsafe { self.queue.Signal(&self.fence, self.last_signal).ok()? };
        Ok(self.last_signal)
    }

    // The newest value that the GPU has signaled.
    pub fn completed(&self) -> u64 {
        unsafe { self.fence.GetCompletedValue() }
    }

    pub fn is_complete(&self, value: u64) -> bool {
        self.completed() >= value
    }

    // Blocks until the GPU has signaled the value.
    pub fn wait_for(&self, value: u64) -> Result<()> {
        if self.is_complete(value) {
            return Ok(());
        }
        unsafe {
            self.fence.SetEventOnCompletion(value, self.fence_event).ok()?;
            WaitForSingleObject(self.fence_event, INFINITE);
        }
        Ok(())
    }

    // Blocks until the GPU has finished everything submitted so far.
    pub fn flush(&mut self) -> Result<()> {
        let value = self.signal()?;
        self.wait_for(value)
    }

    // Records commands with `record`, runs them, and waits for the GPU to
    // finish them, so that whatever they read, like an upload buffer, can go
    // right after. For setup, not for frames.
    pub fn run(&mut self, record: impl FnOnce(&ID3D12GraphicsCommandList)) -> Result<()> {
        unsafe {
            self.setup_allocator.Reset().ok()?;
            self.setup_list.Reset(&self.setup_allocator, None).ok()?;
            record(&self.setup_list);
            self.setup_list.Close().ok()?;
            self.queue
                .ExecuteCommandLists(&[Some(self.setup_list.cast::<ID3D12CommandList>()?)]);
        }
        self.flush()?;
        self.report_messages();
        Ok(())
    }

    // Writes what the debug layer reported since the last call to standard
    // error, and returns how many were errors. Without the debug layer, it
    // has nothing to report.
    pub fn report_messages(&self) -> usize {
        let Some(queue) = &self.info_queue else {
            return 0;
        };
        let mut errors = 0;
        unsafe {
            for index in 0..queue.GetNumStoredMessagesAllowedByRetrievalFilter() {
                let mut length = 0;
                if queue.GetMessage(index, None, &mut length).is_err() || length == 0 {
                    continue;
                }
                // D3D12_MESSAGE is followed by its description in one allocation.
                let mut bytes = vec![0u64; length.div_ceil(8)];
                let message = bytes.as_mut_ptr() as *mut D3D12_MESSAGE;
                if queue.GetMessage(index, Some(message), &mut length).is_err() {
                    continue;
                }
                let message = &*message;
                let text = std::slice::from_raw_parts(
                    message.pDescription as *const u8,
                    message.DescriptionByteLength.saturating_sub(1),
                );
                let severity = match message.Severity {
                    D3D12_MESSAGE_SEVERITY_CORRUPTION => "corruption",
                    D3D12_MESSAGE_SEVERITY_ERROR => "error",
                    D3D12_MESSAGE_SEVERITY_WARNING => "warning",
                    D3D12_MESSAGE_SEVERITY_INFO => "info",
                    _ => "message",
                };
                if message.Severity <= D3D12_MESSAGE_SEVERITY_ERROR {
                    errors += 1;
                }
                eprintln!("D3D12 {severity}: {}", String::from_utf8_lossy(text));
            }
            queue.ClearStoredMessages();
        }
        errors
    }
}

impl Drop for Gpu {
    fn drop(&mut self) {
        // Nothing may still use what the GPU's objects hold.
        let _ = self.flush();
        unsafe {
            let _ = CloseHandle(self.fence_event);
        }
    }
}

// Turns on the D3D12 debug layer, if Windows' optional "Graphics Tools"
// feature is installed.
unsafe fn enable_debug_layer() -> bool {
    unsafe {
        let mut debug: Option<ID3D12Debug> = None;
        if D3D12GetDebugInterface(&mut debug).is_ok()
            && let Some(debug) = debug
        {
            debug.EnableDebugLayer();
            return true;
        }
        false
    }
}

unsafe fn create_device(
    factory: &IDXGIFactory6,
    choice: AdapterChoice,
) -> Result<(IDXGIAdapter1, ID3D12Device)> {
    unsafe {
        let mut hardware = Vec::new();
        let mut index = 0;
        while let Ok(adapter) = factory
            .EnumAdapterByGpuPreference::<IDXGIAdapter1>(index, DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE)
        {
            index += 1;
            let mut description = DXGI_ADAPTER_DESC1::default();
            if adapter.GetDesc1(&mut description).is_err()
                || description.Flags & 2 /* DXGI_ADAPTER_FLAG_SOFTWARE */ != 0
            {
                continue;
            }
            hardware.push(adapter);
        }

        let mut candidates: Vec<IDXGIAdapter1> = Vec::new();
        match choice {
            AdapterChoice::Monitor(monitor) => {
                // The monitor's own GPU first, then the others.
                let (outputs_monitor, others): (Vec<_>, Vec<_>) = hardware
                    .into_iter()
                    .partition(|adapter| drives_monitor(adapter, monitor));
                candidates.extend(outputs_monitor);
                candidates.extend(others);
            }
            AdapterChoice::HighPerformance => candidates.extend(hardware),
            AdapterChoice::Warp => {}
        }
        candidates.push(factory.EnumWarpAdapter()?);

        let mut last_error = Error::new(E_FAIL, "no GPU supports Direct3D 12");
        for adapter in candidates {
            let mut device: Option<ID3D12Device> = None;
            match D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_11_0, &mut device) {
                Ok(()) => {
                    if let Some(device) = device {
                        return Ok((adapter, device));
                    }
                }
                Err(error) => last_error = error,
            }
        }
        Err(last_error)
    }
}

unsafe fn drives_monitor(adapter: &IDXGIAdapter1, monitor: HMONITOR) -> bool {
    unsafe {
        let mut index = 0;
        loop {
            let mut output: Option<IDXGIOutput> = None;
            if adapter.EnumOutputs(index, &mut output).is_err() {
                return false;
            }
            index += 1;
            let Some(output) = output else {
                return false;
            };
            let mut description = DXGI_OUTPUT_DESC::default();
            if output.GetDesc(&mut description).is_ok() && description.Monitor == monitor {
                return true;
            }
        }
    }
}
