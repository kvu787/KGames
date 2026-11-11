// A window's swap chain, paced for the lowest input latency:
//
// - It has a frame latency waitable object and a maximum frame latency of 1,
//   and only one frame is on the GPU at a time, so no frame waits behind
//   another in DXGI's queue.
// - `poll_ready` says, without blocking, whether the GPU has finished the
//   previous frame and DXGI can take another. The main loop polls it while it
//   handles window messages, and reads input only once it's ready, so no input
//   ages during the wait.
// - With vertical sync on, as a presenter starts, frames wait for the
//   screen's refresh, and never tear, so a program draws no faster than the
//   screen shows.
// - With it off, frames present without waiting, and may tear: a frame can
//   reach the screen mid-refresh when Windows flips it straight there, like
//   in borderless fullscreen. There's no way between the two, of frames that
//   don't wait and don't tear either.

use windows_api::*;
use windows_core::{BOOL, Error, Interface, Result};

use crate::resources::{DescriptorHeap, command_list, create_target_views, execute, transition};
use crate::{Frame, Gpu, TARGET_FORMAT, check_handle};

const BUFFER_COUNT: u32 = 2;

// The swap chain's flags, which ResizeBuffers must be given again.
const FLAGS: u32 = DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT as u32 | DXGI_SWAP_CHAIN_FLAG_ALLOW_TEARING as u32;

pub struct Presenter {
    swap_chain: IDXGISwapChain3,
    waitable: HANDLE,
    buffers: Vec<ID3D12Resource>,
    views: DescriptorHeap,
    allocator: ID3D12CommandAllocator,
    list: ID3D12GraphicsCommandList,
    width: u32,
    height: u32,
    // The GPU's fence, and an event to wait on it with, so that the swap
    // chain and its buffers go only once the GPU is done with the last
    // frame, however the main loop ends.
    fence: ID3D12Fence,
    fence_event: HANDLE,
    // What the fence reaches once the GPU finishes the last frame presented.
    last_frame: u64,
    // Whether the waitable object was signaled since the last frame began.
    // Waiting on it takes the signal, so it's remembered until the frame.
    waitable_signaled: bool,
    // The performance counter right before the last Present call.
    present_time: i64,
    // Whether frames wait for vertical sync, which `set_vsync` switches.
    vsync: bool,
}

impl Presenter {
    // Fails where Windows can't present frames that tear, which frames
    // without vertical sync need.
    pub fn new(gpu: &Gpu, window: HWND, width: u32, height: u32) -> Result<Self> {
        unsafe {
            if !tearing_supported(&gpu.factory) {
                return Err(Error::new(
                    E_FAIL,
                    "Windows can't present frames that tear here, which frames without vertical sync need.",
                ));
            }
            let description = DXGI_SWAP_CHAIN_DESC1 {
                Width: width.max(1),
                Height: height.max(1),
                Format: TARGET_FORMAT,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                BufferUsage: DXGI_USAGE(DXGI_USAGE_RENDER_TARGET_OUTPUT),
                BufferCount: BUFFER_COUNT,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                Flags: FLAGS,
                ..Default::default()
            };
            let swap_chain: IDXGISwapChain3 = gpu
                .factory
                .CreateSwapChainForHwnd(&gpu.queue, window, &description, None, None)?
                .cast()?;
            // DXGI would otherwise watch the window's messages, and switch it
            // to fullscreen with SetFullscreenState on Alt+Enter, which
            // changes the display's mode, as Direct3D 12 has no exclusive
            // fullscreen. The programs handle their window themselves: F11
            // switches to borderless fullscreen.
            gpu.factory
                .MakeWindowAssociation(window, (DXGI_MWA_NO_WINDOW_CHANGES | DXGI_MWA_NO_ALT_ENTER) as u32)
                .ok()?;
            swap_chain.SetMaximumFrameLatency(1).ok()?;
            let waitable = swap_chain.GetFrameLatencyWaitableObject();
            let views = DescriptorHeap::new(&gpu.device, D3D12_DESCRIPTOR_HEAP_TYPE_RTV, BUFFER_COUNT * 2, false)?;
            let (allocator, list) = command_list(&gpu.device)?;
            let fence_event = CreateEventW(None, false, false, None);
            check_handle(fence_event.0)?;
            let mut presenter = Self {
                swap_chain,
                waitable,
                buffers: Vec::new(),
                views,
                allocator,
                list,
                width: description.Width,
                height: description.Height,
                fence: gpu.fence(),
                fence_event,
                last_frame: 0,
                waitable_signaled: false,
                present_time: 0,
                vsync: true,
            };
            presenter.create_views(gpu)?;
            Ok(presenter)
        }
    }

    // Makes frames wait for vertical sync, which paces them at the display's
    // refresh rate instead of as fast as the GPU goes, so that they don't
    // keep the GPU busy, and they don't tear. A presenter starts with it on.
    // Off, frames present at once, and may tear.
    pub fn set_vsync(&mut self, vsync: bool) {
        self.vsync = vsync;
    }

    // Whether frames wait for vertical sync.
    pub fn vsync(&self) -> bool {
        self.vsync
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    // Makes the back buffers the window's new size. Waits for the GPU.
    pub fn resize(&mut self, gpu: &mut Gpu, width: u32, height: u32) -> Result<()> {
        let (width, height) = (width.max(1), height.max(1));
        if (width, height) == (self.width, self.height) {
            return Ok(());
        }
        gpu.flush()?;
        self.buffers.clear();
        unsafe {
            self.swap_chain
                .ResizeBuffers(BUFFER_COUNT, width, height, TARGET_FORMAT, FLAGS)
                .ok()?;
        }
        self.width = width;
        self.height = height;
        self.create_views(gpu)
    }

    fn create_views(&mut self, gpu: &Gpu) -> Result<()> {
        unsafe {
            for index in 0..BUFFER_COUNT {
                let buffer: ID3D12Resource = self.swap_chain.GetBuffer(index)?;
                create_target_views(&gpu.device, &buffer, &self.views, index as usize * 2);
                self.buffers.push(buffer);
            }
        }
        Ok(())
    }

    // Whether a frame can begin now: the GPU has finished the last one, and
    // DXGI's queue has room. Never blocks.
    pub fn poll_ready(&mut self, gpu: &Gpu) -> bool {
        if !self.waitable_signaled {
            self.waitable_signaled = unsafe { WaitForSingleObject(self.waitable, 0) } == WAIT_OBJECT_0 as u32;
        }
        self.waitable_signaled && gpu.is_complete(self.last_frame)
    }

    // Waits, without spinning, until a frame can begin, a window message
    // comes, or the milliseconds pass. With vertical sync, a frame can't
    // begin before the last one has gone to the screen anyway, so waiting
    // instead of polling costs only the thread's wake-up.
    pub fn wait_ready(&mut self, gpu: &Gpu, milliseconds: u32) -> Result<()> {
        gpu.wait_for(self.last_frame)?;
        if !self.waitable_signaled {
            let result = unsafe { MsgWaitForMultipleObjects(Some(&[self.waitable]), false, milliseconds, QS_ALLINPUT as u32) };
            self.waitable_signaled = result == WAIT_OBJECT_0 as u32;
        }
        Ok(())
    }

    // Begins a frame in the back buffer, once poll_ready says so. The back
    // buffer is in the render target state.
    pub fn begin(&mut self) -> Result<Frame> {
        self.waitable_signaled = false;
        unsafe {
            self.allocator.Reset().ok()?;
            self.list.Reset(&self.allocator, None).ok()?;
            let index = self.swap_chain.GetCurrentBackBufferIndex() as usize;
            let target = self.buffers[index].clone();
            self.list.ResourceBarrier(&[transition(
                &target,
                D3D12_RESOURCE_STATE_PRESENT,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
            )]);
            Ok(Frame {
                list: self.list.clone(),
                target,
                view: self.views.cpu(index * 2),
                srgb_view: self.views.cpu(index * 2 + 1),
                width: self.width,
                height: self.height,
            })
        }
    }

    // The performance counter right before the last frame's Present call,
    // which input latency measures to and from.
    pub fn present_time(&self) -> i64 {
        self.present_time
    }

    // Submits the frame's commands and presents it. Returns the fence value
    // that the GPU signals once it has drawn the frame, which the presenter
    // waits for before it lets go of the frame's buffers, even when Present
    // fails.
    pub fn present(&mut self, gpu: &mut Gpu, frame: Frame) -> Result<u64> {
        unsafe {
            frame.list.ResourceBarrier(&[transition(
                &frame.target,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
                D3D12_RESOURCE_STATE_PRESENT,
            )]);
            frame.list.Close().ok()?;
            execute(&gpu.queue, &frame.list)?;
            let (interval, flags) = present_arguments(self.vsync);
            let _ = QueryPerformanceCounter(&mut self.present_time);
            let swap_chain = &self.swap_chain;
            // Present's DXGI_STATUS codes are successes, which change nothing
            // here. A flip model swap chain never returns DXGI_STATUS_OCCLUDED.
            let present = || swap_chain.Present(interval, flags).ok();
            present_then_signal(&mut self.last_frame, present, || gpu.signal()).map_err(|error| with_removed_reason(gpu, error))
        }
    }
}

// Present's sync interval and flags: with vertical sync, the frame waits for
// the screen's next refresh, and without it, it goes at once, and may tear.
pub fn present_arguments(vsync: bool) -> (u32, u32) {
    if vsync { (1, 0) } else { (0, DXGI_PRESENT_ALLOW_TEARING) }
}

// Whether Windows can present frames that tear, which both the display
// driver and Windows must support.
fn tearing_supported(factory: &IDXGIFactory6) -> bool {
    let mut supported = BOOL(0);
    let size = size_of::<BOOL>() as u32;
    let result = unsafe { factory.CheckFeatureSupport(DXGI_FEATURE_PRESENT_ALLOW_TEARING, &mut supported as *mut BOOL as *mut _, size) };
    result.is_ok() && supported.as_bool()
}

// Presents, then has the GPU signal the fence after the frame's commands,
// which are on the GPU already, and keeps the value in `last_frame`, whether
// the present succeeded or not, so that whatever lets go of the frame's
// resources waits for the value however the frame ended. Returns the value,
// or the present's error.
pub fn present_then_signal(
    last_frame: &mut u64,
    present: impl FnOnce() -> Result<()>,
    signal: impl FnOnce() -> Result<u64>,
) -> Result<u64> {
    let presented = present();
    *last_frame = signal()?;
    presented.map(|()| *last_frame)
}

// An error, with why the device was removed, if it was, which
// DXGI_ERROR_DEVICE_REMOVED and DXGI_ERROR_DEVICE_RESET don't say.
fn with_removed_reason(gpu: &Gpu, error: windows_core::Error) -> windows_core::Error {
    let reason = unsafe { gpu.device.GetDeviceRemovedReason() };
    if reason.is_ok() {
        return error;
    }
    windows_core::Error::new(error.code(), format!("{} The device was removed: {}", error.message(), windows_core::Error::from(reason)))
}

impl Drop for Presenter {
    fn drop(&mut self) {
        // The swap chain and its buffers go now, so the GPU must be done
        // with the last frame first, whichever way the main loop ended:
        // releasing what a frame still draws with makes its shaders fault,
        // which NVIDIA's driver logs, and which stutters every app on the
        // GPU.
        unsafe {
            if self.fence.GetCompletedValue() < self.last_frame
                && self.fence.SetEventOnCompletion(self.last_frame, self.fence_event).is_ok()
            {
                WaitForSingleObject(self.fence_event, INFINITE);
            }
            let _ = CloseHandle(self.fence_event);
            let _ = CloseHandle(self.waitable);
        }
    }
}
