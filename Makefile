ifeq ($(OS),Windows_NT)
	NAME := $(EXE).exe
else
	NAME := $(EXE)
endif

default:
	cargo rustc --release -p hardfiskur_uci --bin hardfiskur_uci -- \
		-C target-cpu=native --emit link=$(NAME)
