#include "sha256.h"
#include <map>
#include <cuda.h>
#include <stdio.h>
#include "cuda_util.h"

std::map<int, uint32_t *> context_idata[2];
std::map<int, uint32_t *> context_odata[2];
std::map<int, cudaStream_t> context_streams[2];
std::map<int, uint32_t *> context_tstate[2];
std::map<int, uint32_t *> context_ostate[2];
std::map<int, uint32_t *> context_hash[2];

#define HASH_SIZE (8 * sizeof(uint32_t))

int main(int argc, const char* argv[]) {
    int thrd_id = 0;
    int throughput = 128;
    int stream = 0;
    uint32_t cpu_midstate[8] = {0};

    if (argc != 4) {
        printf("Usage: gpuverify <num_blocks> <num verify loops> <input>\n");
        return 1;
    }

    fprintf(stderr, "starting:\n");
    int num_blocks = strtol(argv[1], nullptr, 10);
    int num_verify = strtol(argv[2], nullptr, 10);
    int input = strtol(argv[3], nullptr, 10);

    //uint32_t* h_pdata = (uint32_t*)calloc(num_blocks * 16, sizeof(uint32_t));

    context_idata[stream][0] = NULL;
    cudaMalloc(&context_idata[stream][0], 32 * sizeof(uint32_t));
    //cudaMemset(&context_idata[stream][0], strtol(argv[1], nullptr, 10), 32 * sizeof(uint32_t));

    context_odata[stream][0] = NULL;
    cudaMalloc(&context_odata[stream][0], 32 * sizeof(uint32_t));
    //cudaMemset(&context_odata[stream][0], strtol(argv[1], nullptr, 10), 32 * sizeof(uint32_t));

    context_ostate[stream][0] = NULL;
    cudaMalloc(&context_ostate[stream][0], 32 * sizeof(uint32_t));

    context_tstate[stream][0] = NULL;
    cudaMalloc(&context_tstate[0][0], 32 * sizeof(uint32_t));
 
    context_hash[stream][0] = NULL;
    cudaMalloc(&context_hash[stream][0], 8 * sizeof(uint32_t));

    uint32_t* d_hash = NULL;
    cudaMalloc(&d_hash, num_blocks * 8 * sizeof(uint32_t));

    cudaStream_t cudaStream;
    cudaStreamCreate(&cudaStream);
    context_streams[stream][0] = cudaStream;

    uint8_t h_pdata[65] = "AnatolyYakovenko11/2/201712pmPSTAnatolyYakovenko11/2/201712pmPST";

    size_t input_size_bytes = num_blocks * 16 * sizeof(uint32_t);
    //memset(h_pdata, input, input_size_bytes);
    uint32_t* d_pdata = nullptr;
    cudaMalloc(&d_pdata, input_size_bytes);
    checkCudaErrors(cudaMemcpy(d_pdata, h_pdata, input_size_bytes, cudaMemcpyHostToDevice));
    //cudaMemset(d_pdata, strtol(argv[1], nullptr, 10), 20 * sizeof(uint32_t));
    memset(h_pdata, 0, input_size_bytes);
    cudaMemcpy(h_pdata, d_pdata, input_size_bytes, cudaMemcpyDeviceToHost);
    for (int i = 0; i < 64/4; i++) {
        printf("%x ", ((uint32_t*)h_pdata)[i]);
    }
    printf("\n");

    printf("starting verify\n");

    uint32_t* h_hash = (uint32_t*)calloc(num_blocks * HASH_SIZE, 1);

    prepare_sha256(thrd_id, cpu_midstate);
    sha256_verify(d_pdata, d_hash, num_blocks, num_verify);

    cudaDeviceSynchronize();

    cudaMemcpy(h_hash, d_hash, num_blocks * HASH_SIZE, cudaMemcpyDeviceToHost);

    for (int i = 0; i < (num_blocks * HASH_SIZE) / sizeof(uint32_t); i++) {
        if (i % 8 == 0) {
            printf("\n");
        }
        printf("%08x ", h_hash[i]);
    }
    printf("\n");
 
    return 0;
}
