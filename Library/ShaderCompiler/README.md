# ShaderCompiler

Compiles HLSL to DXIL with `dxc.exe` from the newest installed Windows SDK, for build scripts
and tests. Library crates and programs compile their shaders in their `Build.rs` with it, as a
build dependency, and embed the DXIL in their exe with `include_bytes!`.

`shader_compiler::compile` returns what `dxc.exe` wrote if it fails, like the line of a
mistake in the shader. `shader_compiler::rerun_if_changed` has Cargo run a build script again
when anything in the shader's folder changes, which covers the files it includes from there.
