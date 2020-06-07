PLATFORM = x86_64

ifeq ($(PLATFORM),x86_64)
	TARGET_NAME				= 	x86_64-moondust
	BOOTIMAGE_KERNEL_FILE 	= 	bootimage-moondust-kernel
endif

TARGET_JSON     		=   ./triplets/$(TARGET_NAME).json

.PHONY: build clean check doc run asm

build:
	cargo xbuild --target $(TARGET_JSON)

check:
	cargo xcheck --target $(TARGET_JSON)

clean:
	cargo clean

doc:
	cargo xdoc --target $(TARGET_JSON)

asm:
	RUSTFLAGS="--emit asm -Z asm-comments" cargo xbuild --target $(TARGET_JSON)

ifeq ($(PLATFORM),x86_64)
run: # Important: Running this from terminal will cause qemu to hang because it waits for gdb
	cargo bootimage --target $(TARGET_JSON)
	qemu-system-x86_64 -drive format=raw,file=target/$(TARGET_NAME)/debug/$(BOOTIMAGE_KERNEL_FILE).bin -s -S -smp 2
endif