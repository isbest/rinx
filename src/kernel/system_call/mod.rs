mod gate;

use crate::kernel::system_call::gate::SYSTEM_CALL_TABLE;
use core::arch::asm;

pub const SYS_CALL_SIZE: usize = 20;

#[naked]
#[link_section = ".text"]
pub extern "C" fn system_call() {
    unsafe {
        asm!(
            // 验证系统调用号
            "push %eax",
            "call {0}",
            // 弹出栈顶的eax
            "pop %eax",
            // 魔数
            "push $0x20222202",
            "push $0x80",
            // 保存上下文寄存器信息
            "push %ds",
            "push %es",
            "push %fs",
            "push %gs",
            "pusha",
            "push $0x80", // 向中断处理函数传递参数中断向量 vector
            "push %edx", // 第三个参数
            "push %ecx", // 第二个参数
            "push %ebx", // 第一个参数
            // 调用系统调用处理函数，syscall_table 中存储了系统调用处理函数的指针
            "call *{1}(,%eax,4)",
            "add $12, %esp",
            // 修改栈中 %eax 寄存器，设置系统调用返回值
            "mov %eax, 32(%esp)",
            // 中断返回
            "add $4, %esp",
            // 恢复上下文
            "popa",
            "pop %gs",
            "pop %fs",
            "pop %es",
            "pop %ds",
            "add $8, %esp",
            "iret",
            sym sys_call_check,
            sym SYSTEM_CALL_TABLE,
            options(noreturn, att_syntax),
        );
    }
}

pub extern "C" fn sys_call_check(sys_call_number: u32) {
    assert!(
        sys_call_number as usize <= SYS_CALL_SIZE,
        "system call number:{} must be less than:{}",
        sys_call_number,
        SYS_CALL_SIZE
    );
}
