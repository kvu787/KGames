// Draws a UV sphere with SimplePaint, in an orthographic view, and copies
// the image into the frame at the preview card's view.
//
// Any view of a sphere shows every normal that faces the camera, each once,
// and SimplePaint colors a surface by its normal alone, so the sphere shows
// the whole material. It never needs to turn.
//
// The sphere is drawn with 4x multisampling into its own texture, so its edge
// is smooth, through an sRGB view, which encodes the shader's linear output
// once. The texture is resolved, which averages the samples in linear light,
// and the result is copied into the frame. The copy keeps the bytes, which
// are sRGB, as the frame's are.
//
// The resolved image is kept, and the sphere is drawn again only when the
// material changes. Every frame copies the image, as each frame starts
// cleared to the window's background. The image is as big as the view is in
// borderless fullscreen, its largest, around the card's color, so that it
// fills the view, and a smaller view shows its middle.

use graphics::{DescriptorHeap, Frame, Gpu, UploadBuffer, create_texture, transition};
use math::Matrix;
use simple_paint::GpuMaterial;
use simple_paint::direct3d12::{Mesh, OBJECT_SIZE, Pipeline, Target, materials_size};
use simple_paint::orthographic::{build_object_transforms, make_projection};
use user_interface::{Color, Rect};
use windows_api::*;
use windows_core::Result;

static VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/SimplePaintVertex.cso"));
static PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/SimplePaintPixel.cso"));

// The image's size in pixels: the preview card's view in borderless
// fullscreen. Maximized, the view is 71 pixels shorter.
pub const IMAGE_WIDTH: u32 = 1152;
pub const IMAGE_HEIGHT: u32 = 1226;
const SAMPLE_COUNT: u32 = 4;
// Bytes stored as sRGB, in the frame's order, viewed as sRGB for drawing and
// resolving, and copied into the frame as they are.
const STORAGE_FORMAT: DXGI_FORMAT = DXGI_FORMAT_B8G8R8A8_TYPELESS;
const VIEW_FORMAT: DXGI_FORMAT = DXGI_FORMAT_B8G8R8A8_UNORM_SRGB;
// Enough vertices that the edge looks round at this size, and the normals
// interpolate smoothly.
const LONGITUDES: u32 = 256;
const LATITUDES: u32 = 128;
// How much of the image's width the sphere spans: its diameter is 2 of
// these 2.3.
const VIEW_SIZE: f32 = 2.3;

pub struct Preview {
    pipeline: Pipeline,
    sphere: Mesh,
    // The object's transforms, then the material, each at a constant
    // buffer's start. Only one frame is on the GPU at a time, and it's
    // finished before the next one is recorded, so they can change then.
    constants: UploadBuffer,
    rtv_heap: DescriptorHeap,
    // Made by the first frame, with the card's color around the sphere.
    image: Option<Image>,
    // The material that the image has the sphere in, once it has one.
    drawn: Option<GpuMaterial>,
}

struct Image {
    multisampled: ID3D12Resource,
    resolved: ID3D12Resource,
    // Around the sphere, in sRGB.
    background: Color,
}

impl Preview {
    pub unsafe fn new(gpu: &mut Gpu) -> Result<Self> {
        unsafe {
            let device = gpu.device.clone();
            let pipeline = Pipeline::new(
                &device,
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
            let sphere = Mesh::new(gpu, &mesh.vertices, &mesh.indices, 1)?;

            let constants = UploadBuffer::new(&device, OBJECT_SIZE + materials_size(1))?;
            // The camera is 3 units in front of the sphere, looking at it.
            let view = Matrix::look_at_rh([0.0, 0.0, 3.0], [0.0; 3], [0.0, 1.0, 0.0]);
            let height = VIEW_SIZE * IMAGE_HEIGHT as f32 / IMAGE_WIDTH as f32;
            let projection = make_projection(VIEW_SIZE, height, 1.0, 5.0)
                .map_err(|error| windows_core::Error::new(E_INVALIDARG, error.0))?;
            let object = build_object_transforms(&view, &projection)
                .map_err(|error| windows_core::Error::new(E_INVALIDARG, error.0))?;
            constants.write(0, &[object]);
            let rtv_heap = DescriptorHeap::new(&device, D3D12_DESCRIPTOR_HEAP_TYPE_RTV, 1, false)?;
            Ok(Self {
                pipeline,
                sphere,
                constants,
                rtv_heap,
                image: None,
                drawn: None,
            })
        }
    }

    // Copies the image of the sphere with this material, around `background`,
    // into the frame at `view`, which it fills. Records the sphere first if
    // the image has another material or background.
    pub unsafe fn render(
        &mut self,
        gpu: &Gpu,
        frame: &Frame,
        material: &GpuMaterial,
        background: Color,
        view: Rect,
    ) -> Result<()> {
        unsafe {
            if self.image.as_ref().is_none_or(|image| image.background != background) {
                self.image = Some(self.create_image(gpu, background)?);
                self.drawn = None;
            }
            if self.drawn != Some(*material) {
                self.draw_sphere(frame, material);
                self.drawn = Some(*material);
            }
            self.copy(frame, view);
            Ok(())
        }
    }

    // The multisampled texture, whose clear color is the background, and
    // the resolved one, which waits to be copied from.
    unsafe fn create_image(&self, gpu: &Gpu, background: Color) -> Result<Image> {
        unsafe {
            let device = &gpu.device;
            let clear = D3D12_CLEAR_VALUE {
                Format: VIEW_FORMAT,
                Anonymous: D3D12_CLEAR_VALUE_0 {
                    Color: linear(background),
                },
            };
            let multisampled = create_texture(
                device,
                IMAGE_WIDTH,
                IMAGE_HEIGHT,
                STORAGE_FORMAT,
                D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
                Some(&clear),
                SAMPLE_COUNT,
            )?;
            let resolved = create_texture(
                device,
                IMAGE_WIDTH,
                IMAGE_HEIGHT,
                STORAGE_FORMAT,
                D3D12_RESOURCE_FLAG_NONE,
                D3D12_RESOURCE_STATE_COPY_SOURCE,
                None,
                1,
            )?;
            device.CreateRenderTargetView(
                &multisampled,
                Some(&D3D12_RENDER_TARGET_VIEW_DESC {
                    Format: VIEW_FORMAT,
                    ViewDimension: D3D12_RTV_DIMENSION_TEXTURE2DMS,
                    ..Default::default()
                }),
                self.rtv_heap.cpu(0),
            );
            Ok(Image {
                multisampled,
                resolved,
                background,
            })
        }
    }

    // Records the sphere with this material into the resolved image, which
    // ends ready to copy from. The frame's own target is bound again by
    // whatever draws into it next.
    unsafe fn draw_sphere(&self, frame: &Frame, material: &GpuMaterial) {
        unsafe {
            let image = self.image.as_ref().expect("the image is made before the sphere is drawn");
            self.constants.write(OBJECT_SIZE, &[*material]);
            let list = &frame.list;
            let rtv = self.rtv_heap.cpu(0);
            list.OMSetRenderTargets(1, Some(&raw const rtv), false, None);
            list.RSSetViewports(&[D3D12_VIEWPORT {
                Width: IMAGE_WIDTH as f32,
                Height: IMAGE_HEIGHT as f32,
                MaxDepth: 1.0,
                ..Default::default()
            }]);
            list.RSSetScissorRects(&[D3D12_RECT {
                left: 0,
                top: 0,
                right: IMAGE_WIDTH as i32,
                bottom: IMAGE_HEIGHT as i32,
            }]);
            list.ClearRenderTargetView(rtv, &linear(image.background), &[]);
            self.pipeline.bind(list);
            self.sphere.draw(list, self.constants.gpu_address(0), self.constants.gpu_address(OBJECT_SIZE));

            list.ResourceBarrier(&[
                transition(
                    &image.multisampled,
                    D3D12_RESOURCE_STATE_RENDER_TARGET,
                    D3D12_RESOURCE_STATE_RESOLVE_SOURCE,
                ),
                transition(&image.resolved, D3D12_RESOURCE_STATE_COPY_SOURCE, D3D12_RESOURCE_STATE_RESOLVE_DEST),
            ]);
            list.ResolveSubresource(&image.resolved, 0, &image.multisampled, 0, VIEW_FORMAT);
            list.ResourceBarrier(&[
                transition(
                    &image.multisampled,
                    D3D12_RESOURCE_STATE_RESOLVE_SOURCE,
                    D3D12_RESOURCE_STATE_RENDER_TARGET,
                ),
                transition(&image.resolved, D3D12_RESOURCE_STATE_RESOLVE_DEST, D3D12_RESOURCE_STATE_COPY_SOURCE),
            ]);
        }
    }

    // Copies the middle of the image into the frame, centered in the view,
    // and leaves the frame's target as it found it, ready to draw into.
    unsafe fn copy(&self, frame: &Frame, view: Rect) {
        unsafe {
            let image = self.image.as_ref().expect("the image is made before it's copied");
            let place = copy_place(view, [frame.width, frame.height]);
            let Some(CopyPlace { destination: [x, y], source }) = place else {
                return;
            };
            let list = &frame.list;
            list.ResourceBarrier(&[transition(
                &frame.target,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
                D3D12_RESOURCE_STATE_COPY_DEST,
            )]);
            let destination = D3D12_TEXTURE_COPY_LOCATION {
                pResource: std::mem::transmute_copy(&frame.target),
                Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 { SubresourceIndex: 0 },
            };
            let from = D3D12_TEXTURE_COPY_LOCATION {
                pResource: std::mem::transmute_copy(&image.resolved),
                Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 { SubresourceIndex: 0 },
            };
            list.CopyTextureRegion(&destination, x, y, 0, &from, Some(&source));
            list.ResourceBarrier(&[transition(
                &frame.target,
                D3D12_RESOURCE_STATE_COPY_DEST,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
            )]);
        }
    }
}

// Where the image goes in a frame: the top left in the frame, and the part
// of the image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CopyPlace {
    pub destination: [u32; 2],
    pub source: D3D12_BOX,
}

// The part of the image that fills the view, from its middle, inside the
// frame of the size. None if none of it would show.
pub fn copy_place(view: Rect, frame: [u32; 2]) -> Option<CopyPlace> {
    let view = view.round();
    // The image's middle at the view's middle, cut to the view and the frame.
    let left = (view.x as i64 + (view.width as i64 - IMAGE_WIDTH as i64) / 2).max(view.x as i64).max(0);
    let top = (view.y as i64 + (view.height as i64 - IMAGE_HEIGHT as i64) / 2).max(view.y as i64).max(0);
    let right = (view.x as i64 + (view.width as i64 + IMAGE_WIDTH as i64) / 2)
        .min(view.right() as i64)
        .min(frame[0] as i64);
    let bottom = (view.y as i64 + (view.height as i64 + IMAGE_HEIGHT as i64) / 2)
        .min(view.bottom() as i64)
        .min(frame[1] as i64);
    if right <= left || bottom <= top {
        return None;
    }
    // Where the image's top left is in the frame.
    let image_x = view.x as i64 + (view.width as i64 - IMAGE_WIDTH as i64) / 2;
    let image_y = view.y as i64 + (view.height as i64 - IMAGE_HEIGHT as i64) / 2;
    Some(CopyPlace {
        destination: [left as u32, top as u32],
        source: D3D12_BOX {
            left: (left - image_x) as u32,
            top: (top - image_y) as u32,
            front: 0,
            right: (right - image_x) as u32,
            bottom: (bottom - image_y) as u32,
            back: 1,
        },
    })
}

// An sRGB color as linear light, for clearing through an sRGB view, which
// encodes it back to the same bytes.
fn linear(color: Color) -> [f32; 4] {
    let [r, g, b] = color.to_linear();
    [r, g, b, 1.0]
}
