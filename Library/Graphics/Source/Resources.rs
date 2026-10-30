// Helpers for the buffers and textures that programs make for their draws.

use std::mem::ManuallyDrop;

use crate::Renderer;
use windows_api::*;
use windows_core::Result;

pub unsafe fn create_buffer(
    device: &ID3D12Device,
    size: u64,
    heap: D3D12_HEAP_TYPE,
    state: D3D12_RESOURCE_STATES,
) -> Result<ID3D12Resource> {
    let mut buffer: Option<ID3D12Resource> = None;
    unsafe {
        device.CreateCommittedResource(
            &D3D12_HEAP_PROPERTIES {
                Type: heap,
                ..Default::default()
            },
            D3D12_HEAP_FLAG_NONE,
            &D3D12_RESOURCE_DESC {
                Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
                Width: size,
                Height: 1,
                DepthOrArraySize: 1,
                MipLevels: 1,
                Format: DXGI_FORMAT_UNKNOWN,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
                ..Default::default()
            },
            state,
            None,
            &mut buffer,
        )?;
    }
    Ok(buffer.unwrap())
}

pub unsafe fn create_upload_buffer(device: &ID3D12Device, size: u64) -> Result<ID3D12Resource> {
    unsafe {
        create_buffer(
            device,
            size,
            D3D12_HEAP_TYPE_UPLOAD,
            D3D12_RESOURCE_STATE_GENERIC_READ,
        )
    }
}

// An upload heap buffer that stays mapped, so the CPU writes it directly and
// the GPU reads it in place. Programs keep one frame in flight, so a frame
// can rewrite what the previous one read once the renderer is ready.
pub struct UploadBuffer {
    pub resource: ID3D12Resource,
    mapped: *mut u8,
    pub size: usize,
}

impl UploadBuffer {
    pub unsafe fn new(device: &ID3D12Device, size: usize) -> Result<Self> {
        unsafe {
            let resource = create_upload_buffer(device, size as u64)?;
            let mut mapped = std::ptr::null_mut();
            // An empty read range: the CPU never reads it back.
            resource
                .Map(0, Some(&D3D12_RANGE { Begin: 0, End: 0 }), &mut mapped)
                .ok()?;
            Ok(Self {
                resource,
                mapped: mapped.cast(),
                size,
            })
        }
    }

    // Copies `data` to `offset` bytes into the buffer. The GPU mustn't be
    // reading that part.
    pub unsafe fn write<T: Copy>(&self, offset: usize, data: &[T]) {
        let bytes = size_of_val(data);
        assert!(offset + bytes <= self.size, "write past the end of an UploadBuffer");
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr().cast::<u8>(), self.mapped.add(offset), bytes)
        };
    }

    // The GPU's address of `offset` bytes into the buffer.
    pub fn address(&self, offset: usize) -> D3D12_GPU_VIRTUAL_ADDRESS {
        let start = unsafe { self.resource.GetGPUVirtualAddress() };
        D3D12_GPU_VIRTUAL_ADDRESS(start.0 + offset as u64)
    }
}

// A 2D texture in GPU memory, for render targets, depth buffers and images.
#[allow(clippy::too_many_arguments)]
pub unsafe fn create_texture(
    device: &ID3D12Device,
    width: u32,
    height: u32,
    format: DXGI_FORMAT,
    sample_count: u32,
    flags: D3D12_RESOURCE_FLAGS,
    state: D3D12_RESOURCE_STATES,
    clear_value: Option<&D3D12_CLEAR_VALUE>,
) -> Result<ID3D12Resource> {
    let mut texture: Option<ID3D12Resource> = None;
    unsafe {
        device.CreateCommittedResource(
            &D3D12_HEAP_PROPERTIES {
                Type: D3D12_HEAP_TYPE_DEFAULT,
                ..Default::default()
            },
            D3D12_HEAP_FLAG_NONE,
            &D3D12_RESOURCE_DESC {
                Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
                Width: width as u64,
                Height: height,
                DepthOrArraySize: 1,
                MipLevels: 1,
                Format: format,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: sample_count,
                    Quality: 0,
                },
                Flags: flags,
                ..Default::default()
            },
            state,
            clear_value.map(|value| value as *const _),
            &mut texture,
        )?;
    }
    Ok(texture.unwrap())
}

// Makes a texture from rows of pixels, `bytes_per_pixel` each, and leaves it
// in `state`. It waits for the GPU, so it's for setup, not for frames.
pub unsafe fn upload_texture(
    renderer: &mut Renderer,
    width: u32,
    height: u32,
    format: DXGI_FORMAT,
    bytes_per_pixel: usize,
    pixels: &[u8],
    state: D3D12_RESOURCE_STATES,
) -> Result<ID3D12Resource> {
    unsafe {
        let device = renderer.device.clone();
        let texture = create_texture(
            &device,
            width,
            height,
            format,
            1,
            D3D12_RESOURCE_FLAG_NONE,
            D3D12_RESOURCE_STATE_COPY_DEST,
            None,
        )?;

        // Rows in an upload buffer must start at 256-byte boundaries, so ask
        // D3D12 for the layout instead of assuming a tight one.
        let description = texture.GetDesc();
        let mut footprint = D3D12_PLACED_SUBRESOURCE_FOOTPRINT::default();
        let mut total_bytes = 0u64;
        device.GetCopyableFootprints(
            &description,
            0,
            1,
            0,
            Some(&mut footprint),
            None,
            None,
            Some(&mut total_bytes),
        );
        let upload = create_upload_buffer(&device, total_bytes)?;
        let mut mapped = std::ptr::null_mut();
        upload.Map(0, None, &mut mapped).ok()?;
        let row_pitch = footprint.Footprint.RowPitch as usize;
        let row_bytes = width as usize * bytes_per_pixel;
        for row in 0..height as usize {
            std::ptr::copy_nonoverlapping(
                pixels[row * row_bytes..].as_ptr(),
                (mapped as *mut u8).add(footprint.Offset as usize + row * row_pitch),
                row_bytes,
            );
        }
        upload.Unmap(0, None);

        // The upload buffer must outlive the copy, which run waits for.
        renderer.run(|list| {
            let destination = D3D12_TEXTURE_COPY_LOCATION {
                pResource: std::mem::transmute_copy(&texture),
                Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                    SubresourceIndex: 0,
                },
            };
            let source = D3D12_TEXTURE_COPY_LOCATION {
                pResource: std::mem::transmute_copy(&upload),
                Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                    PlacedFootprint: footprint,
                },
            };
            list.CopyTextureRegion(&destination, 0, 0, 0, &source, None);
            list.ResourceBarrier(&[transition(
                &texture,
                D3D12_RESOURCE_STATE_COPY_DEST,
                state,
            )]);
        })?;
        Ok(texture)
    }
}

pub fn bytecode(bytes: &[u8]) -> D3D12_SHADER_BYTECODE {
    D3D12_SHADER_BYTECODE {
        pShaderBytecode: bytes.as_ptr().cast(),
        BytecodeLength: bytes.len(),
    }
}

pub fn transition(
    resource: &ID3D12Resource,
    before: D3D12_RESOURCE_STATES,
    after: D3D12_RESOURCE_STATES,
) -> D3D12_RESOURCE_BARRIER {
    D3D12_RESOURCE_BARRIER {
        Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
        Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
        Anonymous: D3D12_RESOURCE_BARRIER_0 {
            Transition: ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                // Borrow the resource without touching its reference count.
                pResource: unsafe { std::mem::transmute_copy(resource) },
                Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                StateBefore: before,
                StateAfter: after,
            }),
        },
    }
}
