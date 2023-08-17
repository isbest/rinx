  .section .text.entry
  .globl _start
 _start:
  mov esp, 0x10000

  push ebx
  push eax

  call memory_init
  call init_gdt
  call init_tss
  call init_mem_mapping
  call rust_main