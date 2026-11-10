// A depth buffer as big as the frame, for 3D views. It's made again when the
// frame's size changes, like between the two window setups' heights, which
// is safe because the GPU has finished the last frame by the time the next
// one is drawn.

use windows_api::*;
use windows_core::Result;

use crate::resources::{DescriptorHeap, create_texture};
use crate::{DEPTH_FORMAT, Frame, Gpu};

pub struct DepthBuffer {
    // The buffer, and the frame size that it was made for.
    texture: Option<(ID3D12Resource, u32, u32)>,
    views: DescriptorHeap,
}

impl DepthBuffer {
    pub fn new(gpu: &Gpu) -> Result<Self> {
        Ok(Self {
            texture: None,
            views: unsafe { DescriptorHeap::new(&gpu.device, D3D12_DESCRIPTOR_HEAP_TYPE_DSV, 1, false)? },
        })
    }

    // The view of a depth buffer as big as the frame, made first if there's
    // none yet or the frame's size changed, with whatever it held.
    pub fn view(&mut self, gpu: &Gpu, frame: &Frame) -> Result<D3D12_CPU_DESCRIPTOR_HANDLE> {
        let view = self.views.cpu(0);
        if !matches!(self.texture, Some((_, width, height)) if (width, height) == (frame.width, frame.height)) {
            let clear = D3D12_CLEAR_VALUE {
                Format: DEPTH_FORMAT,
                Anonymous: D3D12_CLEAR_VALUE_0 {
                    DepthStencil: D3D12_DEPTH_STENCIL_VALUE { Depth: 1.0, Stencil: 0 },
                },
            };
            unsafe {
                let texture = create_texture(
                    &gpu.device,
                    frame.width,
                    frame.height,
                    DEPTH_FORMAT,
                    D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL,
                    D3D12_RESOURCE_STATE_DEPTH_WRITE,
                    Some(&clear),
                    1,
                )?;
                gpu.device.CreateDepthStencilView(&texture, None, view);
                self.texture = Some((texture, frame.width, frame.height));
            }
        }
        Ok(view)
    }

    // The view, cleared to the far plane, for a frame that draws the whole
    // of it.
    pub fn clear(&mut self, gpu: &Gpu, frame: &Frame) -> Result<D3D12_CPU_DESCRIPTOR_HANDLE> {
        let view = self.view(gpu, frame)?;
        unsafe { frame.list.ClearDepthStencilView(view, D3D12_CLEAR_FLAG_DEPTH, 1.0, 0, &[]) };
        Ok(view)
    }
}
