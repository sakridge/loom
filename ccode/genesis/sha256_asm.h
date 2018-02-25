#ifndef SHA256_ASM_H
#define SHA256_ASM_H

void sha256_avx(void *input_data, uint32_t digest[8], uint64_t num_blks);
void sha256_rorx_x8ms(void *input_data, uint32_t digest[8], uint64_t num_blks);
void sha256_rorx(void *input_data, uint32_t digest[8], uint64_t num_blks);
void sha256_sse4(void *input_data, uint32_t digest[8], uint64_t num_blks);

#endif
