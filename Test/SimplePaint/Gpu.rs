// SimplePaint's production vertex and pixel shaders on the GPU, against the
// independent binary64 reference: every case draws one pixel, through the
// real root signature and material array, on the preferred hardware adapter
// and on WARP, which works on machines without a GPU. It fails on D3D12
// debug-layer warnings or errors when the debug layer is installed.
//
// With --benchmark, it instead times 128 warmed 1024 x 1024 draws for Shift 0
// and Shift 0.6 on the hardware adapter, with GPU timestamps.

#[path = "Reference.rs"]
mod reference;

use graphics::{UploadBuffer, create_buffer, create_texture, transition};
use math::Matrix;
use reference::Case;
use simple_paint::direct3d12::{Pipeline, Target};
use simple_paint::orthographic::{ObjectTransforms, build_object_transforms, make_projection};
use simple_paint::{GpuMaterial, Material, Parameters, Vertex};
use windows_api::*;
use windows_core::{Interface, PCWSTR, Result};

static VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/SimplePaintVertex.cso"));
static PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/SimplePaintPixel.cso"));
// The shaders' SIMPLE_PAINT_MATERIAL_COUNT, from Build.rs.
const MATERIAL_COUNT: usize = 6;
const FORMAT: DXGI_FORMAT = DXGI_FORMAT_R32G32B32A32_FLOAT;
const BENCHMARK_REPETITIONS: u32 = 128;

// One case's constant buffers: the object's transforms at b0, then the
// materials at b1, each at a 256-byte boundary.
#[repr(C, align(256))]
#[derive(Clone, Copy)]
struct Constants {
    object: ObjectTransforms,
    _object_padding: [u8; 160],
    materials: [GpuMaterial; MATERIAL_COUNT],
    _tail_padding: [u8; 32],
}

const _: () = assert!(size_of::<Constants>() == 768);

#[derive(Clone, Copy, PartialEq)]
enum Adapter {
    Hardware,
    Warp,
}

fn main() {
    let benchmark = std::env::args().any(|argument| argument == "--benchmark");
    unsafe {
        let mut debug: Option<ID3D12Debug> = None;
        if D3D12GetDebugInterface(&mut debug).is_ok() {
            debug.unwrap().EnableDebugLayer();
        }
        if benchmark {
            run(Adapter::Hardware, true).unwrap();
        } else {
            run(Adapter::Hardware, false).unwrap();
            run(Adapter::Warp, false).unwrap();
        }
    }
}

unsafe fn run(adapter: Adapter, benchmark: bool) -> Result<()> {
    unsafe {
        let factory: IDXGIFactory6 = CreateDXGIFactory2(0)?;
        let adapter: IDXGIAdapter1 = match adapter {
            Adapter::Hardware => {
                factory.EnumAdapterByGpuPreference(0, DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE)?
            }
            Adapter::Warp => factory.cast::<IDXGIFactory4>()?.EnumWarpAdapter()?,
        };
        let mut description = DXGI_ADAPTER_DESC1::default();
        adapter.GetDesc1(&mut description).ok()?;
        let name = &description.Description;
        let name = String::from_utf16_lossy(&name[..name.iter().position(|&c| c == 0).unwrap_or(name.len())]);
        let mut device: Option<ID3D12Device> = None;
        D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_11_0, &mut device)?;
        let device = device.unwrap();
        let info: Option<ID3D12InfoQueue> = device.cast().ok();

        let pipeline = Pipeline::new(
            &device,
            VERTEX_SHADER,
            PIXEL_SHADER,
            &Target {
                format: FORMAT,
                depth_format: None,
                sample_count: 1,
                cull_mode: D3D12_CULL_MODE_NONE,
            },
        )?;

        let mut cases = if benchmark {
            let normal = [0.6, 0.2, 0.8];
            vec![
                Case {
                    parameters: Parameters::default(),
                    normal,
                },
                Case {
                    parameters: Parameters {
                        shift: 0.6,
                        ..Default::default()
                    },
                    normal,
                },
            ]
        } else {
            reference::cases()
        };
        let object = build_object_transforms(
            &Matrix::IDENTITY,
            &make_projection(2.0, 2.0, 0.0, 1.0).unwrap(),
        )
        .unwrap();
        let mut constants = Vec::with_capacity(cases.len());
        let mut vertices = Vec::with_capacity(cases.len() * 3);
        let count = cases.len();
        for (i, case) in cases.iter_mut().enumerate() {
            let index = i % MATERIAL_COUNT;
            let mut record = Constants {
                object,
                _object_padding: [0; 160],
                materials: [GpuMaterial::default(); MATERIAL_COUNT],
                _tail_padding: [0; 32],
            };
            record.materials[index] = *Material::compile(&case.parameters).unwrap().constants();
            constants.push(record);
            // A triangle that covers the pixel, whose center has barycentrics
            // (1/2, 1/4, 1/4).
            let n = case.normal;
            let mut triangle = [[-1.0f32, -1.0], [3.0, -1.0], [-1.0, 3.0]].map(|[x, y]| Vertex {
                position: [x, y, 0.0],
                normal: n,
                material_index: index as u32,
            });
            if !benchmark && i >= count - 4096 {
                // Distinct vertex normals exercise interpolation and the
                // vertex stage's rotation together.
                let delta = 0.125 * (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
                triangle[0].normal[0] += delta;
                triangle[1].normal[0] -= 2.0 * delta;
                triangle[0].normal[1] -= delta;
                triangle[2].normal[1] += 2.0 * delta;
                let center = |axis: usize| {
                    (0.5 * triangle[0].normal[axis] as f64
                        + 0.25 * triangle[1].normal[axis] as f64
                        + 0.25 * triangle[2].normal[axis] as f64) as f32
                };
                case.normal = [center(0), center(1), n[2]];
            }
            vertices.extend(triangle);
        }
        let constant_buffer = UploadBuffer::new(&device, size_of_val(constants.as_slice()))?;
        constant_buffer.write(0, &constants);
        let vertex_buffer = UploadBuffer::new(&device, size_of_val(vertices.as_slice()))?;
        vertex_buffer.write(0, &vertices);

        let width = if benchmark { 1024 } else { 256 };
        let height = if benchmark { 1024 } else { count.div_ceil(256) as u32 };
        let target = create_texture(
            &device,
            width,
            height,
            FORMAT,
            1,
            D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET,
            D3D12_RESOURCE_STATE_RENDER_TARGET,
            None,
        )?;
        let rtv_heap: ID3D12DescriptorHeap = device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            NumDescriptors: 1,
            ..Default::default()
        })?;
        let rtv = rtv_heap.GetCPUDescriptorHandleForHeapStart();
        device.CreateRenderTargetView(&target, None, rtv);
        let target_description = target.GetDesc();
        let mut footprint = D3D12_PLACED_SUBRESOURCE_FOOTPRINT::default();
        let mut read_size = 0u64;
        device.GetCopyableFootprints(
            &target_description,
            0,
            1,
            0,
            Some(&mut footprint),
            None,
            None,
            Some(&mut read_size),
        );
        let readback = create_buffer(
            &device,
            read_size,
            D3D12_HEAP_TYPE_READBACK,
            D3D12_RESOURCE_STATE_COPY_DEST,
        )?;

        let queue: ID3D12CommandQueue = device.CreateCommandQueue(&D3D12_COMMAND_QUEUE_DESC {
            Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
            ..Default::default()
        })?;
        let allocator: ID3D12CommandAllocator =
            device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)?;
        let list: ID3D12GraphicsCommandList =
            device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &allocator, None)?;
        let timestamps = if benchmark {
            let mut heap: Option<ID3D12QueryHeap> = None;
            device.CreateQueryHeap(
                &D3D12_QUERY_HEAP_DESC {
                    Type: D3D12_QUERY_HEAP_TYPE_TIMESTAMP,
                    Count: 4,
                    NodeMask: 0,
                },
                &mut heap,
            )?;
            let readback = create_buffer(
                &device,
                4 * size_of::<u64>() as u64,
                D3D12_HEAP_TYPE_READBACK,
                D3D12_RESOURCE_STATE_COPY_DEST,
            )?;
            Some((heap.unwrap(), readback))
        } else {
            None
        };

        pipeline.bind(&list);
        list.OMSetRenderTargets(1, Some(&raw const rtv), false, None);
        list.IASetVertexBuffers(
            0,
            Some(&[D3D12_VERTEX_BUFFER_VIEW {
                BufferLocation: vertex_buffer.address(0),
                SizeInBytes: size_of_val(vertices.as_slice()) as u32,
                StrideInBytes: size_of::<Vertex>() as u32,
            }]),
        );
        for i in 0..count {
            let (x, y) = ((i % width as usize) as i32, (i / width as usize) as i32);
            let (viewport, scissor) = if benchmark {
                ([0.0, 0.0, width as f32, height as f32], [0, 0, width as i32, height as i32])
            } else {
                ([x as f32, y as f32, 1.0, 1.0], [x, y, x + 1, y + 1])
            };
            list.RSSetViewports(&[D3D12_VIEWPORT {
                TopLeftX: viewport[0],
                TopLeftY: viewport[1],
                Width: viewport[2],
                Height: viewport[3],
                MinDepth: 0.0,
                MaxDepth: 1.0,
            }]);
            list.RSSetScissorRects(&[D3D12_RECT {
                left: scissor[0],
                top: scissor[1],
                right: scissor[2],
                bottom: scissor[3],
            }]);
            let offset = i * size_of::<Constants>();
            list.SetGraphicsRootConstantBufferView(0, constant_buffer.address(offset));
            list.SetGraphicsRootConstantBufferView(1, constant_buffer.address(offset + 256));
            let first_vertex = (i * 3) as u32;
            if let Some((heap, _)) = &timestamps {
                // Warm each path, then time production draws on the GPU,
                // without CPU submission, setup, transfers or readback.
                for _ in 0..32 {
                    list.DrawInstanced(3, 1, first_vertex, 0);
                }
                list.EndQuery(heap, D3D12_QUERY_TYPE_TIMESTAMP, (i * 2) as u32);
                for _ in 0..BENCHMARK_REPETITIONS {
                    list.DrawInstanced(3, 1, first_vertex, 0);
                }
                list.EndQuery(heap, D3D12_QUERY_TYPE_TIMESTAMP, (i * 2 + 1) as u32);
            } else {
                list.DrawInstanced(3, 1, first_vertex, 0);
            }
        }
        if let Some((heap, readback)) = &timestamps {
            list.ResolveQueryData(heap, D3D12_QUERY_TYPE_TIMESTAMP, 0, 4, readback, 0);
        }
        list.ResourceBarrier(&[transition(
            &target,
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
            pResource: std::mem::transmute_copy(&target),
            Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
            Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                SubresourceIndex: 0,
            },
        };
        list.CopyTextureRegion(&destination, 0, 0, 0, &source, None);
        list.Close().ok()?;
        queue.ExecuteCommandLists(&[Some(list.cast()?)]);
        let fence: ID3D12Fence = device.CreateFence(0, D3D12_FENCE_FLAG_NONE)?;
        queue.Signal(&fence, 1).ok()?;
        let event = CreateEventW(None, false, false, PCWSTR::null());
        check_handle(event.0)?;
        fence.SetEventOnCompletion(1, event).ok()?;
        let wait = WaitForSingleObject(event, 60_000);
        let _ = CloseHandle(event);
        assert!(wait == WAIT_OBJECT_0 as u32, "GPU test fence timeout");

        if let Some((_, readback)) = &timestamps {
            let mut mapped = std::ptr::null_mut();
            readback
                .Map(0, Some(&D3D12_RANGE { Begin: 0, End: 4 * size_of::<u64>() }), &mut mapped)
                .ok()?;
            let ticks = std::slice::from_raw_parts(mapped as *const u64, 4);
            let frequency = queue.GetTimestampFrequency()?;
            for (i, label) in ["Zero shift", "Shift 0.6"].into_iter().enumerate() {
                let milliseconds = 1000.0 * (ticks[i * 2 + 1] - ticks[i * 2]) as f64
                    / frequency as f64
                    / BENCHMARK_REPETITIONS as f64;
                println!(
                    "{name}: {label}: {milliseconds} ms per 1024x1024 draw \
                     ({BENCHMARK_REPETITIONS} draws, constant normal, float4 render target)."
                );
            }
            readback.Unmap(0, Some(&D3D12_RANGE { Begin: 0, End: 0 }));
            return Ok(());
        }

        let mut mapped = std::ptr::null_mut();
        readback
            .Map(0, Some(&D3D12_RANGE { Begin: 0, End: read_size as usize }), &mut mapped)
            .ok()?;
        let (mut max_linear, mut max_srgb) = (0.0f64, 0.0f64);
        for (i, case) in cases.iter().enumerate() {
            let row = footprint.Offset as usize
                + (i / width as usize) * footprint.Footprint.RowPitch as usize;
            let pixel = std::slice::from_raw_parts(
                (mapped as *const u8).add(row + (i % width as usize) * 16) as *const f32,
                4,
            );
            let expected = reference::reference(case);
            assert!(pixel[3] == 1.0, "Production pixel shader alpha mismatch or missing draw");
            for channel in 0..3 {
                let value = pixel[channel];
                assert!(
                    value.is_finite() && (0.0..=1.0).contains(&value),
                    "GPU output is not finite in [0,1]: case {i}, channel {channel}, value {value}"
                );
                let linear_error = (value as f64 - expected[channel]).abs();
                let srgb_error =
                    (reference::srgb(value as f64) - reference::srgb(expected[channel])).abs();
                max_linear = max_linear.max(linear_error);
                max_srgb = max_srgb.max(srgb_error);
                assert!(
                    linear_error <= 0.001 && srgb_error <= 0.001,
                    "GPU reference mismatch at case {i}, linear error={linear_error}, sRGB error={srgb_error}"
                );
            }
        }
        readback.Unmap(0, Some(&D3D12_RANGE { Begin: 0, End: 0 }));
        if let Some(info) = &info {
            for i in 0..info.GetNumStoredMessages() {
                let mut size = 0usize;
                info.GetMessage(i, None, &mut size).ok()?;
                // u64s, so the message is aligned for its pointer fields.
                let mut storage = vec![0u64; size.div_ceil(8)];
                let message = storage.as_mut_ptr() as *mut D3D12_MESSAGE;
                info.GetMessage(i, Some(message), &mut size).ok()?;
                let message = &*message;
                assert!(
                    message.Severity > D3D12_MESSAGE_SEVERITY_WARNING,
                    "DX12 validation: {}",
                    std::ffi::CStr::from_ptr(message.pDescription).to_string_lossy()
                );
            }
        }
        println!(
            "{name}: {count} production VS/PS cases passed; max linear error={max_linear}, \
             max sRGB error={max_srgb}; debug layer={}",
            if info.is_some() { "checked" } else { "unavailable" }
        );
        Ok(())
    }
}
