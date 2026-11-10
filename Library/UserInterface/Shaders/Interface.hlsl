// Draws the interface. Every shape and glyph is an instance of one quad, whose
// pixel shader computes its coverage from a signed distance, so that edges are
// antialiased and corners round at any size. Colors are sRGB and
// premultiplied, and blend in sRGB, as Windows' own controls do.
//
// The instances are in a structured buffer, laid out like Painter.h's
// Instance, and the pixel shader gets its instance from the vertex shader,
// flat, rather than reading the buffer for every pixel.

#define ROOT_SIGNATURE \
    "RootConstants(num32BitConstants=4, b0)," \
    "SRV(t0)," \
    "SRV(t1)," \
    "SRV(t2)," \
    "DescriptorTable(SRV(t3), visibility=SHADER_VISIBILITY_PIXEL)"

#define KIND_SHAPE 0
#define KIND_SHADOW 1
#define KIND_GLYPH 2
#define KIND_LINE 3
#define KIND_ARC 4
#define KIND_POLYGON 5

static const float TAU = 6.28318531;

// DirectWrite's alpha correction for a gamma of 1.8, its default without
// ClearType Tuner settings, as Windows Terminal's DWrite_GetGammaRatios
// gives it.
static const float4 GAMMA_RATIOS = float4(0.148054421, -0.894594550, 1.47590804, -0.324668258);

cbuffer Frame : register(b0) {
    float2 viewport;
    float2 unused;
};

struct Instance {
    float4 bounds;
    float4 radii;
    float4 border;
    float4 fillLine;
    float4 borderLine;
    uint4 colors;
    uint kind;
    uint3 padding;
};

StructuredBuffer<Instance> instances : register(t0);
// Each clip's left, top, right and bottom.
StructuredBuffer<float4> clips : register(t1);
// Polygons' corners.
StructuredBuffer<float2> points : register(t2);
Texture2D<float> atlas : register(t3);

struct Varyings {
    float4 position : SV_Position;
    float2 pixel : PIXEL;
    float2 local : LOCAL;
    nointerpolation float4 bounds : BOUNDS;
    nointerpolation float4 radii : RADII;
    nointerpolation float4 border : BORDER;
    nointerpolation float4 fillLine : FILL_LINE;
    nointerpolation float4 borderLine : BORDER_LINE;
    nointerpolation uint4 colors : COLORS;
    nointerpolation uint kind : KIND;
    nointerpolation float4 clip : CLIP;
};

float4 unpack(uint color) {
    return float4(color & 0xFF, (color >> 8) & 0xFF, (color >> 16) & 0xFF, color >> 24) / 255.0;
}

[RootSignature(ROOT_SIGNATURE)]
Varyings VSMain(uint vertex : SV_VertexID, uint index : SV_InstanceID) {
    Instance instance = instances[index];
    uint kind = instance.kind & 0xFFFF;
    float2 corner = float2(vertex & 1, vertex >> 1);
    float2 position;
    float2 local = 0;
    bool alignedToAxes = true;
    if (kind == KIND_LINE) {
        // A box around the segment, a pixel bigger than the line.
        float2 from = instance.bounds.xy;
        float2 to = instance.bounds.zw;
        float halfWidth = max(instance.radii.x, 1.0) * 0.5 + 1.0;
        float2 direction = to - from;
        float segmentLength = length(direction);
        direction = segmentLength > 0 ? direction / segmentLength : float2(1, 0);
        float2 normal = float2(-direction.y, direction.x);
        float along = lerp(-halfWidth, segmentLength + halfWidth, corner.x);
        float across = lerp(-halfWidth, halfWidth, corner.y);
        position = from + direction * along + normal * across;
        local = float2(along, across);
        alignedToAxes = false;
    } else if (kind == KIND_ARC) {
        float reach = instance.bounds.z + instance.bounds.w * 0.5 + 1.0;
        position = instance.bounds.xy + (corner * 2 - 1) * reach;
    } else if (kind == KIND_SHADOW) {
        float margin = instance.radii.y * 3.0 + abs(instance.radii.z) + 1.0;
        position = lerp(instance.bounds.xy - margin, instance.bounds.zw + margin, corner);
    } else if (kind == KIND_GLYPH) {
        position = lerp(instance.bounds.xy, instance.bounds.zw, corner);
    } else {
        // A pixel around the shape, for its antialiased edge.
        position = lerp(instance.bounds.xy - 1.0, instance.bounds.zw + 1.0, corner);
    }
    float4 clip = clips[instance.kind >> 16];
    if (alignedToAxes) {
        // Nothing outside the clip draws, so the quad needn't reach past it.
        position = clamp(position, clip.xy, clip.zw);
    }
    Varyings output;
    output.position = float4(position / viewport * float2(2, -2) + float2(-1, 1), 0, 1);
    output.pixel = position;
    output.local = local;
    output.bounds = instance.bounds;
    output.radii = instance.radii;
    output.border = instance.border;
    output.fillLine = instance.fillLine;
    output.borderLine = instance.borderLine;
    output.colors = instance.colors;
    output.kind = kind;
    output.clip = clip;
    return output;
}

// The signed distance from a point to a box around the origin, with a radius
// at each corner: top left, top right, bottom right, bottom left.
float roundedBox(float2 p, float2 halfSize, float4 radii) {
    float radius = p.x < 0 ? (p.y < 0 ? radii.x : radii.w) : (p.y < 0 ? radii.y : radii.z);
    float2 q = abs(p) - halfSize + radius;
    return min(max(q.x, q.y), 0) + length(max(q, 0)) - radius;
}

float coverage(float distance) {
    return saturate(0.5 - distance);
}

// A linear gradient's color at the point, between the line's two ends, and
// the end colors past them, like XAML's LinearGradientBrush.
float4 gradient(float2 p, float4 line_, uint start, uint end) {
    float2 axis = line_.zw - line_.xy;
    float t = saturate(dot(p - line_.xy, axis) / max(dot(axis, axis), 1e-6));
    return lerp(unpack(start), unpack(end), t);
}

// A rounded box's Gaussian shadow, after Evan Wallace's closed form for a
// blurred box, with the corners integrated by sampling.
float2 erf2(float2 x) {
    float2 s = sign(x);
    float2 a = abs(x);
    x = 1.0 + (0.278393 + (0.230389 + 0.078108 * (a * a)) * a) * a;
    x *= x;
    return s - s / (x * x);
}

float gaussian(float x, float sigma) {
    return exp(-(x * x) / (2.0 * sigma * sigma)) / (sqrt(TAU) * sigma);
}

float shadowRow(float x, float y, float sigma, float radius, float2 halfSize) {
    float delta = min(halfSize.y - radius - abs(y), 0.0);
    float curved = halfSize.x - radius + sqrt(max(0.0, radius * radius - delta * delta));
    float2 integral = 0.5 + 0.5 * erf2((x + float2(-curved, curved)) * (sqrt(0.5) / sigma));
    return integral.y - integral.x;
}

float boxShadow(float2 p, float2 halfSize, float sigma, float radius) {
    float low = p.y - halfSize.y;
    float high = p.y + halfSize.y;
    float start = clamp(-3.0 * sigma, low, high);
    float end = clamp(3.0 * sigma, low, high);
    float step = (end - start) / 6.0;
    float y = start + step * 0.5;
    float value = 0.0;
    for (int i = 0; i < 6; i++) {
        value += shadowRow(p.x, p.y - y, sigma, radius, halfSize) * gaussian(y, sigma) * step;
        y += step;
    }
    return value;
}

float4 PSMain(Varyings input) : SV_Target {
    float2 p = input.pixel;
    float4 clip = input.clip;
    if (p.x < clip.x || p.y < clip.y || p.x >= clip.z || p.y >= clip.w) {
        discard;
    }
    uint kind = input.kind;

    if (kind == KIND_SHAPE) {
        float2 center = (input.bounds.xy + input.bounds.zw) * 0.5;
        float2 halfSize = (input.bounds.zw - input.bounds.xy) * 0.5;
        float outer = coverage(roundedBox(p - center, halfSize, input.radii));
        float inner = outer;
        float4 widths = input.border;
        if (any(widths > 0)) {
            // Inside the border: the box less each side's width, with its
            // corners less round by the wider of their two sides.
            float2 innerMin = input.bounds.xy + widths.xy;
            float2 innerMax = input.bounds.zw - widths.zw;
            float2 innerCenter = (innerMin + innerMax) * 0.5;
            float2 innerHalf = max((innerMax - innerMin) * 0.5, 0);
            float4 innerRadii = max(input.radii - float4(max(widths.x, widths.y), max(widths.z, widths.y), max(widths.z, widths.w), max(widths.x, widths.w)), 0);
            inner = all(innerHalf > 0) ? coverage(roundedBox(p - innerCenter, innerHalf, innerRadii)) : 0;
            inner = min(inner, outer);
        }
        float4 fill = gradient(p, input.fillLine, input.colors.x, input.colors.y);
        float4 stroke = gradient(p, input.borderLine, input.colors.z, input.colors.w);
        return fill * inner + stroke * (outer - inner);
    }

    if (kind == KIND_SHADOW) {
        float2 center = (input.bounds.xy + input.bounds.zw) * 0.5;
        float2 halfSize = (input.bounds.zw - input.bounds.xy) * 0.5;
        float radius = min(input.radii.x, min(halfSize.x, halfSize.y));
        float sigma = input.radii.y;
        float value = boxShadow(p - center - float2(0, input.radii.z), halfSize, sigma, radius);
        float outside = 1.0 - coverage(roundedBox(p - center, halfSize, radius.xxxx));
        return unpack(input.colors.x) * saturate(value) * outside;
    }

    if (kind == KIND_GLYPH) {
        int2 texel = int2(input.radii.xy + floor(p - input.bounds.xy));
        float alpha = atlas.Load(int3(texel, 0));
        // Direct2D's grayscale blending: its enhanced contrast, then its
        // alpha correction for the text's intensity, which make blending
        // the coverage in sRGB look like blending it at a gamma of 1.8.
        float contrast = input.radii.z;
        float intensity = input.radii.w;
        alpha = alpha * (contrast + 1.0) / (alpha * contrast + 1.0);
        float4 g = GAMMA_RATIOS;
        alpha = alpha + alpha * (1.0 - alpha) * ((g.x * intensity + g.y) * alpha + (g.z * intensity + g.w));
        return unpack(input.colors.x) * alpha;
    }

    if (kind == KIND_LINE) {
        float segmentLength = length(input.bounds.zw - input.bounds.xy);
        // A line thinner than a pixel draws a pixel wide, as faint as it's
        // thin.
        float width = input.radii.x;
        float halfWidth = max(width, 1.0) * 0.5;
        float along = input.local.x;
        float across = input.local.y;
        float distance;
        if (input.radii.y > 0.5) {
            float nearest = clamp(along, 0, segmentLength);
            distance = length(float2(along - nearest, across)) - halfWidth;
        } else {
            distance = max(abs(across) - halfWidth, max(-along, along - segmentLength));
        }
        return unpack(input.colors.x) * coverage(distance) * min(width, 1.0);
    }

    if (kind == KIND_ARC) {
        float2 offset = p - input.bounds.xy;
        float radius = input.bounds.z;
        float halfWidth = input.bounds.w * 0.5;
        float start = input.radii.x;
        float sweep = input.radii.y;
        // Clockwise from straight up.
        float angle = atan2(offset.x, -offset.y);
        angle = angle < 0 ? angle + TAU : angle;
        float relative = angle - start;
        relative = relative < 0 ? relative + TAU : relative;
        float distance;
        if (relative <= sweep || sweep >= TAU - 0.0001) {
            distance = abs(length(offset) - radius) - halfWidth;
        } else {
            // Round ends.
            float2 first = float2(sin(start), -cos(start)) * radius;
            float2 last = float2(sin(start + sweep), -cos(start + sweep)) * radius;
            distance = min(length(offset - first), length(offset - last)) - halfWidth;
        }
        return unpack(input.colors.x) * coverage(distance);
    }

    if (kind == KIND_POLYGON) {
        uint first = (uint)input.radii.x;
        uint count = (uint)input.radii.y;
        float distance = -1e9;
        for (uint i = 0; i < count; i++) {
            float2 a = points[first + i];
            float2 b = points[first + (i + 1) % count];
            float2 edge = b - a;
            float edgeLength = length(edge);
            if (edgeLength <= 0) {
                continue;
            }
            float2 outward = float2(edge.y, -edge.x) / edgeLength;
            distance = max(distance, dot(p - a, outward));
        }
        return unpack(input.colors.x) * coverage(distance);
    }

    return 0;
}
