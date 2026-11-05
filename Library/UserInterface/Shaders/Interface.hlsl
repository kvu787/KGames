// Draws the interface: every shape and glyph is an instance of one quad,
// whose pixel shader computes the shape's coverage from its signed distance,
// so edges are antialiased and corners round at any size. Colors are sRGB and
// premultiplied, and blend in sRGB, like Windows' own controls.
//
// Instances are in a structured buffer, laid out like Painter.rs' Instance.

#define ROOT_SIGNATURE \
    "RootConstants(num32BitConstants=4, b0)," \
    "SRV(t0)," \
    "SRV(t1)," \
    "SRV(t2)," \
    "DescriptorTable(SRV(t3), visibility=SHADER_VISIBILITY_PIXEL)," \
    "StaticSampler(s0, filter=FILTER_MIN_MAG_MIP_POINT," \
    " addressU=TEXTURE_ADDRESS_CLAMP, addressV=TEXTURE_ADDRESS_CLAMP)"

#define KIND_SHAPE 0
#define KIND_GRADIENT 1
#define KIND_SHADOW 2
#define KIND_GLYPH 3
#define KIND_LINE 4
#define KIND_ARC 5
#define KIND_POLYGON 6

static const float PI = 3.14159265;
static const float TAU = 6.28318531;

cbuffer Frame : register(b0) {
    float2 viewport;
    float2 unused;
};

struct Instance {
    float4 bounds;
    float4 data;
    uint color;
    uint color2;
    uint color3;
    uint kind;
    float4 params;
};

StructuredBuffer<Instance> instances : register(t0);
// Left, top, right and bottom of each clip rectangle.
StructuredBuffer<float4> clips : register(t1);
// The corners of polygons.
StructuredBuffer<float2> points : register(t2);
Texture2D<float> atlas : register(t3);

// The pixel shader gets its instance from the vertex shader, flat, instead of
// reading the instance buffer, which is in system memory, for every pixel.
struct Varyings {
    float4 position : SV_Position;
    float2 pixel : PIXEL;
    float2 local : LOCAL;
    nointerpolation float4 bounds : BOUNDS;
    nointerpolation float4 data : DATA;
    nointerpolation float4 params : PARAMS;
    nointerpolation uint4 colors : COLORS;
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
    bool axisAligned = true;
    if (kind == KIND_LINE) {
        // A box around the segment, a pixel bigger than the line.
        float2 from = instance.bounds.xy;
        float2 to = instance.bounds.zw;
        float halfWidth = instance.data.x * 0.5 + 1.0;
        float2 direction = to - from;
        float length_ = length(direction);
        direction = length_ > 0 ? direction / length_ : float2(1, 0);
        float2 normal = float2(-direction.y, direction.x);
        float along = lerp(-halfWidth, length_ + halfWidth, corner.x);
        float across = lerp(-halfWidth, halfWidth, corner.y);
        position = from + direction * along + normal * across;
        local = float2(along, across);
        axisAligned = false;
    } else if (kind == KIND_ARC) {
        float reach = instance.bounds.z + instance.bounds.w * 0.5 + 1.0;
        position = instance.bounds.xy + (corner * 2 - 1) * reach;
    } else if (kind == KIND_SHADOW) {
        float margin = instance.data.y * 3.0 + abs(instance.data.z) + 1.0;
        position = lerp(instance.bounds.xy - margin, instance.bounds.zw + margin, corner);
    } else if (kind == KIND_GLYPH) {
        position = lerp(instance.bounds.xy, instance.bounds.zw, corner);
    } else {
        // A pixel around the shape, for its antialiased edge.
        position = lerp(instance.bounds.xy - 1.0, instance.bounds.zw + 1.0, corner);
    }
    if (axisAligned) {
        // Nothing outside the clip draws, so the quad needn't cover it.
        float4 clip = clips[instance.kind >> 16];
        position = clamp(position, clip.xy, clip.zw);
    }
    Varyings output;
    output.position = float4(position / viewport * float2(2, -2) + float2(-1, 1), 0, 1);
    output.pixel = position;
    output.local = local;
    output.bounds = instance.bounds;
    output.data = instance.data;
    output.params = instance.params;
    output.colors = uint4(instance.color, instance.color2, instance.color3, instance.kind);
    output.clip = clips[instance.kind >> 16];
    return output;
}

// The signed distance from a point to a rounded box around the origin, with
// a radius for each corner: top left, top right, bottom right, bottom left.
float roundedBox(float2 p, float2 halfSize, float4 radii) {
    float radius = p.x < 0 ? (p.y < 0 ? radii.x : radii.w) : (p.y < 0 ? radii.y : radii.z);
    float2 q = abs(p) - halfSize + radius;
    return min(max(q.x, q.y), 0) + length(max(q, 0)) - radius;
}

float coverage(float distance) {
    return saturate(0.5 - distance);
}

// A rounded box's shadow, after Evan Wallace's closed form for a Gaussian
// blur of a box, with the corners integrated by sampling.
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
    Instance instance;
    instance.bounds = input.bounds;
    instance.data = input.data;
    instance.params = input.params;
    instance.color = input.colors.x;
    instance.color2 = input.colors.y;
    instance.color3 = input.colors.z;
    instance.kind = input.colors.w;
    uint kind = instance.kind & 0xFFFF;
    float2 p = input.pixel;
    float4 clip = input.clip;
    if (p.x < clip.x || p.y < clip.y || p.x >= clip.z || p.y >= clip.w) {
        discard;
    }

    if (kind == KIND_SHAPE || kind == KIND_GRADIENT) {
        float2 center = (instance.bounds.xy + instance.bounds.zw) * 0.5;
        float2 halfSize = (instance.bounds.zw - instance.bounds.xy) * 0.5;
        float distance = roundedBox(p - center, halfSize, instance.data);
        float outer = coverage(distance);
        if (kind == KIND_GRADIENT) {
            float t = instance.params.x > 0.5
                ? (p.y - instance.bounds.y) / max(instance.bounds.w - instance.bounds.y, 1)
                : (p.x - instance.bounds.x) / max(instance.bounds.z - instance.bounds.x, 1);
            return lerp(unpack(instance.color), unpack(instance.color2), saturate(t)) * outer;
        }
        float width = instance.params.x;
        float inner = width > 0 ? coverage(distance + width) : outer;
        float4 stroke = unpack(instance.color2);
        float band = instance.params.y;
        if (band > 0) {
            // The bottom pixel is the bottom color, which shades into the
            // stroke's over the band, like Fluent's elevation border.
            float fromBottom = instance.bounds.w - p.y;
            stroke = lerp(unpack(instance.color3), stroke, saturate((fromBottom - 1.0) / (band - 1.0)));
        }
        return unpack(instance.color) * inner + stroke * (outer - inner);
    }

    if (kind == KIND_SHADOW) {
        float2 center = (instance.bounds.xy + instance.bounds.zw) * 0.5;
        float2 halfSize = (instance.bounds.zw - instance.bounds.xy) * 0.5;
        float radius = min(instance.data.x, min(halfSize.x, halfSize.y));
        float sigma = instance.data.y;
        float value = boxShadow(p - center - float2(0, instance.data.z), halfSize, sigma, radius);
        // Only outside the surface.
        float outside = 1.0 - coverage(roundedBox(p - center, halfSize, radius.xxxx));
        return unpack(instance.color) * saturate(value) * outside;
    }

    if (kind == KIND_GLYPH) {
        int2 texel = int2(instance.data.xy + floor(p - instance.bounds.xy));
        float alpha = atlas.Load(int3(texel, 0));
        return unpack(instance.color) * alpha;
    }

    if (kind == KIND_LINE) {
        float length_ = length(instance.bounds.zw - instance.bounds.xy);
        // A line thinner than a pixel draws a pixel wide, as faint as it's thin.
        float width = instance.data.x;
        float halfWidth = max(width, 1.0) * 0.5;
        float along = input.local.x;
        float across = input.local.y;
        float distance;
        if (instance.params.x > 0.5) {
            float nearest = clamp(along, 0, length_);
            distance = length(float2(along - nearest, across)) - halfWidth;
        } else {
            distance = max(abs(across) - halfWidth, max(-along, along - length_));
        }
        return unpack(instance.color) * coverage(distance) * min(width, 1.0);
    }

    if (kind == KIND_ARC) {
        float2 offset = p - instance.bounds.xy;
        float radius = instance.bounds.z;
        float halfWidth = instance.bounds.w * 0.5;
        float start = instance.data.x;
        float sweep = instance.data.y;
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
        return unpack(instance.color) * coverage(distance);
    }

    if (kind == KIND_POLYGON) {
        uint first = (uint)instance.data.x;
        uint count = (uint)instance.data.y;
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
        return unpack(instance.color) * coverage(distance);
    }

    return 0;
}
