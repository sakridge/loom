#include "sha256.h"
#include "sha256_asm.h"
#include <string.h>
#include <stdio.h>
#include <stdlib.h>

static uint32_t default_sha256_state[8] = {0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                                           0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19};

void sha256_init_digest(uint32_t digest[8]) {
    memcpy(digest, default_sha256_state, 8 * sizeof(uint32_t));
}

void sha256_impl(int impl, void *in, uint32_t digest[8], uint32_t num_blocks)
{
    switch (impl) {
        case SHA256_IMPL_AVX:
            sha256_avx(in, digest, num_blocks);
            break;
        case SHA256_IMPL_AVX2_RORX_X8:
            sha256_rorx_x8ms(in, digest, num_blocks);
            break;
        case SHA256_IMPL_AVX2_RORX_X2:
            sha256_rorx(in, digest, num_blocks);
            break;
        case SHA256_IMPL_SSE4:
            sha256_sse4(in, digest, num_blocks);
            break;
        default:
            return;
    }
}

void sha256(void *in, uint32_t digest[8], uint32_t num_blocks)
{
    // TODO: better way to pick this
    int impl;
    if (num_blocks > 10000) {
        impl = SHA256_IMPL_AVX2_RORX_X8;
    } else {
        impl = SHA256_IMPL_AVX2_RORX_X2;
    }
    sha256_impl(impl, in, digest, num_blocks);
}

void sha256_iterate_impl(int impl, void* in, uint32_t digest[8], int64_t num_iterations)
{
    uint32_t *blkptr = (uint32_t*)in;

    uint32_t state[8];
    memcpy(state, digest, 8 * sizeof(uint32_t));

    for (int64_t i = 0; i < num_iterations; ++i) {
        sha256_impl(impl, in, state, 1);

        memcpy(blkptr, state, 8 * sizeof(uint32_t));
        memcpy(&blkptr[8], state, 8 * sizeof(uint32_t));

        memcpy(state, digest, 8 * sizeof(uint32_t));
    }
}

void sha256_iterate(void* in, uint32_t digest[8], int64_t num_iterations)
{
    // TODO: better way to pick this
    int impl = SHA256_IMPL_AVX2_RORX_X2;
    sha256_iterate_impl(impl, in, digest, num_iterations);
}
