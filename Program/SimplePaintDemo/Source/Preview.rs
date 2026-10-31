// Draws a UV sphere with SimplePaint, in an orthographic view, and puts the
// image into the frame at the preview's place.
//
// Any view of a sphere shows every normal that faces the camera, each once,
// and SimplePaint colors a surface by its normal alone, so the sphere shows
// the whole material. It never needs to turn.
//
// The sphere is drawn with 4x multisampling into its own texture, so its edge
// is smooth. The texture is resolved, averaging the samples in linear light
// through an sRGB format, then copied into the back buffer. The copy keeps
// the bytes, which are sRGB, as the back buffer is.

use graphics::{Renderer, UploadBuffer, create_texture, transition};
use math::Matrix;
use simple_paint::GpuMaterial;
use simple_paint::direct3d12::{Mesh, OBJECT_SIZE, Pipeline, Target, materials_size};
use simple_paint::orthographic::{build_object_transforms, make_projection};
use windows_api::*;
use windows_core::Result;

static VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/SimplePaintVertex.cso"));
static PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/SimplePaintPixel.cso"));

// The image's width and height, in pixels.
pub const IMAGE_SIZE: u32 = 1152;
const SAMPLE_COUNT: u32 = 4;
// Bytes stored as sRGB, viewed as sRGB for drawing and resolving.
const STORAGE_FORMAT: DXGI_FORMAT = DXGI_FORMAT_R8G8B8A8_TYPELESS;
const VIEW_FORMAT: DXGI_FORMAT = DXGI_FORMAT_R8G8B8A8_UNORM_SRGB;
// Around the sphere, the panel's white, in linear light for the sRGB view.
const BACKGROUND: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
// Enough vertices that the edge looks round at this size, and the normals
// interpolate smoothly.
const LONGITUDES: u32 = 256;
const LATITUDES: u32 = 128;
// How much of the image the sphere spans: its diameter is 2 of these 2.3.
const VIEW_SIZE: f32 = 2.3;

pub struct Preview {
    pipeline: Pipeline,
    sphere: Mesh,
    // The object's transforms, then the material, each at a constant
    // buffer's start. The GPU has finished with them when a frame is ready.
    constants: UploadBuffer,
    multisampled: ID3D12Resource,
    resolved: ID3D12Resource,
    rtv_heap: ID3D12DescriptorHeap,
}

impl Preview {
    pub unsafe fn new(renderer: &mut Renderer) -> Result<Self> {
        unsafe {
            let device = &renderer.device.clone();
            let pipeline = Pipeline::new(
                device,
                VERTEX_SHADER,
                PIXEL_SHADER,
                &Target {
                    format: VIEW_FORMAT,
                    depth_format: None,
                    sample_count: SAMPLE_COUNT,
                    // The sphere is closed and convex, so hiding its back
                    // faces leaves only the front ones, without a depth buffer.
                    cull_mode: D3D12_CULL_MODE_BACK,
                },
            )?;
            let mesh = meshes::uv_sphere(LONGITUDES, LATITUDES, 0);
            let sphere = Mesh::new(renderer, &mesh.vertices, &mesh.indices, 1)?;

            let constants = UploadBuffer::new(device, OBJECT_SIZE + materials_size(1))?;
            // The camera is 3 units in front of the sphere, looking at it.
            let view = Matrix::look_at_rh([0.0, 0.0, 3.0], [0.0; 3], [0.0, 1.0, 0.0]);
            let projection = make_projection(VIEW_SIZE, VIEW_SIZE, 1.0, 5.0)
                .map_err(|error| windows_core::Error::new(E_INVALIDARG, error.0))?;
            let object = build_object_transforms(&view, &projection)
                .map_err(|error| windows_core::Error::new(E_INVALIDARG, error.0))?;
            constants.write(0, &[object]);

            let mut clear = D3D12_CLEAR_VALUE {
                Format: VIEW_FORMAT,
                ..Default::default()
            };
            clear.Anonymous.Color = BACKGROUND;
            let multisampled = create_texture(
                device,
                IMAGE_SIZE,
                IMAGE_SIZE,
                STORAGE_FORMAT,
                SAMPLE_COUNT,
                D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
                Some(&clear),
            )?;
            let resolved = create_texture(
                device,
                IMAGE_SIZE,
                IMAGE_SIZE,
                STORAGE_FORMAT,
                1,
                D3D12_RESOURCE_FLAG_NONE,
                D3D12_RESOURCE_STATE_COPY_SOURCE,
                None,
            )?;
            let rtv_heap: ID3D12DescriptorHeap =
                device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                    Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
                    NumDescriptors: 1,
                    ..Default::default()
                })?;
            device.CreateRenderTargetView(
                &multisampled,
                Some(&D3D12_RENDER_TARGET_VIEW_DESC {
                    Format: VIEW_FORMAT,
                    ViewDimension: D3D12_RTV_DIMENSION_TEXTURE2DMS,
                    ..Default::default()
                }),
                rtv_heap.GetCPUDescriptorHandleForHeapStart(),
            );
            Ok(Self {
                pipeline,
                sphere,
                constants,
                multisampled,
                resolved,
                rtv_heap,
            })
        }
    }

    // Records the sphere with this material, and copies the image into the
    // frame's back buffer with its top left at `corner`, over what's there.
    pub unsafe fn draw(&self, renderer: &Renderer, material: &GpuMaterial, corner: [u32; 2]) {
        unsafe {
            self.constants.write(OBJECT_SIZE, &[*material]);
            let list = &renderer.command_list;
            let rtv = self.rtv_heap.GetCPUDescriptorHandleForHeapStart();
            list.OMSetRenderTargets(1, Some(&raw const rtv), false, None);
            list.RSSetViewports(&[D3D12_VIEWPORT {
                Width: IMAGE_SIZE as f32,
                Height: IMAGE_SIZE as f32,
                MaxDepth: 1.0,
                ..Default::default()
            }]);
            list.RSSetScissorRects(&[D3D12_RECT {
                left: 0,
                top: 0,
                right: IMAGE_SIZE as i32,
                bottom: IMAGE_SIZE as i32,
            }]);
            list.ClearRenderTargetView(rtv, &BACKGROUND, &[]);
            self.pipeline.bind(list);
            self.sphere.draw(
                list,
                self.constants.address(0),
                self.constants.address(OBJECT_SIZE),
            );

            list.ResourceBarrier(&[
                transition(
                    &self.multisampled,
                    D3D12_RESOURCE_STATE_RENDER_TARGET,
                    D3D12_RESOURCE_STATE_RESOLVE_SOURCE,
                ),
                transition(
                    &self.resolved,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                    D3D12_RESOURCE_STATE_RESOLVE_DEST,
                ),
            ]);
            list.ResolveSubresource(&self.resolved, 0, &self.multisampled, 0, VIEW_FORMAT);
            let back_buffer = renderer.back_buffer();
            list.ResourceBarrier(&[
                transition(
                    &self.multisampled,
                    D3D12_RESOURCE_STATE_RESOLVE_SOURCE,
                    D3D12_RESOURCE_STATE_RENDER_TARGET,
                ),
                transition(
                    &self.resolved,
                    D3D12_RESOURCE_STATE_RESOLVE_DEST,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                ),
                transition(
                    back_buffer,
                    D3D12_RESOURCE_STATE_RENDER_TARGET,
                    D3D12_RESOURCE_STATE_COPY_DEST,
                ),
            ]);
            let destination = D3D12_TEXTURE_COPY_LOCATION {
                pResource: std::mem::transmute_copy(back_buffer),
                Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                    SubresourceIndex: 0,
                },
            };
            let source = D3D12_TEXTURE_COPY_LOCATION {
                pResource: std::mem::transmute_copy(&self.resolved),
                Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                    SubresourceIndex: 0,
                },
            };
            list.CopyTextureRegion(&destination, corner[0], corner[1], 0, &source, None);
            list.ResourceBarrier(&[transition(
                back_buffer,
                D3D12_RESOURCE_STATE_COPY_DEST,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
            )]);
        }
    }
}
