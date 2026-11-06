// A window's swap chain, paced for the lowest input latency:
//
// - It has a frame latency waitable object and a maximum frame latency of 1,
//   and only one frame is on the GPU at a time, so no frame waits behind
//   another in DXGI's queue.
// - `poll_ready` says, without blocking, whether the GPU has finished the
//   previous frame and DXGI can take another. The main loop polls it while it
//   handles window messages, and reads input only once it's ready, so no input
//   ages during the wait.
// - Frames present without waiting for vertical sync. With tearing allowed,
//   a frame can reach the screen mid-refresh, when Windows flips it straight
//   there, like in borderless fullscreen.

use windows_api::*;
use windows_core::{Interface, Result};

use crate::resources::{DescriptorHeap, command_list, create_target_views, execute, transition};
use crate::{Frame, Gpu, TARGET_FORMAT};

const BUFFER_COUNT: u32 = 2;

pub struct Presenter {
    swap_chain: IDXGISwapChain3,
    waitable: HANDLE,
    flags: u32,
    tearing: bool,
    buffers: Vec<ID3D12Resource>,
    views: DescriptorHeap,
    allocator: ID3D12CommandAllocator,
    list: ID3D12GraphicsCommandList,
    width: u32,
    height: u32,
    // What the fence reaches once the GPU finishes the last frame presented.
    last_frame: u64,
    // Whether the waitable object was signaled since the last frame began.
    // Waiting on it takes the signal, so it's remembered until the frame.
    waitable_signaled: bool,
    // The performance counter right before the last Present call.
    present_time: i64,
    // Whether frames wait for vertical sync, like in test runs.
    vsync: bool,
}

impl Presenter {
    // With `allow_tearing` false, frames still don't wait for vertical sync,
    // but the screen takes the newest one at each refresh, so none tear.
    pub fn new(gpu: &Gpu, window: HWND, width: u32, height: u32, allow_tearing: bool) -> Result<Self> {
        unsafe {
            let tearing = allow_tearing && gpu.tearing_supported;
            let mut flags = DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT as u32;
            if gpu.tearing_supported {
                flags |= DXGI_SWAP_CHAIN_FLAG_ALLOW_TEARING as u32;
            }
            let description = DXGI_SWAP_CHAIN_DESC1 {
                Width: width.max(1),
                Height: height.max(1),
                Format: TARGET_FORMAT,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                BufferUsage: DXGI_USAGE(DXGI_USAGE_RENDER_TARGET_OUTPUT),
                BufferCount: BUFFER_COUNT,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                Flags: flags,
                ..Default::default()
            };
            let swap_chain: IDXGISwapChain3 = gpu
                .factory
                .CreateSwapChainForHwnd(&gpu.queue, window, &description, None, None)?
                .cast()?;
            // Alt+Enter would switch to exclusive fullscreen, which the
            // programs don't use: F11 switches to borderless fullscreen.
            gpu.factory.MakeWindowAssociation(window, DXGI_MWA_NO_ALT_ENTER as u32).ok()?;
            swap_chain.SetMaximumFrameLatency(1).ok()?;
            let waitable = swap_chain.GetFrameLatencyWaitableObject();
            let views = DescriptorHeap::new(&gpu.device, D3D12_DESCRIPTOR_HEAP_TYPE_RTV, BUFFER_COUNT * 2, false)?;
            let (allocator, list) = command_list(&gpu.device)?;
            let mut presenter = Self {
                swap_chain,
                waitable,
                flags,
                tearing,
                buffers: Vec::new(),
                views,
                allocator,
                list,
                width: description.Width,
                height: description.Height,
                last_frame: 0,
                waitable_signaled: false,
                present_time: 0,
                vsync: false,
            };
            presenter.create_views(gpu)?;
            Ok(presenter)
        }
    }

    // Whether frames may tear.
    pub fn tearing(&self) -> bool {
        self.tearing && !self.vsync
    }

    // Makes frames wait for vertical sync, which paces them at the display's
    // refresh rate instead of as fast as the GPU goes, like for test runs, so
    // that they don't keep the GPU busy. They don't tear then.
    pub fn set_vsync(&mut self, vsync: bool) {
        self.vsync = vsync;
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
                .ResizeBuffers(BUFFER_COUNT, width, height, TARGET_FORMAT, self.flags)
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
    // that the GPU signals once it has drawn the frame.
    pub fn present(&mut self, gpu: &mut Gpu, frame: Frame) -> Result<u64> {
        unsafe {
            frame.list.ResourceBarrier(&[transition(
                &frame.target,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
                D3D12_RESOURCE_STATE_PRESENT,
            )]);
            frame.list.Close().ok()?;
            execute(&gpu.queue, &frame.list)?;
            let (interval, flags) = match (self.vsync, self.tearing) {
                (true, _) => (1, 0),
                (false, true) => (0, DXGI_PRESENT_ALLOW_TEARING),
                (false, false) => (0, 0),
            };
            let _ = QueryPerformanceCounter(&mut self.present_time);
            // DXGI_STATUS_OCCLUDED and the like are successes that say the
            // frame isn't visible, which changes nothing here.
            self.swap_chain.Present(interval, flags).ok()?;
        }
        self.last_frame = gpu.signal()?;
        Ok(self.last_frame)
    }
}

impl Drop for Presenter {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.waitable);
        }
    }
}
