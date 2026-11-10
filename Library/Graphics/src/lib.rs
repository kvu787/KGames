// Direct3D 12 for KGames' programs: the GPU that draws, a swap chain paced
// for the lowest input latency, frames to draw into, offscreen targets for
// tests, readback of what was drawn, and the small helpers that every user of
// the device needs, like buffers, textures and barriers.
//
// Library/UserInterface draws its widgets with it, and programs draw their 3D
// views into the same frames, before the widgets go on top.

mod depth;
mod device;
mod frame;
mod png;
mod presenter;
mod resources;
mod timer;

pub use depth::DepthBuffer;
pub use device::{AdapterChoice, Gpu, GpuPriority, VideoMemory, gpu_priority, set_gpu_priority};
pub use frame::{Frame, Image, Offscreen, Readback};
pub use png::write_png;
pub use presenter::{Presenter, present_then_signal};
pub use resources::{
    DescriptorHeap, UploadBuffer, bytecode, create_buffer, create_texture, heap_properties,
    transition,
};
pub use timer::{GpuTimer, Span};

use windows_api::*;

// The format of every frame's target, swap chain or offscreen. Flip model
// swap chains can't be sRGB, so frames have two views: this one, which
// stores colors as they are, for the interface, whose colors are sRGB
// already, and an sRGB one, which encodes linear colors, for 3D shading.
pub const TARGET_FORMAT: DXGI_FORMAT = DXGI_FORMAT_B8G8R8A8_UNORM;
pub const TARGET_SRGB_FORMAT: DXGI_FORMAT = DXGI_FORMAT_B8G8R8A8_UNORM_SRGB;
// For the depth buffers of 3D views.
pub const DEPTH_FORMAT: DXGI_FORMAT = DXGI_FORMAT_D32_FLOAT;
// Where constant buffers may start in a buffer.
pub const CONSTANT_ALIGNMENT: usize = 256;
