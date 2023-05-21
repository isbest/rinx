  .section .text.entry
  .globl _start
 _start:
  mov esp, 0x10000
  call rust_main