// Buffers, textures, barriers and descriptor heaps.

use windows_api::*;
use windows_core::{Interface, Result};

pub fn heap_properties(kind: D3D12_HEAP_TYPE) -> D3D12_HEAP_PROPERTIES {
    D3D12_HEAP_PROPERTIES {
        Type: kind,
        CreationNodeMask: 1,
        VisibleNodeMask: 1,
        ..Default::default()
    }
}

// A buffer of `size` bytes in a heap of the kind.
pub unsafe fn create_buffer(
    device: &ID3D12Device,
    size: u64,
    heap: D3D12_HEAP_TYPE,
    state: D3D12_RESOURCE_STATES,
) -> Result<ID3D12Resource> {
    unsafe {
        let description = D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
            Width: size.max(1),
            Height: 1,
            DepthOrArraySize: 1,
            MipLevels: 1,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            ..Default::default()
        };
        let mut resource: Option<ID3D12Resource> = None;
        device.CreateCommittedResource(
            &heap_properties(heap),
            D3D12_HEAP_FLAG_NONE,
            &description,
            state,
            None,
            &mut resource,
        )?;
        Ok(resource.unwrap())
    }
}

// A 2D texture in GPU memory, with one mip level.
#[allow(clippy::too_many_arguments)]
pub unsafe fn create_texture(
    device: &ID3D12Device,
    width: u32,
    height: u32,
    format: DXGI_FORMAT,
    flags: D3D12_RESOURCE_FLAGS,
    state: D3D12_RESOURCE_STATES,
    clear: Option<&D3D12_CLEAR_VALUE>,
    sample_count: u32,
) -> Result<ID3D12Resource> {
    unsafe {
        let description = D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
            Width: width as u64,
            Height: height,
            DepthOrArraySize: 1,
            MipLevels: 1,
            Format: format,
            SampleDesc: DXGI_SAMPLE_DESC { Count: sample_count, Quality: 0 },
            Flags: flags,
            ..Default::default()
        };
        let mut resource: Option<ID3D12Resource> = None;
        device.CreateCommittedResource(
            &heap_properties(D3D12_HEAP_TYPE_DEFAULT),
            D3D12_HEAP_FLAG_NONE,
            &description,
            state,
            clear.map(|clear| clear as *const _),
            &mut resource,
        )?;
        Ok(resource.unwrap())
    }
}

// A barrier that moves every subresource of the resource from one state to
// another. It borrows the resource, which must outlive it.
pub fn transition(
    resource: &ID3D12Resource,
    before: D3D12_RESOURCE_STATES,
    after: D3D12_RESOURCE_STATES,
) -> D3D12_RESOURCE_BARRIER {
    D3D12_RESOURCE_BARRIER {
        Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
        Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
        Anonymous: D3D12_RESOURCE_BARRIER_0 {
            Transition: std::mem::ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                // Borrowed without touching its reference count.
                pResource: unsafe { std::mem::transmute_copy(resource) },
                Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                StateBefore: before,
                StateAfter: after,
            }),
        },
    }
}

pub fn bytecode(shader: &[u8]) -> D3D12_SHADER_BYTECODE {
    D3D12_SHADER_BYTECODE {
        pShaderBytecode: shader.as_ptr() as *const _,
        BytecodeLength: shader.len(),
    }
}

// A buffer in an upload heap that stays mapped, which the CPU writes and the
// GPU reads across PCIe. The writer must keep the GPU from reading what it
// overwrites, like by waiting for the frame that read it.
pub struct UploadBuffer {
    pub resource: ID3D12Resource,
    pub size: usize,
    mapped: *mut u8,
}

impl UploadBuffer {
    pub unsafe fn new(device: &ID3D12Device, size: usize) -> Result<Self> {
        unsafe {
            let resource = create_buffer(
                device,
                size as u64,
                D3D12_HEAP_TYPE_UPLOAD,
                D3D12_RESOURCE_STATE_GENERIC_READ,
            )?;
            let mut mapped = std::ptr::null_mut();
            // The CPU never reads it.
            resource
                .Map(0, Some(&D3D12_RANGE { Begin: 0, End: 0 }), &mut mapped)
                .ok()?;
            Ok(Self {
                resource,
                size,
                mapped: mapped as *mut u8,
            })
        }
    }

    // Copies the values to `offset` bytes into the buffer.
    pub fn write<T: Copy>(&self, offset: usize, values: &[T]) {
        let bytes = size_of_val(values);
        assert!(offset + bytes <= self.size, "write past the end of an upload buffer");
        unsafe {
            std::ptr::copy_nonoverlapping(values.as_ptr() as *const u8, self.mapped.add(offset), bytes);
        }
    }

    pub fn bytes_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.mapped, self.size) }
    }

    pub fn gpu_address(&self, offset: usize) -> D3D12_GPU_VIRTUAL_ADDRESS {
        D3D12_GPU_VIRTUAL_ADDRESS(unsafe { self.resource.GetGPUVirtualAddress().0 } + offset as u64)
    }
}

// A descriptor heap of one kind, with its descriptors' handles.
pub struct DescriptorHeap {
    pub heap: ID3D12DescriptorHeap,
    cpu_start: D3D12_CPU_DESCRIPTOR_HANDLE,
    gpu_start: D3D12_GPU_DESCRIPTOR_HANDLE,
    increment: usize,
}

impl DescriptorHeap {
    pub unsafe fn new(
        device: &ID3D12Device,
        kind: D3D12_DESCRIPTOR_HEAP_TYPE,
        count: u32,
        shader_visible: bool,
    ) -> Result<Self> {
        unsafe {
            let heap: ID3D12DescriptorHeap = device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                Type: kind,
                NumDescriptors: count,
                Flags: if shader_visible {
                    D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE
                } else {
                    D3D12_DESCRIPTOR_HEAP_FLAG_NONE
                },
                NodeMask: 0,
            })?;
            let cpu_start = heap.GetCPUDescriptorHandleForHeapStart();
            let gpu_start = if shader_visible {
                heap.GetGPUDescriptorHandleForHeapStart()
            } else {
                D3D12_GPU_DESCRIPTOR_HANDLE::default()
            };
            let increment = device.GetDescriptorHandleIncrementSize(kind) as usize;
            Ok(Self {
                heap,
                cpu_start,
                gpu_start,
                increment,
            })
        }
    }

    pub fn cpu(&self, index: usize) -> D3D12_CPU_DESCRIPTOR_HANDLE {
        D3D12_CPU_DESCRIPTOR_HANDLE {
            ptr: self.cpu_start.ptr + index * self.increment,
        }
    }

    pub fn gpu(&self, index: usize) -> D3D12_GPU_DESCRIPTOR_HANDLE {
        D3D12_GPU_DESCRIPTOR_HANDLE {
            ptr: self.gpu_start.ptr + (index * self.increment) as u64,
        }
    }
}

// Creates the render target views of a target: one that stores colors as
// they are, and one that encodes linear colors to sRGB.
pub(crate) unsafe fn create_target_views(
    device: &ID3D12Device,
    target: &ID3D12Resource,
    heap: &DescriptorHeap,
    first: usize,
) {
    unsafe {
        for (index, format) in [(first, crate::TARGET_FORMAT), (first + 1, crate::TARGET_SRGB_FORMAT)] {
            let description = D3D12_RENDER_TARGET_VIEW_DESC {
                Format: format,
                ViewDimension: D3D12_RTV_DIMENSION_TEXTURE2D,
                ..Default::default()
            };
            device.CreateRenderTargetView(target, Some(&description), heap.cpu(index));
        }
    }
}

pub(crate) unsafe fn command_list(
    device: &ID3D12Device,
) -> Result<(ID3D12CommandAllocator, ID3D12GraphicsCommandList)> {
    unsafe {
        let allocator: ID3D12CommandAllocator =
            device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)?;
        let list: ID3D12GraphicsCommandList =
            device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &allocator, None)?;
        list.Close().ok()?;
        Ok((allocator, list))
    }
}

pub(crate) unsafe fn execute(queue: &ID3D12CommandQueue, list: &ID3D12GraphicsCommandList) -> Result<()> {
    unsafe {
        queue.ExecuteCommandLists(&[Some(list.cast::<ID3D12CommandList>()?)]);
    }
    Ok(())
}
