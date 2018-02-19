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
    //uint32_t h_pdata[20] = {0};
    int num_blocks = 128;
    uint32_t* h_pdata = (uint32_t*)calloc(num_blocks * 16, sizeof(uint32_t));
    uint32_t cpu_midstate[8] = {0};

    if (argc != 3) {
        printf("Usage: gpumerkle <init value> <num bytes>\n");
        return 1;
    }

    fprintf(stderr, "starting:\n");
    int input = strtol(argv[1], nullptr, 10);
    int input2 = strtol(argv[2], nullptr, 10);
    h_pdata[1] = input;

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

    memset(h_pdata, input, input2 * sizeof(uint32_t));
    uint32_t* d_pdata = nullptr;
    cudaMalloc(&d_pdata, 20 * sizeof(uint32_t));
    checkCudaErrors(cudaMemcpy(d_pdata, h_pdata, 20 * sizeof(uint32_t), cudaMemcpyHostToDevice));
    //cudaMemset(d_pdata, strtol(argv[1], nullptr, 10), 20 * sizeof(uint32_t));
    memset(h_pdata, 0, 20 * sizeof(uint32_t));
    cudaMemcpy(h_pdata, d_pdata, 20 * sizeof(uint32_t), cudaMemcpyDeviceToHost);
    for (int i = 0; i < 20; i++) {
        printf("%x ", h_pdata[i]);
    }
    printf("\n");

    int num_levels = 2;
    int levels[] = {2, 2};
    sha256_merkle(d_pdata, d_hash, levels, num_levels);

    uint32_t* h_hash = (uint32_t*)calloc(num_blocks * HASH_SIZE, 1);

    cudaMemcpy(h_hash, context_hash[stream][thrd_id], HASH_SIZE, cudaMemcpyDeviceToHost);

    cudaDeviceSynchronize();

    for (int i = 0; i < (num_blocks * HASH_SIZE) / sizeof(uint32_t); i++) {
        printf("%08x ", h_hash[i]);
    }
    printf("\n");
 
    return 0;
}
