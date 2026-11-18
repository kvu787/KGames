# WindowsApi

What KGames adds to the Windows API bindings in
[Library/windows-api-raw](../windows-api-raw/README.md), which are only what `windows-bindgen`
generates. Code that goes beyond the bindings, and that more than one crate uses, goes here.
Bindings never do: add them to WindowsApiRaw.

## Layout

| Path         | Contents                                                                                |
| ------------ | --------------------------------------------------------------------------------------- |
| `src/lib.rs` | `check_handle`, which turns a Win32 function's null handle into the thread's last error |
