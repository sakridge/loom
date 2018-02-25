#include <stdint.h>
#include <stdio.h>
#include <assert.h>
#include <stdlib.h>
#include <inttypes.h>

#include "sha256.h"

//#ifndef sha256 
//#define sha256 sha256_avx
//#endif

static void print_block(const uint32_t* blkptr) {
    printf("block %04x%04x%04x%04x\n", blkptr[0], blkptr[1], blkptr[2], blkptr[3]);
    printf("      %04x%04x%04x%04x\n", blkptr[4], blkptr[5], blkptr[6], blkptr[7]);
    printf("      %04x%04x%04x%04x\n", blkptr[8], blkptr[9], blkptr[10], blkptr[11]);
    printf("      %04x%04x%04x%04x\n", blkptr[12], blkptr[13], blkptr[14], blkptr[15]);
}

#define USE_RDTSC

#ifdef USE_RDTSC
static inline uint64_t rdtsc()
{
    unsigned int hi, lo;
    __asm__ volatile("rdtsc" : "=a" (lo), "=d" (hi));
    return ((uint64_t)hi << 32) | lo;
}

typedef struct {
    uint64_t count;
} perftime_t;

#elif defined(USE_CLOCK_GETTIME)
#include <time.h>
typedef struct timespec perftime_t;
#else
#include <sys/time.h>
typedef struct timeval perftime_t;
#endif

static int get_time(perftime_t* t) {
#ifdef USE_RDTSC
    t->count = rdtsc();
    return 0;
#elif defined(USE_CLOCK_GETTIME)
    return clock_gettime(CLOCK_MONOTONIC_RAW, t);
    //return clock_gettime(CLOCK_PROCESS_CPUTIME_ID, t);
#else
    return gettimeofday(t, NULL /* timezone */);
#endif
}

static double get_us(const perftime_t* time) {
#ifdef USE_RDTSC
    return time->count;
#elif defined(USE_CLOCK_GETTIME)
    return ((time->tv_nsec/1000) + (double)time->tv_sec * 1000000);
#else
    return (time->tv_usec + (double)time->tv_sec * 1000000);
#endif
}

static double get_diff(const perftime_t* start, const perftime_t* end) {
    return get_us(end) - get_us(start);
}

int main(int argc, char *argv[]) {
    uint32_t state[8];
    uint8_t block[64] = "AnatolyYakovenko11/2/201712pmPSTAnatolyYakovenko11/2/201712pmPST";
    uint32_t *blkptr = (void*)block;
    uint64_t i=0;

    if (argc != 4) {
        printf("Usage: %s <iterations> <impl> <output file>\n", argv[0]);
        return 1;
    }

    uint32_t iterations = strtol(argv[1], NULL, 10);
    uint32_t impl = strtol(argv[2], NULL, 10);

    FILE *f = fopen(argv[3], "a+");
    if (f == NULL) {
        printf("Couldn't file open: %s\n", argv[1]);
        return 1;
    }

    sha256_init_digest(state);

    perftime_t start, now;
    if (!fseek(f, -40, SEEK_END)) {
    	assert(8 == fread(&i, 1, 8, f));
        i = i<<20;
    	assert(32 == fread(blkptr, 1, 32, f));
        blkptr[8] =  blkptr[0];
        blkptr[9] =  blkptr[1];
        blkptr[10] = blkptr[2];
        blkptr[11] = blkptr[3];
        blkptr[12] = blkptr[4];
        blkptr[13] = blkptr[5];
        blkptr[14] = blkptr[6];
        blkptr[15] = blkptr[7];
    	assert(0 == fseek(f, 0, SEEK_END));
	}
    print_block(blkptr);
    printf("state %04x%04x%04x%04x\n", state[0], state[1], state[2], state[3]);
    printf("      %04x%04x%04x%04x\n", state[4], state[5], state[6], state[7]);

    int run_bench = 0;
    if (run_bench) {
        int num_blocks = 1000000;
        void * speed_blk = calloc(64, num_blocks);
        for (int j = 0; j < 5; j++) {
            sha256_impl(impl, speed_blk, state, num_blocks);
        }

        for (int j = 0; j < 4; j++) {
            double impl_total = 0.0;
            for (int k = 0; k < 5; k++) {
                assert(!get_time(&start));
                sha256_impl(j, speed_blk, state, num_blocks);
                assert(!get_time(&now));
                double total = get_diff(&start, &now);
                impl_total += total;
                printf("impl: %d %f state: %x\n", j, total, state[0]);
                ((uint32_t*)speed_blk)[0] = k;
            }
            printf("total: %f\n", impl_total);
        }
        free(speed_blk);
    }

    sha256_init_digest(state);

    assert(!get_time(&start));
    for (;;++i) {
        sha256_iterate_impl(impl, blkptr, state, iterations);

        {
            double total;
            uint64_t ix = i * iterations;
            assert(!get_time(&now));
            total = get_diff(&start, &now);
            start = now;

            fwrite(&ix, 8, 1, f);
            fwrite(blkptr, 4, 8, f);
            fflush(f);

            print_block(blkptr);
            printf("speed i: %" PRIu64 " total: %f ms us/iteration: %f\n",
                   i, total / 1000, (total/iterations));
        }
    }
}

