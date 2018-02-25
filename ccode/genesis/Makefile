CFLAGS:=-fPIC -pedantic -Werror -Wall -O3 -std=c99 -march=native -D_POSIX_C_SOURCE=199309L

all: sha256_avx1 libsha256.a

# Check the assembly
sha256.s:sha256.c
	gcc $(CFLAGS) -Wa,-adhln -c sha256.c > sha256.s

sha256.o:sha256.c
	gcc $(CFLAGS) -c sha256.c -o sha256.o

# Library for rust to ingest
libsha256.a:sha256_avx1.o sha256_sse4.o sha256_avx2_rorx8.o sha256_avx2_rorx2.o sha256.o
	ar cr libsha256.a $^

sha256_avx1:*.asm *.c sha256_avx1.o libsha256.a
	gcc $(CFLAGS) -o $@ main.c $@.o -L. -lsha256

sha256_sse4:*.asm main.c sha256_sse4.o sha256.o
	gcc $(CFLAGS) -Dsha256=sha256_sse4 -o $@ main.c $@.o sha256.o

sha256_avx2_rorx8:*.asm *.c sha256_avx2_rorx8.o sha256.o
	gcc $(CFLAGS) -Dsha256=sha256_rorx_x8ms -o $@ main.c $@.o

sha256_avx2_rorx2:*.asm *.c sha256_avx2_rorx2.o sha256.o
	gcc $(CFLAGS) -Dsha256=sha256_rorx -o $@ main.c $@.o

UNAME := $(shell uname)

ifeq ($(UNAME), Linux)
  YASM_FLAGS:=-f x64 -f elf64 -X gnu -g dwarf2 -D LINUX
else ifeq ($(UNAME), Darwin)
  YASM_FLAGS:=-D MACOS -f x64 -f macho64 -X gnu -g null -D LINUX
else
  $(error $(UNAME) "I don't know what this is!")
endif

%.o:%.asm
	yasm $(YASM_FLAGS) -o $@ $^
