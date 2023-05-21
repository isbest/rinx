[org 0x1000]

; 魔数
dw 0x55aa

; 打印字符串
mov si, loading
call print

detect_memory:
  xor ebx, ebx; 清空ebx

  ; 结构体缓存位置
  mov ax, 0
  mov es, ax
  mov edi, ards_buffer

  mov edx, 0x534d4150; 固定签名 SMAP
  .next:
    ; 子功能号
    mov eax, 0xe820
    ; ards 结构的大小
    mov ecx, 20

    ; 调用0x15系统调用
    int 0x15

    ; 如果CF置位,表示出错
    jc error

    ; 将缓存指针指向下一个
    add di, cx
    ; 将结构体数量+1
    inc word [ards_count]

    ; 是0表示检测结束
    cmp ebx, 0
    ; 否则继续检测下一个
    jnz .next

    mov si, detecting
    call print

    xchg bx, bx
    
    ; 结构体数量
    mov cx, [ards_count]
    ; 结构体指针
    mov si, 0
  .show:
    ; 解析结构体,基地址低32位
    ; 0-4低32位 4-8 高32位
    mov eax, [ards_buffer + si]
    ; 内存长度
    ; 8-12 内存长度低32位, 12-16内存长度高12位
    mov ebx, [ards_buffer + si + 8]
    ; 内存类型
    mov edx, [ards_buffer + si + 16]
    ; 指针+20 指向下一个结构体
    add si, 20
    xchg bx, bx
    ; loop 是根据ecx的值循环
    loop .show

; 阻塞
jmp $;

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

loading: 
  db "Loading Rnix...", 10, 13, 0; 10 \n 13 \r
detecting: 
  db "Detecting Memory Success !", 10, 13, 0; 10 \n 13 \r

error:
  mov si, .msg
  call print
  hlt; cpu停机
  jmp $
  .msg db "Loading Error!!!", 10, 13, 0

ards_count:
  dw 0
ards_buffer:
