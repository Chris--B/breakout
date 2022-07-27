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
        packed_float2 pos;
        packed_float2 dims;
        packed_float3 color;
    };
    CheckSize(PerQuad, 4 * (2 + 2 + 3));
    CheckAlign(PerQuad, 4);
}
