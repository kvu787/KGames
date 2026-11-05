// A frame being drawn, into a swap chain's back buffer or an offscreen
// texture, and what reads its pixels back.

use windows_api::*;
use windows_core::Result;

use crate::Gpu;
use crate::resources::{
    DescriptorHeap, command_list, create_buffer, create_target_views, create_texture, execute,
    transition,
};

// What a frame's commands draw into. The target is in the render target state
// while the frame is being recorded, and must be left in it.
pub struct Frame {
    pub list: ID3D12GraphicsCommandList,
    pub target: ID3D12Resource,
    // Stores colors as they are, for colors that are sRGB already.
    pub view: D3D12_CPU_DESCRIPTOR_HANDLE,
    // Encodes linear colors to sRGB.
    pub srgb_view: D3D12_CPU_DESCRIPTOR_HANDLE,
    pub width: u32,
    pub height: u32,
}

impl Frame {
    // Draws into the whole target, through one of its views, with an
    // optional depth buffer.
    pub unsafe fn bind(&self, srgb: bool, depth: Option<D3D12_CPU_DESCRIPTOR_HANDLE>) {
        unsafe {
            let view = if srgb { self.srgb_view } else { self.view };
            self.list.OMSetRenderTargets(1, Some(&view), false, depth.as_ref().map(|depth| depth as *const _));
            self.list.RSSetViewports(&[self.viewport()]);
            self.list.RSSetScissorRects(&[RECT {
                left: 0,
                top: 0,
                right: self.width as i32,
                bottom: self.height as i32,
            }]);
        }
    }

    pub fn viewport(&self) -> D3D12_VIEWPORT {
        D3D12_VIEWPORT {
            TopLeftX: 0.0,
            TopLeftY: 0.0,
            Width: self.width as f32,
            Height: self.height as f32,
            MinDepth: 0.0,
            MaxDepth: 1.0,
        }
    }

    pub unsafe fn clear(&self, color: [f32; 4]) {
        unsafe { self.list.ClearRenderTargetView(self.view, &color, &[]) };
    }
}

// Pixels, top row first, 4 bytes each, in the target's order: blue, green,
// red and alpha.
#[derive(Clone)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl Image {
    // Red, green, blue and alpha of a pixel.
    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let at = ((y * self.width + x) * 4) as usize;
        let bgra = &self.pixels[at..at + 4];
        [bgra[2], bgra[1], bgra[0], bgra[3]]
    }
}

// A copy of a frame's target into memory that the CPU can read, once the GPU
// has made it.
pub struct Readback {
    buffer: ID3D12Resource,
    footprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT,
    width: u32,
    height: u32,
}

impl Readback {
    // Records the copy into the frame's commands, and leaves the target in the
    // render target state.
    pub unsafe fn record(device: &ID3D12Device, frame: &Frame) -> Result<Self> {
        unsafe {
            let description = frame.target.GetDesc();
            let mut footprint = D3D12_PLACED_SUBRESOURCE_FOOTPRINT::default();
            let mut total = 0u64;
            device.GetCopyableFootprints(&description, 0, 1, 0, Some(&mut footprint), None, None, Some(&mut total));
            let buffer = create_buffer(device, total, D3D12_HEAP_TYPE_READBACK, D3D12_RESOURCE_STATE_COPY_DEST)?;
            let list = &frame.list;
            list.ResourceBarrier(&[transition(
                &frame.target,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
                D3D12_RESOURCE_STATE_COPY_SOURCE,
            )]);
            let destination = D3D12_TEXTURE_COPY_LOCATION {
                pResource: std::mem::transmute_copy(&buffer),
                Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 { PlacedFootprint: footprint },
            };
            let source = D3D12_TEXTURE_COPY_LOCATION {
                pResource: std::mem::transmute_copy(&frame.target),
                Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 { SubresourceIndex: 0 },
            };
            list.CopyTextureRegion(&destination, 0, 0, 0, &source, None);
            list.ResourceBarrier(&[transition(
                &frame.target,
                D3D12_RESOURCE_STATE_COPY_SOURCE,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
            )]);
            Ok(Self {
                buffer,
                footprint,
                width: frame.width,
                height: frame.height,
            })
        }
    }

    // The pixels, once the GPU has signaled `fence`, which it does after the
    // frame's commands.
    pub fn read(self, gpu: &Gpu, fence: u64) -> Result<Image> {
        gpu.wait_for(fence)?;
        unsafe {
            let mut mapped = std::ptr::null_mut();
            self.buffer.Map(0, None, &mut mapped).ok()?;
            let pitch = self.footprint.Footprint.RowPitch as usize;
            let row = self.width as usize * 4;
            let mut pixels = Vec::with_capacity(row * self.height as usize);
            for y in 0..self.height as usize {
                let start = (mapped as *const u8).add(self.footprint.Offset as usize + y * pitch);
                pixels.extend_from_slice(std::slice::from_raw_parts(start, row));
            }
            self.buffer.Unmap(0, Some(&D3D12_RANGE { Begin: 0, End: 0 }));
            Ok(Image {
                width: self.width,
                height: self.height,
                pixels,
            })
        }
    }
}

// A texture to draw frames into without a window, for tests and for
// drawing a program's frames headless.
pub struct Offscreen {
    pub texture: ID3D12Resource,
    views: DescriptorHeap,
    allocator: ID3D12CommandAllocator,
    list: ID3D12GraphicsCommandList,
    width: u32,
    height: u32,
}

impl Offscreen {
    // Frames clear it to `clear`, which the GPU clears to fastest.
    pub fn new(gpu: &Gpu, width: u32, height: u32, clear: [f32; 4]) -> Result<Self> {
        unsafe {
            let clear = D3D12_CLEAR_VALUE {
                Format: crate::TARGET_FORMAT,
                Anonymous: D3D12_CLEAR_VALUE_0 { Color: clear },
            };
            let texture = create_texture(
                &gpu.device,
                width,
                height,
                crate::TARGET_FORMAT,
                D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
                Some(&clear),
                1,
            )?;
            let views = DescriptorHeap::new(&gpu.device, D3D12_DESCRIPTOR_HEAP_TYPE_RTV, 2, false)?;
            create_target_views(&gpu.device, &texture, &views, 0);
            let (allocator, list) = command_list(&gpu.device)?;
            Ok(Self {
                texture,
                views,
                allocator,
                list,
                width,
                height,
            })
        }
    }

    pub fn begin(&mut self) -> Result<Frame> {
        unsafe {
            self.allocator.Reset().ok()?;
            self.list.Reset(&self.allocator, None).ok()?;
        }
        Ok(Frame {
            list: self.list.clone(),
            target: self.texture.clone(),
            view: self.views.cpu(0),
            srgb_view: self.views.cpu(1),
            width: self.width,
            height: self.height,
        })
    }

    // Runs the frame's commands, waits for them, and returns what they drew.
    pub fn finish(&mut self, gpu: &mut Gpu, frame: Frame) -> Result<Image> {
        unsafe {
            let readback = Readback::record(&gpu.device, &frame)?;
            frame.list.Close().ok()?;
            execute(&gpu.queue, &frame.list)?;
            let fence = gpu.signal()?;
            let image = readback.read(gpu, fence);
            gpu.report_messages();
            image
        }
    }
}
