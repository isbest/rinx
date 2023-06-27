use crate::kernel::interrupts::clock::init_clock;
use crate::kernel::interrupts::handler_entry::InterruptEntry;
use crate::kernel::interrupts::idt::init_idt;
use crate::kernel::interrupts::pic::pic_controller::init_pic;
use crate::kernel::interrupts::pic::{PIC_M_DATA, PIC_S_DATA};
use core::arch::asm;
use x86::bits32::eflags::{self, EFlags};

pub mod clock;
pub mod entry;
pub mod handler;
pub mod handler_entry;
pub mod idt;
pub mod pic;

/// IDT的大小
pub const IDT_SIZE: usize = 256;
/// 异常中断向量入口的大小
pub const ENTRY_SIZE: usize = 0x30;
/// 外中断主片开始的向量
pub const IRQ_MASTER_NR: usize = 0x20;
/// 外中断从片开始的向量
pub const IRQ_SLAVE_NR: usize = 0x28;

/// 外中断的bit索引
pub const IRQ_CLOCK: u8 = 0;
// 时钟
pub const IRQ_KEYBOARD: u8 = 1;
// 键盘
pub const IRQ_CASCADE: u8 = 2;
// 8259 从片控制器
pub const IRQ_SERIAL_2: u8 = 3;
// 串口 2
pub const IRQ_SERIAL_1: u8 = 4;
// 串口 1
pub const IRQ_PARALLEL_2: u8 = 5;
// 并口 2
pub const IRQ_FLOPPY: u8 = 6;
// 软盘控制器
pub const IRQ_PARALLEL_1: u8 = 7;
// 并口 1
pub const IRQ_RTC: u8 = 8;
// 实时时钟
pub const IRQ_REDIRECT: u8 = 9;
// 重定向 IRQ2
pub const IRQ_MOUSE: u8 = 12;
// 鼠标
pub const IRQ_MATH: u8 = 13;
// 协处理器 x87
pub const IRQ_HARDDISK: u8 = 14;
// ATA 硬盘第一通道
pub const IRQ_HARDDISK2: u8 = 15; // ATA 硬盘第二通道

/// 初始化中断
pub fn init_interrupt() {
    init_pic();
    init_idt();
    init_clock();
}

/// 开启外中断
pub fn enable_interrupt(enable: bool) {
    if enable {
        unsafe { asm!("sti", options(nomem, nostack)) }
    } else {
        unsafe { asm!("cli", options(nomem, nostack)) }
    }
}

/// 外中断是否开启 true开启,false未开启
pub fn eflags_if() -> bool {
    let if_bit: u32;

    unsafe {
        asm!(
        "pushf",
        "pop %eax",
        "mov {}, %eax",
        out(reg) if_bit,
        options(att_syntax)
        );
    }

    (if_bit & (1 << 9)) != 0
}

/// 屏蔽外 中断执行函数
pub fn without_interrupt<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let saved_interrupt_flag = are_enabled();
    if saved_interrupt_flag {
        enable_interrupt(false);
    }
    let ret = f();
    if saved_interrupt_flag {
        enable_interrupt(true);
    }
    ret
}

/// 判断外中断有没有开启
pub fn are_enabled() -> bool {
    unsafe { eflags::read().contains(EFlags::FLAGS_IF) }
}

// 开启或者关闭某个外中断
pub fn set_interrupt_mask(mut irq: u8, enable: bool) {
    use x86::io::{inb, outb};
    assert!((0..=15).contains(&irq));

    // 判断应该往主片写还是往从片写
    let port = if irq < 8 {
        PIC_M_DATA
    } else {
        irq -= 8;
        PIC_S_DATA
    };

    if enable {
        unsafe {
            // 有效置为0,切不能影响别的中断
            outb(port, inb(port) & !(1 << irq));
        }
    } else {
        // 无效置为1,且不影响别的中断
        unsafe {
            outb(port, inb(port) | (1 << irq));
        }
    }
}
