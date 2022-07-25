#pragma once

#define CheckSize(T, Size) static_assert(sizeof(T) == (Size), "Type is an unexpected size");
#define CheckAlign(T, Align) static_assert(alignof(T) == (Align), "Type has an unexpected alignment");

namespace breakout {
    constant int BUFFER_IDX_VIEW = 1;
    constant int BUFFER_IDX_PER_QUAD = 2;

    struct View {
        float todo;
    };

    struct PerQuad {
        packed_float2 pos;
        packed_float2 scale;
        packed_float3 color;
    };
    CheckSize(PerQuad, 4 * (2 + 2 + 3));
    CheckAlign(PerQuad, 4);
}
