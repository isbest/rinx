### os

### 编译nasm
```shell
nasm -f bin boot.asm -o boot.bin
```
### 创建硬盘镜像
```
bximage -q -hd=16 -func=create -sectsize=512 -imgmode=flat master.img
```
```text
ata0-master: type=disk, path="master.img", mode=flat
```
### 写入代码到硬盘
```shell
dd if=boot.bin of=master.img bs=512 count=1 conv=notrunc
```
