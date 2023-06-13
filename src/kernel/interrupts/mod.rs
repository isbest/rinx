use crate::kernel::interrupts::handler_entry::InterruptEntry;

pub mod entry;
pub mod handler;
pub mod handler_entry;
pub mod idt;

/// IDT的大小
pub const IDT_SIZE: usize = 256;
/// 异常中断向量入口的大小
pub const ENTRY_SIZE: usize = 0x20;
