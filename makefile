BUILD:=./build
RUST_KERNEL_OUT=./build/x86-rnix_os/debug
SRC:=.

$(BUILD)/boot/%.bin: $(SRC)/boot/%.asm
	$(shell mkdir -p $(dir $@))
	nasm -f bin $< -o $@

$(BUILD)/boot/loader.bin: $(SRC)/boot/loader.asm $(BUILD)/system.bin
	$(shell mkdir -p $(dir $@))
	nasm -f bin $< -o $@ -DKERNEL_SIZE=$$(stat -c%s "$(BUILD)/system.bin")

.PHONY: $(RUST_KERNEL_OUT)/rnix
$(RUST_KERNEL_OUT)/rnix: $(SRC)/x86-rnix_os.json
	cargo build

$(BUILD)/system.bin: $(RUST_KERNEL_OUT)/rnix
	objcopy -O binary $< $@

$(BUILD)/master.img: $(BUILD)/boot/boot.bin \
					 $(BUILD)/boot/loader.bin \
					 $(BUILD)/system.bin
	yes | bximage -q -hd=16 -func=create -sectsize=512 -imgmode=flat $@
	dd if=$(BUILD)/boot/boot.bin of=$@ bs=512 count=1 conv=notrunc
	dd if=$(BUILD)/boot/loader.bin of=$@ bs=512 count=4 seek=2 conv=notrunc
	test -n "$$(find $(BUILD)/system.bin -size -500k)"
	dd if=$(BUILD)/system.bin of=$@ bs=512 count=1000 seek=10 conv=notrunc
.PHONY: clean
clean:
	rm -rf $(BUILD)
	rm -rf ./target
	rm -rf *.ini

.PHONY: bochs
bochs: $(BUILD)/master.img
	bochs -q

.PHONY: qemu
qemu: $(BUILD)/master.img
	qemu-system-i386 \
		-m 32M \
		-drive file=$(BUILD)/master.img,if=ide,index=0,media=disk,format=raw

.PHONY: qemu-gdb
qemu-gdb: $(BUILD)/master.img
	qemu-system-i386 \
		-gdb tcp::9001 -S \
		-m 32M \
		-drive file=$(BUILD)/master.img,if=ide,index=0,media=disk,format=raw
