use crate::kernel::interrupts::handler_entry::InterruptEntry;
use crate::kernel::interrupts::idt::init_idt;
use crate::kernel::interrupts::pic::controller::init_pic;

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
