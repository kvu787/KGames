// The root signature is embedded in the compiled vertex shader, and the Rust
// side creates it straight from those bytes. No parameters are needed here.
#define ROOT_SIGNATURE "RootFlags(0)"

struct VertexOutput
{
    float4 position : SV_Position;
    float3 color : COLOR;
};

// No vertex buffer: positions come from the vertex index.
[RootSignature(ROOT_SIGNATURE)]
VertexOutput VSMain(uint id : SV_VertexID)
{
    static const float2 positions[3] = { float2(0.0, 0.5), float2(0.5, -0.5), float2(-0.5, -0.5) };
    static const float3 colors[3] = { float3(1, 0, 0), float3(0, 1, 0), float3(0, 0, 1) };

    VertexOutput output;
    output.position = float4(positions[id], 0.0, 1.0);
    output.color = colors[id];
    return output;
}

float4 PSMain(VertexOutput input) : SV_Target
{
    return float4(input.color, 1.0);
}
