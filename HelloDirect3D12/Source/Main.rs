// No console window in any build.
#![windows_subsystem = "windows"]

use std::mem::ManuallyDrop;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::{CreateEventW, INFINITE, WaitForSingleObject};
use windows::Win32::UI::HiDpi::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::{Interface, Result, w};

const FRAME_COUNT: usize = 2;
const BACK_BUFFER_FORMAT: DXGI_FORMAT = DXGI_FORMAT_R8G8B8A8_UNORM;

static VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/Vertex.cso"));
static PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/Pixel.cso"));

// The window procedure only records what happened; the main loop reacts.
// This keeps renderer state out of the callback.
static CLIENT_WIDTH: AtomicU32 = AtomicU32::new(0);
static CLIENT_HEIGHT: AtomicU32 = AtomicU32::new(0);
static RESIZED: AtomicBool = AtomicBool::new(false);

extern "system" fn window_procedure(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_SIZE => {
            CLIENT_WIDTH.store((lparam.0 & 0xFFFF) as u32, Ordering::Relaxed);
            CLIENT_HEIGHT.store(((lparam.0 >> 16) & 0xFFFF) as u32, Ordering::Relaxed);
            RESIZED.store(true, Ordering::Relaxed);
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}

struct Renderer {
    device: ID3D12Device,
    queue: ID3D12CommandQueue,
    swap_chain: IDXGISwapChain3,
    rtv_heap: ID3D12DescriptorHeap,
    rtv_increment: usize,
    back_buffers: Vec<ID3D12Resource>,
    allocators: Vec<ID3D12CommandAllocator>,
    command_list: ID3D12GraphicsCommandList,
    root_signature: ID3D12RootSignature,
    pipeline: ID3D12PipelineState,
    fence: ID3D12Fence,
    fence_event: HANDLE,
    // Fence value that marks when each back buffer's commands are finished.
    frame_fence_values: [u64; FRAME_COUNT],
    last_fence_value: u64,
    width: u32,
    height: u32,
}

impl Renderer {
    unsafe fn new(window: HWND) -> Result<Self> {
        unsafe {
            // The debug layer needs the "Graphics Tools" optional Windows feature.
            let mut factory_flags = DXGI_CREATE_FACTORY_FLAGS(0);
            if cfg!(debug_assertions) {
                let mut debug: Option<ID3D12Debug> = None;
                if D3D12GetDebugInterface(&mut debug).is_ok() {
                    debug.unwrap().EnableDebugLayer();
                    factory_flags = DXGI_CREATE_FACTORY_DEBUG;
                }
            }

            let factory: IDXGIFactory6 = CreateDXGIFactory2(factory_flags)?;
            let adapter: IDXGIAdapter1 =
                factory.EnumAdapterByGpuPreference(0, DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE)?;

            let mut device: Option<ID3D12Device> = None;
            D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_12_0, &mut device)?;
            let device = device.unwrap();

            let queue: ID3D12CommandQueue =
                device.CreateCommandQueue(&D3D12_COMMAND_QUEUE_DESC {
                    Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
                    ..Default::default()
                })?;

            // Width and height 0 means "use the window's client size".
            let swap_chain_description = DXGI_SWAP_CHAIN_DESC1 {
                Format: BACK_BUFFER_FORMAT,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: FRAME_COUNT as u32,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                ..Default::default()
            };
            let swap_chain: IDXGISwapChain3 = factory
                .CreateSwapChainForHwnd(&queue, window, &swap_chain_description, None, None)?
                .cast()?;
            factory.MakeWindowAssociation(window, DXGI_MWA_NO_ALT_ENTER)?;
            let actual = swap_chain.GetDesc1()?;

            let rtv_heap: ID3D12DescriptorHeap =
                device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                    Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
                    NumDescriptors: FRAME_COUNT as u32,
                    ..Default::default()
                })?;
            let rtv_increment =
                device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV) as usize;

            let allocators = (0..FRAME_COUNT)
                .map(|_| device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT))
                .collect::<Result<Vec<ID3D12CommandAllocator>>>()?;

            let root_signature: ID3D12RootSignature =
                device.CreateRootSignature(0, VERTEX_SHADER)?;
            let pipeline = create_pipeline(&device, &root_signature)?;

            let command_list: ID3D12GraphicsCommandList = device.CreateCommandList(
                0,
                D3D12_COMMAND_LIST_TYPE_DIRECT,
                &allocators[0],
                &pipeline,
            )?;
            command_list.Close()?;

            let fence: ID3D12Fence = device.CreateFence(0, D3D12_FENCE_FLAG_NONE)?;
            let fence_event = CreateEventW(None, false, false, None)?;

            let mut renderer = Self {
                device,
                queue,
                swap_chain,
                rtv_heap,
                rtv_increment,
                back_buffers: Vec::new(),
                allocators,
                command_list,
                root_signature,
                pipeline,
                fence,
                fence_event,
                frame_fence_values: [0; FRAME_COUNT],
                last_fence_value: 0,
                width: actual.Width,
                height: actual.Height,
            };
            renderer.create_render_target_views()?;
            Ok(renderer)
        }
    }

    unsafe fn create_render_target_views(&mut self) -> Result<()> {
        unsafe {
            for index in 0..FRAME_COUNT {
                let buffer: ID3D12Resource = self.swap_chain.GetBuffer(index as u32)?;
                self.device
                    .CreateRenderTargetView(&buffer, None, self.rtv_handle(index));
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
                self.fence.SetEventOnCompletion(value, self.fence_event)?;
                WaitForSingleObject(self.fence_event, INFINITE);
            }
            Ok(())
        }
    }

    unsafe fn wait_for_idle(&mut self) -> Result<()> {
        unsafe {
            self.last_fence_value += 1;
            self.queue.Signal(&self.fence, self.last_fence_value)?;
            self.wait_for_fence(self.last_fence_value)
        }
    }

    unsafe fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        unsafe {
            if width == 0 || height == 0 || (width == self.width && height == self.height) {
                return Ok(());
            }
            // All references to the old back buffers must be gone before ResizeBuffers.
            self.wait_for_idle()?;
            self.back_buffers.clear();
            self.swap_chain.ResizeBuffers(
                FRAME_COUNT as u32,
                width,
                height,
                DXGI_FORMAT_UNKNOWN,
                DXGI_SWAP_CHAIN_FLAG(0),
            )?;
            self.width = width;
            self.height = height;
            self.create_render_target_views()
        }
    }

    unsafe fn render(&mut self) -> Result<()> {
        unsafe {
            let index = self.swap_chain.GetCurrentBackBufferIndex() as usize;

            // The GPU might still be using this frame's allocator.
            self.wait_for_fence(self.frame_fence_values[index])?;

            let allocator = &self.allocators[index];
            let list = &self.command_list;
            allocator.Reset()?;
            list.Reset(allocator, &self.pipeline)?;

            list.SetGraphicsRootSignature(&self.root_signature);
            list.RSSetViewports(&[D3D12_VIEWPORT {
                Width: self.width as f32,
                Height: self.height as f32,
                MaxDepth: 1.0,
                ..Default::default()
            }]);
            list.RSSetScissorRects(&[RECT {
                left: 0,
                top: 0,
                right: self.width as i32,
                bottom: self.height as i32,
            }]);

            let back_buffer = &self.back_buffers[index];
            list.ResourceBarrier(&[transition(
                back_buffer,
                D3D12_RESOURCE_STATE_PRESENT,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
            )]);

            let rtv = self.rtv_handle(index);
            list.OMSetRenderTargets(1, Some(&rtv), false, None);
            list.ClearRenderTargetView(rtv, &[0.1, 0.1, 0.15, 1.0], None);
            list.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            list.DrawInstanced(3, 1, 0, 0);

            list.ResourceBarrier(&[transition(
                back_buffer,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
                D3D12_RESOURCE_STATE_PRESENT,
            )]);
            list.Close()?;

            self.queue.ExecuteCommandLists(&[Some(list.cast()?)]);
            // Sync interval 1 = vsync. Present blocks when the queue of frames is full.
            self.swap_chain.Present(1, DXGI_PRESENT(0)).ok()?;

            self.last_fence_value += 1;
            self.queue.Signal(&self.fence, self.last_fence_value)?;
            self.frame_fence_values[index] = self.last_fence_value;
            Ok(())
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            let _ = self.wait_for_idle();
            let _ = CloseHandle(self.fence_event);
        }
    }
}

unsafe fn create_pipeline(
    device: &ID3D12Device,
    root_signature: &ID3D12RootSignature,
) -> Result<ID3D12PipelineState> {
    let mut description = D3D12_GRAPHICS_PIPELINE_STATE_DESC {
        // Borrow the root signature without touching its reference count.
        pRootSignature: unsafe { std::mem::transmute_copy(root_signature) },
        VS: bytecode(VERTEX_SHADER),
        PS: bytecode(PIXEL_SHADER),
        BlendState: D3D12_BLEND_DESC {
            RenderTarget: [D3D12_RENDER_TARGET_BLEND_DESC {
                RenderTargetWriteMask: D3D12_COLOR_WRITE_ENABLE_ALL.0 as u8,
                ..Default::default()
            }; 8],
            ..Default::default()
        },
        SampleMask: u32::MAX,
        RasterizerState: D3D12_RASTERIZER_DESC {
            FillMode: D3D12_FILL_MODE_SOLID,
            CullMode: D3D12_CULL_MODE_NONE,
            DepthClipEnable: true.into(),
            ..Default::default()
        },
        PrimitiveTopologyType: D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
        NumRenderTargets: 1,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        ..Default::default()
    };
    description.RTVFormats[0] = BACK_BUFFER_FORMAT;
    unsafe { device.CreateGraphicsPipelineState(&description) }
}

fn bytecode(bytes: &[u8]) -> D3D12_SHADER_BYTECODE {
    D3D12_SHADER_BYTECODE {
        pShaderBytecode: bytes.as_ptr().cast(),
        BytecodeLength: bytes.len(),
    }
}

fn transition(
    resource: &ID3D12Resource,
    before: D3D12_RESOURCE_STATES,
    after: D3D12_RESOURCE_STATES,
) -> D3D12_RESOURCE_BARRIER {
    D3D12_RESOURCE_BARRIER {
        Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
        Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
        Anonymous: D3D12_RESOURCE_BARRIER_0 {
            Transition: ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                pResource: unsafe { std::mem::transmute_copy(resource) },
                Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                StateBefore: before,
                StateAfter: after,
            }),
        },
    }
}

fn main() -> Result<()> {
    unsafe {
        // Without this, Windows bitmap-stretches the window on high-DPI displays.
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)?;

        let instance = GetModuleHandleW(None)?;
        let class_name = w!("MainWindow");
        let class = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_procedure),
            hInstance: instance.into(),
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            lpszClassName: class_name,
            ..Default::default()
        };
        RegisterClassExW(&class);

        let window = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            w!("Hello Direct3D 12"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            1280,
            720,
            None,
            None,
            Some(instance.into()),
            None,
        )?;

        let mut renderer = Renderer::new(window)?;

        let mut message = MSG::default();
        loop {
            // Drain all pending messages, then render one frame.
            while PeekMessageW(&mut message, None, 0, 0, PM_REMOVE).as_bool() {
                if message.message == WM_QUIT {
                    return Ok(());
                }
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }

            if RESIZED.swap(false, Ordering::Relaxed) {
                renderer.resize(
                    CLIENT_WIDTH.load(Ordering::Relaxed),
                    CLIENT_HEIGHT.load(Ordering::Relaxed),
                )?;
            }

            if IsIconic(window).as_bool() {
                // Minimized: sleep until something happens instead of spinning.
                WaitMessage()?;
                continue;
            }

            renderer.render()?;
        }
    }
}
