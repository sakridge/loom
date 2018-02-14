#ifndef CUDA_UTIL_H
#define CUDA_UTIL_H

#define LOG_ERR 0
#define applog(err, str, ...)  printf(str, __VA_ARGS__)

#define checkCudaErrors(x) \
{ \
    cudaGetLastError(); \
    x; \
    cudaError_t err = cudaGetLastError(); \
    if (err != cudaSuccess) \
        applog(LOG_ERR, "GPU #%d: cudaError %d (%s) calling '%s' (%s line %d)\n", 0, err, cudaGetErrorString(err), #x, __FILE__, __LINE__); \
}

//applog(LOG_ERR, "GPU #%d: cudaError %d (%s) calling '%s' (%s line %d)\n", device_map[thr_id], err, cudaGetErrorString(err), #x, __FILENAME__, __LINE__);
#endif
