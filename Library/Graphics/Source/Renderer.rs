// The Direct3D 12 renderer that every program's frames go through: the device,
// a flip model swap chain for the window, and the pacing of frames. Programs
// record their own draws into each frame, between is_ready and present.
//
// It's set up so that input reaches the screen as soon as possible: a frame
// only reads input once the previous one has left the GPU and DXGI's queue,
// so no finished frame waits behind another, and frames are presented
// without waiting for vertical sync.

use std::path::{Path, PathBuf};

use windows_api::*;
use windows_core::{Error, Interface, PCWSTR, Result};

use crate::{create_buffer, create_texture, save_png, transition};

// The fewest a flip model swap chain allows: one on screen, one being drawn.
const BUFFER_COUNT: u32 = 2;
// Flip model swap chains can't have an sRGB format, but their buffers can be
// viewed as sRGB: see srgb_render_target.
pub const BACK_BUFFER_FORMAT: DXGI_FORMAT = DXGI_FORMAT_R8G8B8A8_UNORM;
pub const BACK_BUFFER_SRGB_FORMAT: DXGI_FORMAT = DXGI_FORMAT_R8G8B8A8_UNORM_SRGB;
pub const DEPTH_FORMAT: DXGI_FORMAT = DXGI_FORMAT_D32_FLOAT;

pub struct Renderer {
    pub device: ID3D12Device,
    pub queue: ID3D12CommandQueue,
    // The debug layer's messages, in debug builds where it's installed.
    debug_messages: Option<ID3D12InfoQueue>,
    swap_chain: IDXGISwapChain3,
    // Two views of each back buffer: as stored, then as sRGB.
    rtv_heap: ID3D12DescriptorHeap,
    rtv_increment: usize,
    back_buffers: Vec<ID3D12Resource>,
    // A depth buffer as big as the back buffers, if the program asked for one.
    depth: Option<(ID3D12Resource, ID3D12DescriptorHeap)>,
    // One frame is in flight at a time, so one allocator is enough.
    allocator: ID3D12CommandAllocator,
    // The frame's commands. Programs record their draws into it.
    pub command_list: ID3D12GraphicsCommandList,
    fence: ID3D12Fence,
    fence_event: HANDLE,
    // Marks when the last submitted commands are finished.
    last_fence_value: u64,
    pub adapter_name: String,
    // Whether the adapter is the one the window's monitor is connected to.
    pub adapter_drives_monitor: bool,
    // Whether frames may tear, which lets them reach the screen mid-refresh,
    // and whether the program asked for that.
    pub tearing: bool,
    allow_tearing: bool,
    // ResizeBuffers must be given the flags the swap chain was created with.
    swap_chain_flags: u32,
    // Signaled when DXGI's queue of presented frames has room for another.
    frame_latency_event: HANDLE,
    // Whether frame_latency_event has been waited on since the last Present.
    queue_has_room: bool,
    // The back buffer that is_ready started recording the next frame for.
    recording: Option<usize>,
    // Where to save the frame being recorded, once it's presented.
    capture: Option<PathBuf>,
    pub width: u32,
    pub height: u32,
}

impl Renderer {
    // Frames may tear if `allow_tearing` and the GPU's driver allow it.
    // `depth` makes a depth buffer, for programs that draw in 3D.
    pub unsafe fn new(window: HWND, allow_tearing: bool, depth: bool) -> Result<Self> {
        unsafe {
            // The debug layer needs the "Graphics Tools" optional Windows feature.
            let mut factory_flags = 0;
            if cfg!(debug_assertions) {
                let mut debug: Option<ID3D12Debug> = None;
                if D3D12GetDebugInterface(&mut debug).is_ok() {
                    debug.unwrap().EnableDebugLayer();
                    factory_flags = DXGI_CREATE_FACTORY_DEBUG as u32;
                }
            }

            let factory: IDXGIFactory6 = CreateDXGIFactory2(factory_flags)?;
            // Draw with the GPU that the window's monitor is connected to.
            // Drawing with another, like the discrete GPU of a laptop whose
            // screen is wired to the integrated one, adds a copy between GPUs
            // to every frame, and keeps Windows from flipping frames straight
            // to the screen. If none reports the monitor, like over Remote
            // Desktop, use the high-performance one. The choice isn't
            // revisited if the window moves to a monitor on another GPU.
            let monitor = MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST as u32);
            let mut adapter: IDXGIAdapter1 =
                factory.EnumAdapterByGpuPreference(0, DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE)?;
            let mut adapter_drives_monitor = false;
            for index in 0.. {
                let Ok(candidate) = factory.EnumAdapterByGpuPreference::<IDXGIAdapter1>(
                    index,
                    DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE,
                ) else {
                    break;
                };
                if drives_monitor(&candidate, monitor) {
                    adapter = candidate;
                    adapter_drives_monitor = true;
                    break;
                }
            }
            let mut adapter_description = DXGI_ADAPTER_DESC1::default();
            adapter.GetDesc1(&mut adapter_description).ok()?;
            let name = &adapter_description.Description;
            let length = name
                .iter()
                .position(|&unit| unit == 0)
                .unwrap_or(name.len());
            let adapter_name = String::from_utf16_lossy(&name[..length]);

            let mut device: Option<ID3D12Device> = None;
            D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_12_0, &mut device)?;
            let device = device.unwrap();
            let debug_messages = if factory_flags != 0 {
                device.cast().ok()
            } else {
                None
            };

            let queue: ID3D12CommandQueue =
                device.CreateCommandQueue(&D3D12_COMMAND_QUEUE_DESC {
                    Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
                    ..Default::default()
                })?;

            // Tearing needs Windows 10 1809 or later, and a driver that allows it.
            let mut supported = windows_core::BOOL(0);
            let tearing = allow_tearing
                && factory
                    .cast::<IDXGIFactory5>()?
                    .CheckFeatureSupport(
                        DXGI_FEATURE_PRESENT_ALLOW_TEARING,
                        (&raw mut supported).cast(),
                        size_of::<windows_core::BOOL>() as u32,
                    )
                    .is_ok()
                && supported.as_bool();
            let mut swap_chain_flags = DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT as u32;
            if tearing {
                swap_chain_flags |= DXGI_SWAP_CHAIN_FLAG_ALLOW_TEARING as u32;
            }

            // Width and height 0 means "use the window's client size".
            let swap_chain_description = DXGI_SWAP_CHAIN_DESC1 {
                Format: BACK_BUFFER_FORMAT,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                BufferUsage: DXGI_USAGE(DXGI_USAGE_RENDER_TARGET_OUTPUT),
                BufferCount: BUFFER_COUNT,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                Flags: swap_chain_flags,
                ..Default::default()
            };
            let swap_chain: IDXGISwapChain3 = factory
                .CreateSwapChainForHwnd(&queue, window, &swap_chain_description, None, None)?
                .cast()?;
            factory
                .MakeWindowAssociation(window, DXGI_MWA_NO_ALT_ENTER as u32)
                .ok()?;
            let mut actual = DXGI_SWAP_CHAIN_DESC1::default();
            swap_chain.GetDesc1(&mut actual).ok()?;
            // DXGI otherwise lets 3 presented frames queue up before Present
            // blocks, and each one queued delays the newest by a refresh.
            swap_chain.SetMaximumFrameLatency(1).ok()?;
            let frame_latency_event = swap_chain.GetFrameLatencyWaitableObject();
            check_handle(frame_latency_event.0)?;

            let rtv_heap: ID3D12DescriptorHeap =
                device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                    Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
                    NumDescriptors: BUFFER_COUNT * 2,
                    ..Default::default()
                })?;
            let rtv_increment =
                device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV) as usize;
            let depth = if depth {
                let heap: ID3D12DescriptorHeap =
                    device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                        Type: D3D12_DESCRIPTOR_HEAP_TYPE_DSV,
                        NumDescriptors: 1,
                        ..Default::default()
                    })?;
                let buffer = create_depth_buffer(&device, &heap, actual.Width, actual.Height)?;
                Some((buffer, heap))
            } else {
                None
            };

            let allocator: ID3D12CommandAllocator =
                device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)?;
            let command_list: ID3D12GraphicsCommandList = device.CreateCommandList(
                0,
                D3D12_COMMAND_LIST_TYPE_DIRECT,
                &allocator,
                None,
            )?;
            command_list.Close().ok()?;

            let fence: ID3D12Fence = device.CreateFence(0, D3D12_FENCE_FLAG_NONE)?;
            let fence_event = CreateEventW(None, false, false, PCWSTR::null());
            check_handle(fence_event.0)?;

            let mut renderer = Self {
                device,
                queue,
                debug_messages,
                swap_chain,
                rtv_heap,
                rtv_increment,
                back_buffers: Vec::new(),
                depth,
                allocator,
                command_list,
                fence,
                fence_event,
                last_fence_value: 0,
                adapter_name,
                adapter_drives_monitor,
                tearing,
                allow_tearing,
                swap_chain_flags,
                frame_latency_event,
                queue_has_room: false,
                recording: None,
                capture: None,
                width: actual.Width,
                height: actual.Height,
            };
            renderer.create_render_target_views()?;
            Ok(renderer)
        }
    }

    // For the title bar, as both affect latency, like "NVIDIA GeForce RTX
    // 5090 Laptop GPU (the monitor's GPU), tearing allowed".
    pub fn description(&self) -> String {
        let mut description = self.adapter_name.clone();
        description += if self.adapter_drives_monitor {
            " (the monitor's GPU)"
        } else {
            " (no GPU reports the monitor)"
        };
        description += if self.tearing {
            ", tearing allowed"
        } else if self.allow_tearing {
            ", tearing not supported"
        } else {
            ", tearing off in the settings"
        };
        description
    }

    unsafe fn create_render_target_views(&mut self) -> Result<()> {
        unsafe {
            for index in 0..BUFFER_COUNT as usize {
                let buffer: ID3D12Resource = self.swap_chain.GetBuffer(index as u32)?;
                for (view, format) in [BACK_BUFFER_FORMAT, BACK_BUFFER_SRGB_FORMAT]
                    .into_iter()
                    .enumerate()
                {
                    let mut description = D3D12_RENDER_TARGET_VIEW_DESC {
                        Format: format,
                        ViewDimension: D3D12_RTV_DIMENSION_TEXTURE2D,
                        ..Default::default()
                    };
                    description.Anonymous.Texture2D.MipSlice = 0;
                    self.device.CreateRenderTargetView(
                        &buffer,
                        Some(&description),
                        self.rtv_handle(index * 2 + view),
                    );
                }
                self.back_buffers.push(buffer);
            }
            Ok(())
        }
    }

    unsafe fn rtv_handle(&self, index: usize) -> D3D12_CPU_DESCRIPTOR_HANDLE {
        let start = unsafe { self.rtv_heap.GetCPUDescriptorHandleForHeapStart() };
        D3D12_CPU_DESCRIPTOR_HANDLE {
            ptr: start.ptr + index * self.rtv_increment,
        }
    }

    unsafe fn wait_for_fence(&self, value: u64) -> Result<()> {
        unsafe {
            if self.fence.GetCompletedValue() < value {
                self.fence
                    .SetEventOnCompletion(value, self.fence_event)
                    .ok()?;
                WaitForSingleObject(self.fence_event, INFINITE);
            }
            Ok(())
        }
    }

    unsafe fn wait_for_idle(&mut self) -> Result<()> {
        unsafe {
            self.last_fence_value += 1;
            self.queue.Signal(&self.fence, self.last_fence_value).ok()?;
            self.wait_for_fence(self.last_fence_value)
        }
    }

    // Waits until the GPU has finished every frame, so that the resources
    // they used can be released. Programs call it once their main loop ends,
    // as what they made for their draws is dropped before the renderer.
    pub fn wait_for_gpu(&mut self) -> Result<()> {
        unsafe {
            self.discard_frame()?;
            self.wait_for_idle()
        }
    }

    // Drops the frame that is_ready started recording, if any, so that the
    // command allocator can be reset and the back buffers released.
    unsafe fn discard_frame(&mut self) -> Result<()> {
        if self.recording.take().is_some() {
            unsafe { self.command_list.Close() }.ok()?;
        }
        Ok(())
    }

    pub unsafe fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        unsafe {
            if width == 0 || height == 0 || (width == self.width && height == self.height) {
                return Ok(());
            }
            // All references to the old back buffers must be gone before ResizeBuffers.
            self.discard_frame()?;
            self.wait_for_idle()?;
            self.back_buffers.clear();
            self.swap_chain
                .ResizeBuffers(
                    BUFFER_COUNT,
                    width,
                    height,
                    DXGI_FORMAT_UNKNOWN,
                    self.swap_chain_flags,
                )
                .ok()?;
            self.width = width;
            self.height = height;
            if let Some((buffer, heap)) = &mut self.depth {
                *buffer = create_depth_buffer(&self.device, heap, width, height)?;
            }
            self.create_render_target_views()
        }
    }

    // Records commands with the command list outside of a frame, like the
    // copy that uploads a texture, runs them, and waits for the GPU to finish.
    // It's for setup, not for frames.
    pub unsafe fn run(&mut self, record: impl FnOnce(&ID3D12GraphicsCommandList)) -> Result<()> {
        unsafe {
            self.discard_frame()?;
            self.wait_for_idle()?;
            self.allocator.Reset().ok()?;
            self.command_list.Reset(&self.allocator, None).ok()?;
            record(&self.command_list);
            self.command_list.Close().ok()?;
            self.queue
                .ExecuteCommandLists(&[Some(self.command_list.cast()?)]);
            self.wait_for_idle()
        }
    }

    // Whether a frame drawn now would reach the screen without waiting behind
    // an older one: the GPU has finished the previous frame, and DXGI's queue
    // has room. It doesn't block, so the caller can keep reading the newest
    // input until it's true, then draw with it.
    //
    // The GPU usually finishes tens of microseconds before DXGI's queue has
    // room. In between, this records the commands that don't depend on input,
    // so they aren't left for between reading input and presenting.
    pub unsafe fn is_ready(&mut self) -> Result<bool> {
        unsafe {
            if self.recording.is_none() {
                if self.fence.GetCompletedValue() < self.last_fence_value {
                    return Ok(false);
                }
                self.begin_frame()?;
            }
            if !self.queue_has_room {
                match WaitForSingleObject(self.frame_latency_event, 0) as i32 {
                    WAIT_OBJECT_0 => self.queue_has_room = true,
                    WAIT_TIMEOUT => {}
                    _ => return Err(Error::from_thread()),
                }
            }
            Ok(self.queue_has_room)
        }
    }

    // Starts recording a frame, with the back buffer as the render target.
    // The GPU must have finished with the command allocator.
    unsafe fn begin_frame(&mut self) -> Result<()> {
        unsafe {
            let index = self.swap_chain.GetCurrentBackBufferIndex() as usize;
            self.allocator.Reset().ok()?;
            self.command_list.Reset(&self.allocator, None).ok()?;
            self.command_list.ResourceBarrier(&[transition(
                &self.back_buffers[index],
                D3D12_RESOURCE_STATE_PRESENT,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
            )]);
            self.recording = Some(index);
            self.set_render_target(false, false);
            Ok(())
        }
    }

    fn frame_index(&self) -> usize {
        self.recording
            .expect("frames are recorded once is_ready is true, until present")
    }

    // The back buffer of the frame being recorded. It's in the render target
    // state, and must be again by present.
    pub fn back_buffer(&self) -> &ID3D12Resource {
        &self.back_buffers[self.frame_index()]
    }

    // The frame's back buffer, viewed as it's stored, for colors that are
    // already sRGB, like the theme's. Blending happens in sRGB.
    pub fn render_target(&self) -> D3D12_CPU_DESCRIPTOR_HANDLE {
        unsafe { self.rtv_handle(self.frame_index() * 2) }
    }

    // The frame's back buffer, viewed as sRGB, for shaders that output linear
    // colors. Writes are encoded to sRGB, and blending happens in linear.
    pub fn srgb_render_target(&self) -> D3D12_CPU_DESCRIPTOR_HANDLE {
        unsafe { self.rtv_handle(self.frame_index() * 2 + 1) }
    }

    pub fn depth_stencil(&self) -> D3D12_CPU_DESCRIPTOR_HANDLE {
        let (_, heap) = self
            .depth
            .as_ref()
            .expect("the renderer was made without a depth buffer");
        unsafe { heap.GetCPUDescriptorHandleForHeapStart() }
    }

    // Draws to the frame's back buffer, as stored or as sRGB, and to the
    // depth buffer if `depth`, all over the window.
    pub fn set_render_target(&self, srgb: bool, depth: bool) {
        let rtv = if srgb {
            self.srgb_render_target()
        } else {
            self.render_target()
        };
        let dsv = depth.then(|| self.depth_stencil());
        let list = &self.command_list;
        unsafe {
            list.OMSetRenderTargets(
                1,
                Some(&raw const rtv),
                false,
                dsv.as_ref().map(|dsv| dsv as *const _),
            );
            list.RSSetViewports(&[D3D12_VIEWPORT {
                Width: self.width as f32,
                Height: self.height as f32,
                MaxDepth: 1.0,
                ..Default::default()
            }]);
            list.RSSetScissorRects(&[D3D12_RECT {
                left: 0,
                top: 0,
                right: self.width as i32,
                bottom: self.height as i32,
            }]);
        }
    }

    // Fills the frame's back buffer with a color, which is stored as it is,
    // like the theme's background.
    pub fn clear(&self, color: [f32; 4]) {
        unsafe {
            self.command_list
                .ClearRenderTargetView(self.render_target(), &color, &[]);
        }
    }

    pub fn clear_depth(&self) {
        unsafe {
            self.command_list.ClearDepthStencilView(
                self.depth_stencil(),
                D3D12_CLEAR_FLAG_DEPTH,
                1.0,
                0,
                &[],
            );
        }
    }

    // Saves the frame being recorded as a PNG at `path`, as the window shows
    // it, once present has presented it. Present then waits for the GPU, so
    // it's for tests, like a script's checkpoints, not for every frame.
    pub fn capture(&mut self, path: &Path) -> Result<()> {
        self.capture = Some(path.to_path_buf());
        Ok(())
    }

    // Finishes the frame that is_ready started, and presents it.
    pub unsafe fn present(&mut self) -> Result<()> {
        unsafe {
            let index = self
                .recording
                .take()
                .expect("present is called once is_ready is true");
            let back_buffer = &self.back_buffers[index];
            let list = &self.command_list;
            // A capture copies the frame into memory the CPU can read.
            let capture = match self.capture.take() {
                Some(path) => {
                    let description = back_buffer.GetDesc();
                    let mut footprint = D3D12_PLACED_SUBRESOURCE_FOOTPRINT::default();
                    let mut size = 0u64;
                    self.device.GetCopyableFootprints(
                        &description,
                        0,
                        1,
                        0,
                        Some(&mut footprint),
                        None,
                        None,
                        Some(&mut size),
                    );
                    let readback = create_buffer(
                        &self.device,
                        size,
                        D3D12_HEAP_TYPE_READBACK,
                        D3D12_RESOURCE_STATE_COPY_DEST,
                    )?;
                    list.ResourceBarrier(&[transition(
                        back_buffer,
                        D3D12_RESOURCE_STATE_RENDER_TARGET,
                        D3D12_RESOURCE_STATE_COPY_SOURCE,
                    )]);
                    let destination = D3D12_TEXTURE_COPY_LOCATION {
                        pResource: std::mem::transmute_copy(&readback),
                        Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
                        Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                            PlacedFootprint: footprint,
                        },
                    };
                    let source = D3D12_TEXTURE_COPY_LOCATION {
                        pResource: std::mem::transmute_copy(back_buffer),
                        Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
                        Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                            SubresourceIndex: 0,
                        },
                    };
                    list.CopyTextureRegion(&destination, 0, 0, 0, &source, None);
                    list.ResourceBarrier(&[transition(
                        back_buffer,
                        D3D12_RESOURCE_STATE_COPY_SOURCE,
                        D3D12_RESOURCE_STATE_PRESENT,
                    )]);
                    Some((path, readback, footprint, size))
                }
                None => {
                    list.ResourceBarrier(&[transition(
                        back_buffer,
                        D3D12_RESOURCE_STATE_RENDER_TARGET,
                        D3D12_RESOURCE_STATE_PRESENT,
                    )]);
                    None
                }
            };
            list.Close().ok()?;

            self.queue.ExecuteCommandLists(&[Some(list.cast()?)]);
            // Sync interval 0: don't wait for vertical sync. With tearing
            // allowed, a frame can replace the one being scanned out mid-refresh
            // when Windows gives the window its own flips, like in fullscreen.
            // Otherwise the desktop compositor shows the newest frame at its
            // next refresh, and drops the ones it didn't get to.
            let flags = if self.tearing {
                DXGI_PRESENT_ALLOW_TEARING
            } else {
                0
            };
            self.swap_chain.Present(0, flags).ok()?;
            self.queue_has_room = false;

            self.last_fence_value += 1;
            self.queue.Signal(&self.fence, self.last_fence_value).ok()?;
            self.report_debug_messages();
            if let Some((path, readback, footprint, size)) = capture {
                self.wait_for_fence(self.last_fence_value)?;
                self.save_capture(&path, &readback, &footprint, size)?;
            }
            Ok(())
        }
    }

    // Writes a copied frame, whose bytes are red, green, blue and alpha, as
    // a PNG, without alpha, which the window doesn't show.
    unsafe fn save_capture(
        &self,
        path: &Path,
        readback: &ID3D12Resource,
        footprint: &D3D12_PLACED_SUBRESOURCE_FOOTPRINT,
        size: u64,
    ) -> Result<()> {
        unsafe {
            let (width, height) = (footprint.Footprint.Width, footprint.Footprint.Height);
            let mut mapped = std::ptr::null_mut();
            readback
                .Map(0, Some(&D3D12_RANGE { Begin: 0, End: size as usize }), &mut mapped)
                .ok()?;
            let bytes = std::slice::from_raw_parts(mapped as *const u8, size as usize);
            let mut pixels = Vec::with_capacity(width as usize * height as usize * 3);
            for row in 0..height as usize {
                let start = footprint.Offset as usize + row * footprint.Footprint.RowPitch as usize;
                for pixel in bytes[start..start + width as usize * 4].chunks_exact(4) {
                    pixels.extend([pixel[2], pixel[1], pixel[0]]);
                }
            }
            readback.Unmap(0, Some(&D3D12_RANGE { Begin: 0, End: 0 }));
            save_png(path, width, height, &pixels)
        }
    }

    // Writes the debug layer's warnings and errors to standard error, where
    // RunInBackground keeps them in a log, and forgets every message.
    fn report_debug_messages(&self) {
        let Some(messages) = &self.debug_messages else {
            return;
        };
        unsafe {
            for index in 0..messages.GetNumStoredMessages() {
                let mut size = 0;
                if messages.GetMessage(index, None, &mut size).is_err() {
                    continue;
                }
                // u64s, so the message is aligned for its pointer fields.
                let mut storage = vec![0u64; size.div_ceil(8)];
                let message = storage.as_mut_ptr() as *mut D3D12_MESSAGE;
                if messages.GetMessage(index, Some(message), &mut size).is_ok()
                    && (*message).Severity <= D3D12_MESSAGE_SEVERITY_WARNING
                {
                    let description = std::ffi::CStr::from_ptr((*message).pDescription);
                    eprintln!("D3D12: {}", description.to_string_lossy());
                }
            }
            messages.ClearStoredMessages();
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            let _ = self.wait_for_idle();
            let _ = CloseHandle(self.fence_event);
            let _ = CloseHandle(self.frame_latency_event);
        }
    }
}

// Whether one of the adapter's outputs is this monitor.
unsafe fn drives_monitor(adapter: &IDXGIAdapter1, monitor: HMONITOR) -> bool {
    (0..)
        .map_while(|index| {
            let mut output = None;
            unsafe { adapter.EnumOutputs(index, &mut output) }
                .ok()
                .ok()?;
            output
        })
        .any(|output| {
            let mut description = DXGI_OUTPUT_DESC::default();
            unsafe { output.GetDesc(&mut description) }.is_ok() && description.Monitor == monitor
        })
}

// Makes a depth buffer, and its view in the heap's only descriptor.
unsafe fn create_depth_buffer(
    device: &ID3D12Device,
    heap: &ID3D12DescriptorHeap,
    width: u32,
    height: u32,
) -> Result<ID3D12Resource> {
    unsafe {
        let mut clear = D3D12_CLEAR_VALUE {
            Format: DEPTH_FORMAT,
            ..Default::default()
        };
        clear.Anonymous.DepthStencil.Depth = 1.0;
        let buffer = create_texture(
            device,
            width,
            height,
            DEPTH_FORMAT,
            1,
            D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL,
            D3D12_RESOURCE_STATE_DEPTH_WRITE,
            Some(&clear),
        )?;
        device.CreateDepthStencilView(&buffer, None, heap.GetCPUDescriptorHandleForHeapStart());
        Ok(buffer)
    }
}
