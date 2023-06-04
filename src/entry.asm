  .section .text.entry
  .globl _start
 _start:
  mov esp, 0x10000

  push ebx
  push eax

  call clear_bss
  call init_gdt
  call memory_init
  call rust_main