mod gate;
pub mod sys_call;

use crate::kernel::system_call::gate::{default_sys_call, SYSTEM_CALL_TABLE};
use crate::kernel::system_call::sys_call::{task_sleep, task_yield, SysCall};
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

pub(crate) extern "C" fn sys_call_check(sys_call_number: u32) {
    assert!(
        sys_call_number as usize <= SYS_CALL_SIZE,
        "system call number:{} must be less than:{}",
        sys_call_number,
        SYS_CALL_SIZE
    );
}

pub(crate) fn sys_call(sys_call: SysCall) -> u32 {
    let res: u32;
    unsafe {
        asm!(
        "int $0x80",
        inout("eax") sys_call as u32 => res,
        options(att_syntax)
        );
        res
    }
}

pub(crate) fn sys_call_1(sys_call: SysCall, arg1: u32) -> u32 {
    let res: u32;
    unsafe {
        asm!(
        "int $0x80",
        inout("eax") sys_call as u32 => res,
        in("ebx") arg1,
        options(att_syntax)
        );
        res
    }
}

pub fn init_system_call() {
    unsafe {
        SYSTEM_CALL_TABLE[SysCall::Test as usize] = default_sys_call;
        SYSTEM_CALL_TABLE[SysCall::Yield as usize] = task_yield;
        SYSTEM_CALL_TABLE[SysCall::Sleep as usize] = task_sleep;
    }
}
