
constant int BUFFER_IDX_VIEW = 1;
constant int BUFFER_IDX_PER_QUAD = 2;

template <typename T, size_t N>
size_t array_len(constant T const (&arr)[N]) {
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

struct View {
    float todo;
};

struct PerQuad {
    float2 pos;
    float2 scale;
    float3 color;
};
static_assert(sizeof(PerQuad) == 4 * (2 + 2 + 4), "Unexpected size of PerQuad");

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
