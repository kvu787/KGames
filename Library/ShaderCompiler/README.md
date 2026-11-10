# ShaderCompiler

Compiles HLSL to DXIL with `dxc.exe` from the Windows SDK, in two halves: the build's, in
`KGames.targets`, for a project's own shaders, and this library's, for tests that compile
shaders while they run.

Libraries and programs compile their shaders as they build. A project names each with a `Shader`
item in its `.vcxproj`, like
`<Shader Include="Shaders\Interface.hlsl" Name="Interface" Arguments="-O3 -WX" />`, and
`KGames.targets` compiles the file's `VSMain` and `PSMain` for Shader Model 6.0, with the
`dxc.exe` of the Windows SDK that the project builds with, into `<Name>Vertex.h` and
`<Name>Pixel.h` in the project's generated folder,
`target\Intermediate\<Configuration>\<Project>\Generated`. Each defines the shader's bytes as an
array of that name, like `InterfaceVertex`, which the project's code includes, so the DXIL is in
its exe. They're compiled again when a `.hlsl` or `.hlsli` file in the shader's folder changes,
which covers the files it includes from there. A shader that doesn't compile fails the build.

`shader_compiler::compile` compiles one entry point of a shader into a file while a program
runs, with the `dxc.exe` of the newest installed Windows SDK, which `shader_compiler::find_dxc`
picks. It returns what `dxc.exe` wrote if it fails, like the line of a mistake in the shader.
[Test/SimplePaint](../../Test/SimplePaint) compiles SimplePaint's shader with it, with material
counts that must compile and one that mustn't.
