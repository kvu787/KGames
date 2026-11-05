# Graphics

Direct3D 12 for KGames' programs: the GPU that draws, a swap chain paced for the lowest input
latency, frames to draw into, offscreen targets for tests, readback of what was drawn, PNGs, and the
helpers that every user of the device needs. [Library/UserInterface](../UserInterface/README.md)
draws its widgets with it, and programs draw their 3D views into the same frames, before the
widgets go on top.

| Type or function                                            | What it is                                                                 |
| ----------------------------------------------------------- | -------------------------------------------------------------------------- |
| `Gpu`                                                       | The adapter, device, command queue and fence, and `run`, which records,    |
|                                                             | runs and waits for setup work, like copying meshes into GPU memory         |
| `AdapterChoice`                                             | The GPU that a monitor is an output of, the fastest one, or WARP           |
| `Presenter`                                                 | A window's swap chain, with `poll_ready`, `begin` and `present`            |
| `Frame`                                                     | What a frame draws into: its command list, its target, and two views of it |
| `Offscreen`                                                 | A texture to draw frames into without a window                             |
| `Readback`, `Image`                                         | A copy of a frame's pixels that the CPU reads                              |
| `write_png`                                                 | Writes an image as a PNG, with the Windows Imaging Component               |
| `UploadBuffer`                                              | A buffer in an upload heap that stays mapped                               |
| `DescriptorHeap`                                            | A descriptor heap and its descriptors' handles                             |
| `create_buffer`, `create_texture`, `transition`, `bytecode` | The rest of the basics                                                     |

## Frames

Every frame's target is `TARGET_FORMAT`, 8-bit BGRA, with two views: `Frame::view`, which stores
colors as they are, for the interface, whose colors are sRGB already, and `Frame::srgb_view`, which
encodes linear colors to sRGB, for 3D shading like SimplePaint's. `DEPTH_FORMAT` is for 3D views'
depth buffers. A frame's target is in the render target state while its commands are recorded.

## Pacing

The swap chain is a flip model one, with a frame latency waitable object and a maximum frame
latency of 1, so no frame waits behind another in DXGI's queue, and only one frame is on the GPU at
a time. `Presenter::poll_ready` says, without blocking, whether the GPU has finished the last frame
and DXGI can take another, so the main loop can keep handling messages while it waits, and read
input only once a frame can start. Frames present without waiting for vertical sync, and may tear
if `allow_tearing` and the system allow it. Alt+Enter's exclusive fullscreen is off.

The GPU that draws is the one that the window's monitor is an output of: drawing with another would
add a copy between GPUs to every frame, and keep Windows from flipping frames straight to the screen.

## The debug layer

Debug builds turn on the D3D12 debug layer, if Windows' optional "Graphics Tools" feature is
installed, and `Gpu::report_messages` writes what it says to standard error. The main loop calls
it every frame; RunInBackground's `--debug-build` puts it in `<program>Errors.log`.
