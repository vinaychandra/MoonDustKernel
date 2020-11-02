# Adapted from https://gitlab.com/bztsrc/bootboot/-/blob/master/images/Makefile

# overall disk image size in megabytes (128M)
DISKSIZE	=	128
# boot partition size in kilobytes (16M)
BOOTSIZE	=	16384
# boot partition FAT type (16 or 32). Note that smallest possible FAT32 is 33M
# FAT 32 doesn't currently work (need to fix mformat)
BOOTTYPE	=	16
# platform, options: x86_64
PLATFORM	=	x86_64
# the path to OVMF.fd (for testing with EFI)
ifneq ("$(wildcard /usr/share/qemu/OVMF.fd)","")
    OVMF		=	/usr/share/qemu/OVMF.fd
else
	OVMF		=	./others/bootboot/OVMF-pure-efi-$(PLATFORM).fd
endif

# Choose font to link at core level (psf file in others/fonts)
FONT		=	bootboot

SOURCES := $(shell find ./src -name '*.rs')
USERSPACE := $(wildcard ./userspace/*)

.PHONY: userspace

all: userspace target/$(PLATFORM)-moondust/debug/moondust-kernel target/initrd.rom target/disk-$(PLATFORM).img

# compile the image creator
target/mkimg: others/bootboot/mkimg.c
	@mkdir ./target 2>/dev/null | true
	gcc -ansi -pedantic -Wall -Wextra -g $^ -o $@

# Generate bindings for bootboot. (Converts *.h files to rust bindings)
src/bootboot.rs: others/bootboot/bootloader.h others/bootboot/bootboot.h
	bindgen $< -o $@ --use-core --ctypes-prefix=custom_ctypes
	echo "pub mod custom_ctypes { pub type c_int = i64; pub type c_uchar = u64; }" >> $@

userspace:
# cargo xbuild --target ./triplets/$(PLATFORM)-moondust-user.json --workspace --exclude moondust-kernel
	cargo build -Z build-std=core,alloc --target ./triplets/$(PLATFORM)-moondust-user.json --workspace --exclude moondust-kernel

# Kernel build
target/$(PLATFORM)-moondust/debug/moondust-kernel: $(SOURCES)
	@mkdir ./target 2>/dev/null | true
	cp ./others/fonts/$(FONT).psf ./target/font.psf
	cd ./target; objcopy -O elf64-x86-64 -B i386 -I binary font.psf font.o || true ; cd ..
# https://github.com/rust-lang/wg-cargo-std-aware/issues/41
	cargo build -Z build-std=core,alloc --target ./triplets/$(PLATFORM)-moondust.json -p moondust-kernel
# cargo xbuild --target ./triplets/$(PLATFORM)-moondust.json -p moondust-kernel

# create an initial ram disk image with the kernel inside
target/initrd.bin: target/$(PLATFORM)-moondust/debug/moondust-kernel userspace
	@mkdir ./target/initrd ./target/initrd/sys 2>/dev/null | true
	cp ./$< ./target/initrd/sys/core
	@mkdir ./target/initrd ./target/initrd/userspace 2>/dev/null | true
	cp $(USERSPACE:./userspace/%=./target/$(PLATFORM)-moondust-user/debug/%) ./target/initrd/userspace/
	@cd ./target/initrd && (find . | cpio -H ustar -o | gzip > ../initrd.bin) && cd ../..

# Create the bootloader partition.
target/bootpart.bin: target/initrd.bin
	dd if=/dev/zero of=./target/bootpart.staging.bin bs=1024 count=$(BOOTSIZE)
	mformat -i ./target/bootpart.staging.bin  -v "EFI System" -T $$(( $(BOOTSIZE) * 2 )) -M 512 -h 64 -s 32 ::
	mmd -i ./target/bootpart.staging.bin ::/BOOTBOOT
	
	mcopy -i ./target/bootpart.staging.bin ./others/bootboot/bootboot.bin ::/BOOTBOOT/LOADER || true
	mmd -i ./target/bootpart.staging.bin ::/EFI
	mmd -i ./target/bootpart.staging.bin ::/EFI/BOOT
	mcopy -i ./target/bootpart.staging.bin ./others/bootboot/bootboot.efi ::/EFI/BOOT/BOOTX64.EFI || true
	
	echo -e "screen=800x600\nkernel=sys/core\n" >CONFIG || true
	mcopy -i ./target/bootpart.staging.bin CONFIG ::/BOOTBOOT/CONFIG || true
	rm CONFIG
	mcopy -i ./target/bootpart.staging.bin ./target/initrd.bin ::/BOOTBOOT/INITRD || true
	mv ./target/bootpart.staging.bin ./target/bootpart.bin

check-image: target/$(PLATFORM)-moondust/debug/moondust-kernel target/mkimg
	cd target; ./mkimg check ../$^

# create hybrid disk / cdrom image or ROM image
target/disk-$(PLATFORM).img: target/mkimg target/bootpart.bin
	cd target; ./mkimg disk $(DISKSIZE) ./disk-$(PLATFORM).img; cd ..

target/initrd.rom: target/mkimg target/initrd.bin
	cd target/; ./mkimg rom; cd ..

# test the disk image
rom: target/initrd.rom
	qemu-system-x86_64 -option-rom ./others/bootboot/bootboot.bin -option-rom ./target/initrd.rom -serial stdio

bios: target/disk-$(PLATFORM).img
	qemu-system-x86_64 -drive file=./target/disk-x86_64.img,format=raw -serial stdio

bios-wait: target/disk-$(PLATFORM).img
	qemu-system-x86_64 -drive file=./target/disk-x86_64.img,format=raw -serial stdio -s -S

efi: target/disk-$(PLATFORM).img
	qemu-system-x86_64 -bios $(OVMF) -m 64 -drive file=./target/disk-x86_64.img,format=raw -serial stdio

efi-wait: target/disk-$(PLATFORM).img
	qemu-system-x86_64 -bios $(OVMF) -m 64 -drive file=./target/disk-x86_64.img,format=raw -serial stdio -s -S

clean:
	cargo clean


