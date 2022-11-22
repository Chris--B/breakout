#include "Public.h"

using namespace breakout;
using namespace metal;

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

struct VsInstancedQuadOut {
    float4 pos [[position]];
};

vertex VsInstancedQuadOut vs_instanced_quad(
           uint         vid             [[vertex_id]],
    device View         const& view     [[buffer(BUFFER_IDX_VIEW)]],
    device PerQuad      const* per_quad [[buffer(BUFFER_IDX_PER_QUAD)]]
) {
    const uint vert_id = vid % array_len(quad_verts);
    const uint quad_id = vid / array_len(quad_verts);
    const PerQuad quad = per_quad[quad_id];

    // Construct world space position
    float2 vert = quad_verts[vert_id] + float2(0.5);
    float2 pos  = quad.pos + (quad.dims * vert);
    pos *= 0.5;

    VsInstancedQuadOut out;
    out.pos = view.matViewProj * float4(pos, 0., 1.);
    return out;
}

fragment float4 fs_instanced_quad(
           uint               prim_id     [[primitive_id]],
           float2             barycentric [[barycentric_coord]],
    device PerQuad     const* per_quad    [[buffer(BUFFER_IDX_PER_QUAD)]]
) {
    const uint quad_id = prim_id / 2; // 2 prims per quad
    const PerQuad quad = per_quad[quad_id];

    if (quad.flags & PER_QUAD_FLAGS_AS_CIRCLE) {
        float d = length(barycentric - float2(0.5));
        if (d > 0.5) {
            // Don't generate fragments outside of the circle
            discard_fragment();
        }
    }

    return float4(quad.color, 1.0);
}
