cmake_minimum_required(VERSION 2.8)
find_package(CUDA QUIET REQUIRED)
 
# Pass options to NVCC
set(
	CUDA_NVCC_FLAGS
	${CUDA_NVCC_FLAGS};
	--ptxas-options=-v -O3 --gpu-architecture=compute_61 -std=c++11
   )

cuda_add_library(sha256 STATIC sha256.cu)

cuda_add_executable(
	gpuverify
	gpuverify.cu)

cuda_add_executable(
	gpumerkle
	main.cu)

target_link_libraries(gpumerkle sha256)
target_link_libraries(gpuverify sha256)
