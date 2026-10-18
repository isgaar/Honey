CC ?= gcc
CFLAGS ?= -O3 -Wall -Wextra -fPIC -std=c99
LDFLAGS ?= -lm

UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
    SHLIB_EXT = dylib
    SHLIB_FLAGS = -dynamiclib
else
    SHLIB_EXT = so
    SHLIB_FLAGS = -shared -ldl
endif

TARGET_LIB = libhoney_quartz.$(SHLIB_EXT)
TARGET_BENCH = honey_raster_bench

all: $(TARGET_LIB) $(TARGET_BENCH)

$(TARGET_LIB): libhoney_quartz.c
	$(CC) $(CFLAGS) $(SHLIB_FLAGS) -o $@ $< $(LDFLAGS)

$(TARGET_BENCH): honey_raster_bench.c libhoney_quartz.c
	$(CC) $(CFLAGS) -o $@ honey_raster_bench.c libhoney_quartz.c $(LDFLAGS)

clean:
	rm -f $(TARGET_LIB) $(TARGET_BENCH) *.o

test: $(TARGET_BENCH)
	./$(TARGET_BENCH)

.PHONY: all clean test
