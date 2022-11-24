#pragma once

#include <metal_stdlib>

using metal::float4x4;

#define CheckSize(T, Size) static_assert(sizeof(T) == (Size), "Type is an unexpected size");
#define CheckAlign(T, Align) static_assert(alignof(T) == (Align), "Type has an unexpected alignment");

namespace breakout {
    constant int BUFFER_IDX_VIEW = 1;
    constant int BUFFER_IDX_PER_QUAD = 2;

    struct View {
        float4x4 matViewProj;
    };
    CheckSize(View, 4 * (16));
    CheckAlign(View, 16);

    struct PerQuad {
        packed_float3 pos;
        packed_float2 dims;
        packed_float3 color;
        uint32_t      flags;
    };
    CheckSize(PerQuad, 4 * (3 + 3 + 2 + 1));
    CheckAlign(PerQuad, 4);

    /// Default behavior for our Quad renderer. Renders a single-colored quad.
    constant constexpr uint32_t PER_QUAD_FLAGS_NONE = 0;

    /// When this bit is set, the quad is rendered as an oval instead of a rectangle
    constant constexpr uint32_t PER_QUAD_FLAGS_AS_CIRCLE = (1 << 0);
}
