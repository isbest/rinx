BUILD:=./build
RUST_KERNEL_OUT=./build/i386_32-rnix_os/debug
SRC:=.

$(BUILD)/boot/%.bin: $(SRC)/boot/%.asm
	$(shell mkdir -p $(dir $@))
	nasm -f bin $< -o $@


$(RUST_KERNEL_OUT)/rnix: $(SRC)/src/*.rs
	cargo build

$(BUILD)/system.bin: $(RUST_KERNEL_OUT)/rnix
	objcopy -O binary $< $@

$(BUILD)/master.img: $(BUILD)/boot/boot.bin \
					 $(BUILD)/boot/loader.bin \
					 $(BUILD)/system.bin
	yes | bximage -q -hd=16 -func=create -sectsize=512 -imgmode=flat $@
	dd if=$(BUILD)/boot/boot.bin of=$@ bs=512 count=1 conv=notrunc
	dd if=$(BUILD)/boot/loader.bin of=$@ bs=512 count=4 seek=2 conv=notrunc
	dd if=$(BUILD)/system.bin of=$@ bs=512 count=200 seek=10 conv=notrunc
.PHONY: clean
clean:
	rm -rf $(BUILD)
	rm -rf ./target
	rm -rf *.ini

.PHONY: bochs
bochs: $(BUILD)/master.img

.PHONY: qemu
qemu: $(BUILD)/master.img
	qemu-system-i386 \
		-m 32M \
		-boot c \
		-hda $<

.PHONY: qemu-gdb
qemu-gdb: $(BUILD)/master.img
	qemu-system-i386 \
		-s -S \
		-m 32M \
		-boot c \
		-hda $<
