# Graphics

The Direct3D 12 renderer that every KGames program's frames go through, and helpers for the
buffers and textures that programs make for their own draws.

The renderer owns the device, a flip model swap chain for the window, and the pacing of
frames, which gets input to the screen as soon as possible:

- It draws with the GPU that the window's monitor is connected to.
- One frame is in flight at a time, and DXGI's queue holds at most one presented frame.
- `Renderer::is_ready` doesn't block. A program's main loop spins on it. Once it's true,
  `Renderer::begin_frame` resets the command list, transitions the back buffer to a render
  target, and sets it as the render target, with the viewport and scissor rectangle. Only then
  does the program read input and record its draws, so the input is as new as it can be.
- `Renderer::present` doesn't wait for vertical sync, and lets frames tear if the driver allows
  it and the program asks for it.

[GameInputDemo's README](../../Program/GameInputDemo/README.md#input-latency) explains each of
these choices.

Each frame, a program records its draws into `Renderer::command_list`, between `begin_frame`
and `present`. The back buffer has two views: `render_target`, as it's stored, for colors that are
already sRGB, like the theme's; and `srgb_render_target`, which encodes linear colors to sRGB,
for shaders like SimplePaint's. A renderer made with a depth buffer also has `depth_stencil`.

| Path                  | Contents                                                                 |
| --------------------- | ------------------------------------------------------------------------ |
| `Source/Renderer.rs`  | The device, swap chain, back buffer views, depth buffer and frame pacing |
| `Source/Resources.rs` | Buffers, mapped upload buffers, textures, and resource barriers          |
