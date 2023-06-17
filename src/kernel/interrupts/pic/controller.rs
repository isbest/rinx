use crate::kernel::interrupts::pic::{PIC_EOI, PIC_M_CTRL, PIC_M_DATA, PIC_S_CTRL, PIC_S_DATA};
use x86::io::outb;

pub fn init_pic() {
    unsafe {
        outb(PIC_M_CTRL, 0b00010001); // ICW1: 边沿触发, 级联 8259, 需要ICW4.
        outb(PIC_M_DATA, 0x20); // ICW2: 起始端口号 0x20
        outb(PIC_M_DATA, 0b00000100); // ICW3: IR2接从片.
        outb(PIC_M_DATA, 0b00000001); // ICW4: 8086模式, 正常EOI

        outb(PIC_S_CTRL, 0b00010001); // ICW1: 边沿触发, 级联 8259, 需要ICW4.
        outb(PIC_S_DATA, 0x28); // ICW2: 起始端口号 0x28
        outb(PIC_S_DATA, 2); // ICW3: 设置从片连接到主片的 IR2 引脚
        outb(PIC_S_DATA, 0b00000001); // ICW4: 8086模式, 正常EOI

        outb(PIC_M_DATA, 0b11111110); // 关闭所有中断
        outb(PIC_S_DATA, 0b11111111); // 关闭所有中断
    }
}

pub fn send_eoi(vector: u32) {
    if _contains(0x20..0x28, vector) {
        unsafe {
            outb(PIC_M_CTRL, PIC_EOI);
        }
    }

    if _contains(0x28..0x30, vector) {
        unsafe {
            outb(PIC_M_CTRL, PIC_EOI);
            outb(PIC_S_CTRL, PIC_EOI);
        }
    }
}

#[doc(hidden)]
fn _contains(range: core::ops::Range<u32>, vector: u32) -> bool {
    range.start <= vector && vector < range.end
}
