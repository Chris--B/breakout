#pragma once

// C++ & Metal compatibility
#if !defined(__METAL_VERSION__)
    #include <simd/SIMD.h>
    using namespace simd;

    // no-op these MSL-only keywords
    #define constant
    #define device
#endif

// Helper that errors if two types have a mismatched size
// *and* prints both the real and expected sizes.
// A plain `static_assert()` cannot do this.
template <size_t A, size_t B>
struct Private_SizeCheck {
    static_assert(
        A == B,
        "struct sizes must match between C++ & MSL, "
        "double check that they're still right"
    );

    // We use this whole type in a `static_assert` to evaluate it.
    // This value is used to evaluate it, but should never trip the assert.
    constant static constexpr bool value = (A == B);
};
#define CheckSize(Type, Expected) static_assert(Private_SizeCheck<sizeof(Type), Expected>::value, "")

constant int BUFFER_IDX_VIEW = 1;
constant int BUFFER_IDX_PER_QUAD = 2;

struct View {
    float todo;
};
CheckSize(View, 4);

struct PerQuad {
    float2 pos;
    float2 scale;
    float3 color;
};
CheckSize(PerQuad, 4 * (2 + 2 + 4));
