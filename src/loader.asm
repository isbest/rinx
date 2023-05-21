[org 0x1000]

; 魔数
dw 0x55aa

; 打印字符串
mov si, loading
call print

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

