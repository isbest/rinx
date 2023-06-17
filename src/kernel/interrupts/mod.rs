use crate::kernel::interrupts::handler_entry::InterruptEntry;
use crate::kernel::interrupts::idt::init_idt;
use crate::kernel::interrupts::pic::controller::init_pic;
use core::arch::asm;
use x86::bits32::eflags;
use x86::bits32::eflags::EFlags;

pub mod entry;
pub mod handler;
pub mod handler_entry;
pub mod idt;
pub mod pic;

/// IDT的大小
pub const IDT_SIZE: usize = ENTRY_SIZE;
/// 异常中断向量入口的大小
pub const ENTRY_SIZE: usize = 0x30;
/// 外中断开始的向量
pub const EXT_START_VECTOR: usize = 0x20;

pub fn init_interrupt() {
    init_pic();
    init_idt();
}

pub fn sti() {
    unsafe { asm!("sti", options(nomem, nostack)) }
}

pub fn cli() {
    unsafe { asm!("cli", options(nomem, nostack)) }
}

pub fn without_interrupt<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let saved_interrupt_flag = are_enabled();
    if saved_interrupt_flag {
        cli();
    }
    let ret = f();
    if saved_interrupt_flag {
        sti();
    }
    ret
}

pub fn are_enabled() -> bool {
    unsafe { eflags::read().contains(EFlags::FLAGS_IF) }
}
