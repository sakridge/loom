#ifndef SHA256_H
#define SHA256_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define SHA256_IMPL_AVX 0
#define SHA256_IMPL_AVX2_RORX_X8 1
#define SHA256_IMPL_AVX2_RORX_X2 2
#define SHA256_IMPL_SSE4 3

void sha256_init_digest(uint32_t digest[8]);
void sha256(void *input_data, uint32_t digest[8], uint32_t num_blocks);
void sha256_impl(int impl, void *in, uint32_t digest[8], uint32_t num_blocks);
void sha256_iterate(void *input_data, uint32_t digest[8], int64_t num_iterations);
void sha256_iterate_impl(int impl, void* in, uint32_t digest[8], int64_t num_iterations);

#ifdef __cplusplus
}
#endif

#endif
