TARGET_DIR ?= ../../target
PROFILE ?= debug
LIB_DIR := $(TARGET_DIR)/$(PROFILE)
UNAME_S := $(shell uname -s)

ifeq ($(UNAME_S),Darwin)
  RPATH_FLAG := -Wl,-rpath,@loader_path/../../../target/$(PROFILE)
else
  RPATH_FLAG := -Wl,-rpath,'$$ORIGIN/../../../target/$(PROFILE)'
endif

.PHONY: build test clean

build:
	cargo build -p braillify-c

test: build
	$(CC) -std=c11 -Wall -Wextra -Werror -Iinclude tests/smoke.c \
		-L$(LIB_DIR) -lbraillify_c $(RPATH_FLAG) -o tests/smoke
	./tests/smoke

clean:
	$(RM) tests/smoke
