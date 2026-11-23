// Draws the tester's frames: each glyph, and each rectangle, like the text
// box and the caret, is an instance of one quad, which covers whole pixels,
// with integer corners. A glyph's quad finds the glyph's pixel under each of
// its pixels with integer math, so each pixel of a glyph is exactly `scale`
// × `scale` pixels of the screen, and nothing is filtered or blended across
// pixels' edges.
//
// The quads are in a structured buffer, laid out like scene.rs' Quad. Colors
// are sRGB, for the frame's view that stores them as they are, and
// premultiplied.

#define ROOT_SIGNATURE \
    "RootConstants(num32BitConstants=4, b0)," \
    "SRV(t0)," \
    "DescriptorTable(SRV(t1), visibility=SHADER_VISIBILITY_PIXEL)"

// A quad's glyph when it's a rectangle of its color.
#define SOLID 0xFFFFFFFF

cbuffer Frame : register(b0) {
    // The target's size in pixels.
    float2 targetSize;
    // A glyph's width in the atlas, where the glyph of character c starts at
    // c × glyphWidth.
    uint glyphWidth;
    uint unused;
};

struct Quad {
    // The pixels that it covers: left and top in, right and bottom out.
    int4 area;
    // Where its glyph's top left pixel is on the screen.
    int2 origin;
    uint scale;
    uint glyph;
    // Red, green, blue and alpha, a byte each, red lowest.
    uint color;
};

StructuredBuffer<Quad> quads : register(t0);
// Each glyph's coverage, the glyphs side by side in one row.
Texture2D<float> atlas : register(t1);

struct Varyings {
    float4 position : SV_Position;
    nointerpolation uint index : INDEX;
};

float4 unpack(uint color) {
    return float4(color & 0xFF, (color >> 8) & 0xFF, (color >> 16) & 0xFF, color >> 24) / 255.0;
}

[RootSignature(ROOT_SIGNATURE)]
Varyings VSMain(uint vertex : SV_VertexID, uint index : SV_InstanceID) {
    Quad quad = quads[index];
    // A strip of 4: top left, top right, bottom left, bottom right.
    float2 corner = float2((vertex & 1) ? quad.area.z : quad.area.x, (vertex & 2) ? quad.area.w : quad.area.y);
    Varyings output;
    output.position = float4(corner.x / targetSize.x * 2.0 - 1.0, 1.0 - corner.y / targetSize.y * 2.0, 0.0, 1.0);
    output.index = index;
    return output;
}

float4 PSMain(Varyings input) : SV_Target {
    Quad quad = quads[input.index];
    float4 color = unpack(quad.color);
    if (quad.glyph != SOLID) {
        // The pixel's own coordinates: its center is at .5, and it's inside
        // the area, which is inside the glyph, so the offset isn't negative.
        int2 pixel = int2(input.position.xy);
        uint2 texel = uint2(pixel - quad.origin) / quad.scale;
        color.a *= atlas.Load(int3(quad.glyph * glyphWidth + texel.x, texel.y, 0));
    }
    return float4(color.rgb * color.a, color.a);
}
