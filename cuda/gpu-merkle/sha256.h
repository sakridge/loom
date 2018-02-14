#ifndef SHA256_H
#define SHA256_H

#include <stdint.h>

extern "C" void prepare_sha256(int thr_id, uint32_t cpu_midstate[8]);
extern "C" void pre_sha256(int thr_id, int stream, uint32_t nonce, int throughput, uint32_t* pdata);
extern "C" void post_sha256(int thr_id, int stream, int throughput);
extern "C" void sha256_verify(uint32_t* pdata, uint32_t* g_ostate, int num_sha_blocks, int num_iterations);

#endif // #ifndef SHA256_H
