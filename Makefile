# This Makefile builds:
#   1. A Rust static library (libkfs.a)
#   2. A bootable ELF kernel (kernel.bin)
#   3. A bootable ISO image (os.iso) with GRUB
#
# Requirements:
#   - rustc + cargo
#   - ld.lld
#   - nasm
#   - grub-mkrescue, xorriso, mtools
#   - qemu (for `make run`)
#
# Usage:
#   make         # build os.iso
#   make run     # boot in QEMU
#   make clean   # clean all artifacts

BUILD_MODE := --release
GRUB_ARCH  := i386-pc

NASM    := nasm
LD      := ld.lld
QEMU    := qemu-system-i386

BUILD   := build
ISO     := os.iso
BOOTDIR := src/boot
KERNEL_BIN := $(BUILD)/boot/kernel.bin
GRUB_DIR   := $(BUILD)/boot/grub
GRUB_CFG   := $(BOOTDIR)/grub.cfg
LINKER_LD  := $(BOOTDIR)/linker.ld
BOOT_O     := $(BOOTDIR)/boot.o
LIBKFS_OUT := $(BUILD)/libkfs.a
ARTIFACTS := artifacts
DOCKER_IMAGE := kfs-builder

.PHONY: all kernel iso run clean docker

all: iso

# ----- directory prerequisites -----
$(BUILD):
	mkdir -p $@

$(BUILD)/boot: | $(BUILD)
	mkdir -p $@

$(GRUB_DIR): | $(BUILD)/boot
	mkdir -p $@

# 1) Build Rust staticlib and copy exact artifact Cargo produced
$(LIBKFS_OUT): | $(BUILD)
	@echo "[CARGO] building libkfs.a"
	@artifact=$$(cargo build $(BUILD_MODE) --message-format=json \
	  | sed -n 's/.*"filenames":\["\([^"]*libkfs\.a\)".*/\1/p' \
	  | tail -n1); \
	if [ -z "$$artifact" ]; then \
	  echo "ERROR: libkfs.a not found in Cargo artifacts"; exit 1; \
	fi; \
	cp "$$artifact" "$(LIBKFS_OUT)"; \
	echo "[OK] copied $$artifact -> $(LIBKFS_OUT)"

# 2) Assemble Multiboot stub
$(BOOT_O): $(BOOTDIR)/boot.asm | $(BUILD)/boot
	$(NASM) -felf32 $< -o $@

# 3) Link final ELF kernel
$(KERNEL_BIN): $(BOOT_O) $(LINKER_LD) $(LIBKFS_OUT) | $(BUILD)/boot
	$(LD) -m elf_i386 -T $(LINKER_LD) $(BOOT_O) $(LIBKFS_OUT) -o $(KERNEL_BIN) --gc-sections

kernel: $(KERNEL_BIN)
	grub-file --is-x86-multiboot $(KERNEL_BIN) || \
	(echo "Not multiboot!" && exit 1)
	@echo "[OK] multiboot header present"

# 4) Bootable ISO with GRUB
iso: kernel | $(GRUB_DIR)
	cp $(GRUB_CFG) $(GRUB_DIR)/grub.cfg
	grub-mkrescue -o $(ISO) $(BUILD)

# 5) Run in QEMU
run: iso
	$(QEMU) -cdrom $(ISO) -serial stdio -no-reboot -cpu qemu32

# 6) Clean
clean:
	rm -rf target $(BUILD) $(ISO) $(ARTIFACTS) $(BOOT_O)
	docker rmi $(DOCKER_IMAGE)

docker:
	@echo "[DOCKER] Building image $(DOCKER_IMAGE)"
	@docker build -t $(DOCKER_IMAGE) .
	@echo "[DOCKER] Building ISO inside container"
	@docker run --rm \
		-v "$(PWD):/workspace" \
		-w /workspace \
		$(DOCKER_IMAGE) \
		bash -lc 'make'

qemu:
	$(QEMU) -cdrom $(ISO) -serial stdio -no-reboot -cpu qemu32
