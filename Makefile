# platform, options: x86_64
PLATFORM	=	x86_64
HOST		=  $(shell uname -s)
# the path to OVMF.fd (for testing with EFI)
ifneq ("$(wildcard /usr/share/qemu/OVMF.fd)","")
    OVMF		=	/usr/share/qemu/OVMF.fd
else
	OVMF		=	./others/bootboot/OVMF-pure-efi-$(PLATFORM).fd
endif

KERNEL_SOURCES := $(shell find ./src -name '*.rs')
USERSPACE := $(wildcard ./userspace/*)

.PHONY: userspace

all: userspace target/$(PLATFORM)-moondust/debug/moondust-kernel
run: efi

# Generate bindings for bootboot. (Converts *.h files to rust bindings)
src/bootboot.rs: others/bootboot/bootloader.h others/bootboot/bootboot.h
	bindgen $< -o $@ --use-core --ctypes-prefix=custom_ctypes
	echo "pub mod custom_ctypes { pub type c_int = i64; pub type c_uchar = u64; }" >> $@

userspace:
	cargo build --target ./triplets/$(PLATFORM)-moondust-user.json --workspace --exclude moondust-kernel

# Kernel build
target/$(PLATFORM)-moondust/debug/moondust-kernel: $(KERNEL_SOURCES)
	@mkdir ./target 2>/dev/null | true
	# https://github.com/rust-lang/wg-cargo-std-aware/issues/41
	cargo build --target ./triplets/$(PLATFORM)-moondust.json -p moondust-kernel

# create an initial ram disk image with the kernel inside
target/disk-$(PLATFORM).img: target/$(PLATFORM)-moondust/debug/moondust-kernel
	@mkdir ./target/initrd ./target/initrd/sys ./target/initrd/sys ./target/initrd/userspace 2>/dev/null | true
	cp ./$< ./target/initrd/sys/core
	cd ./target/initrd/sys; echo -e "screen=1280x768\nkernel=sys/core\n" >config || true;
	cp $(USERSPACE:./userspace/%=./target/$(PLATFORM)-moondust-user/debug/%) ./target/initrd/userspace/
	./others/bootboot/mkbootimg-$(HOST) ./others/bootboot/mkimgconfig.json $@

check-image: target/$(PLATFORM)-moondust/debug/moondust-kernel
	./others/bootboot/mkbootimg-${HOST} check $^

efi: target/disk-$(PLATFORM).img
	qemu-system-x86_64 -bios $(OVMF) -m 128 -drive file=./target/disk-x86_64.img,format=raw -serial stdio

efi-wait: target/disk-$(PLATFORM).img
	qemu-system-x86_64 -bios $(OVMF) -m 128 -drive file=./target/disk-x86_64.img,format=raw -serial stdio -s -S

clean:
	cargo clean
