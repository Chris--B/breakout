#include "Public.h"

// Helper to get a compile-time array's length
template <typename T, size_t N>
constexpr size_t array_len(constant T const (&arr)[N]) {
    return N;
}

constant constexpr float2 quad_verts[] = {
    float2(-0.5f,  0.5f),
    float2( 0.5f, -0.5f),
    float2(-0.5f, -0.5f),

    float2(-0.5f,  0.5f),
    float2( 0.5f, -0.5f),
    float2( 0.5f,  0.5f),
};

struct VsOut {
    float4 pos [[position]];
    float3 color;
};

vertex VsOut vs_instanced_quad(
           unsigned int        vid      [[vertex_id]],
    device View         const& view     [[buffer(BUFFER_IDX_VIEW)]],
    device PerQuad      const* per_quad [[buffer(BUFFER_IDX_PER_QUAD)]]
) {
    // Use this vert ID since we're soft instancing our vertices
    const size_t vert_id = vid % array_len(quad_verts);
    const size_t quad_id = vid / array_len(quad_verts);

    const PerQuad quad = per_quad[quad_id];

    // "world" space position
    float2 pos = quad_verts[vert_id];
    pos *= quad.scale;
    pos += quad.pos;

    pos *= 0.5; // lol should fix this

    // TODO: viewport transform

    VsOut out;
    out.pos = float4(pos, 0., 1.);
    out.color = quad.color;

    return out;
}

fragment float4 fs_instanced_quad(
    VsOut input [[stage_in]]
) {
    return float4(input.color, 1.0);
}
