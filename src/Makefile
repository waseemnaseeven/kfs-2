all: compile lunch

compile: 
	cargo build -Z build-std=core,alloc --target=i386.json --release
	nasm	-felf32 ./src/boot/boot.asm -o ./src/boot/boot.o
	ld -m elf_i386 -T ./src/boot/linker.ld ./src/boot/boot.o ./target/i386/release/libkfs.a -o \
	src/boot/kernel.bin
	mkdir -p build/boot/grub
	cp src/boot/grub.cfg build/boot/grub/grub.cfg
	mv src/boot/kernel.bin build/boot/kernel.bin
	grub-mkrescue -o os.iso build

lunch:	all
	qemu-system-i386 -cdrom os.iso
	
docker:
	docker build -t kfs-1 .

clean:
	rm -r target
	rm -r build
	rm src/boot/boot.o
	rm os.iso
run:
	qemu-system-i386 -cdrom os.iso

re: clean all

.PHONY: all clean run re

# qemu-system-i386 -kernel build/boot/kernel.bin pour lancer directement le kernel