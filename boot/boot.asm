[org 0x7c00]
; 在内存的0x7c00处

; 设置屏幕模式为文本模式,清除屏幕
mov ax, 3
int 0x10

; 初始化寄存器,避免异常情况
mov ax, 0
mov ds, ax
mov es, ax
mov ss, ax
; 将栈初始化到0x7c00处
mov sp, 0x7c00

; bochs 的魔术断点
; xchg bx, bx

mov si, booting
call print

; edi存储读取的数据放到目标内存地址
; ecx 起始扇区
; bl 读取多少个扇区
mov edi, 0x1000
mov ecx, 2
mov bl, 4
call read_disk

; 魔数校验
cmp word [0x1000], 0x55aa
jnz error

jmp 0:0x1002

; 阻塞
jmp $

; LBA28,总共能访问128G磁盘
; 0x1F0 16位的端口,用来读写数据
; 0x1F1 检测前一次操作的错误
; 0x1F2 读写扇区的数量
; 0x1F3 起始扇区的0-7位
; 0x1F4 起始扇区的8-15位
; 0x1F4 起始扇区的16-23位
; 0x1F6 0-3位是起始扇区的24-27位
;   第4位 是0表示主盘,否则是从盘
;   第6位 0表示CHS模式,1表示LBA模式
;   第5,7 位固定是1
; 0x1F7 
;   out操作
;     0xEC表示识别硬盘
;     0x20表示读硬盘
;     0x30表示写硬盘
;   in操作
;     是一个8bit的数据
;     第0位是错误
;     第3位是数据准备完毕
;     第7位是BSY 表示硬盘是否繁忙
read_disk:
  ;设置读写扇区数量
  mov dx, 0x1f2;读取扇区数量的端口
  mov al, bl; 获取读取扇区数的参数
  out dx, al; 写入端口

  ; 设置起始扇区,低8位
  inc dx; 0x1F3 设置0-8位
  mov al, cl; 起始扇区的中8位
  out dx, al; 写入前8位

  ; 设置起始扇区,中8位
  inc dx; 0x1F4 设置0-8位
  shr ecx, 8; eax右移8位
  mov al, cl; 起始扇区的高8位
  out dx, al; 写入前8位

  ; 设置起始扇区高8位
  inc dx; 0x1F5 设置0-8位
  shr ecx, 8; eax右移8位
  mov al, ch; 起始扇区的高8位
  out dx, al; 写入前8位

  ; 设置起始扇区高四位,及读取模式
  inc dx; 0x1F6
  shr ecx, 8
  and cl, 0b0000_1111; 将高四位置为0
  mov al, 0b1110_0000; 5,6,7都是1,第6位表示LBA模式,第4位为0表示是主盘
  ; 合并al,cl
  or al, cl
  out dx, al
  
  ; 输出
  inc dx; 0x1F7
  mov al, 0x20; 表示读硬盘
  out dx, al

  ; 清空ecx,为读取设置参数
  xor ecx, ecx
  mov cl, bl; 得到读写扇区的数量

  ; 读取数据主流程
  ; 先等待数据准备完毕
  ; 然后去读取
  .read:
    push cx; 保存cx,下面的read_to_mem修改了cx
    call .read_waits; 等待数据准备完毕
    call .read_to_mem; 读取一个扇区
    pop cx; 恢复cx
    loop .read
  ret 

  ; 等待数据准备完毕
  .read_waits:
    mov dx, 0x1f7
    .read_check: ; 检查数据是否准备完毕
      in al, dx; 将数据读到al寄存器
      ; 做一些延时
      jmp $ + 2
      jmp $ + 2
      jmp $ + 2
      ; 只获取第三位和第七位
      ; 第三位表示数据是否准备完毕
      ; 第七位表示硬盘是否忙
      and al, 0b1000_1000;
      ; 判断数据是否准备完毕
      cmp al, 0b0000_1000;
      ; 没有准备完毕,继续空转cpu,直到准备完毕
      jnz .read_check
    ; 准备完毕,check返回,开始读硬盘
    ret

  ; 将数据读到内存
  .read_to_mem:
    mov dx, 0x1f0; 读数据的端口
    mov cx, 256; 一个扇区256个字
    .read_w:
      in ax, dx
      ; 做一下延时
      jmp $ + 2
      jmp $ + 2
      jmp $ + 2
      ; 将数据读到edi
      mov [edi], ax
      ; 将edi挪动到下一个存放数据的地址
      add edi, 2
      loop .read_w
    ret

; 打印字符串
print:
  mov ah, 0x0e
  .next:
    ; 获取当前字符
    mov al, [si]
    ; 比较当前字符是不是0
    cmp al, 0
    ; 是的话就代表已经到达字符串末尾,直接ret
    jz .done
    ; 否则,打印当前字符
    int 0x10
    ; 自增si,去打印下一个字符
    inc si
    ; 回到next继续打印
    jmp .next
  .done:
    ; 函数返回
    ret 

booting: 
  db "Booting Rnix...", 10, 13, 0; 10 \n 13 \r

error:
  mov si, .msg
  call print
  hlt; cpu停机
  jmp $
  .msg: db "Booting Error!!!", 10, 13, 0

; 剩余字节填充0,第一个扇区512字节,去掉魔数还剩下510字节
; $是当前行代码的偏移地址, $$是当前段起始地址
; 510 减去已经使用的字节数量,然后填充0
times 510 - ($ - $$) db 0

; 魔数 或者dw 0xaa55,cpu是小端存储的,低位在低地址,高位在高地址
db 0x55,0xaa
