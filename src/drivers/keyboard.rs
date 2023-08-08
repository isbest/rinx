use crate::info;
use crate::kernel::interrupts::handler::set_interrupt_handler;
use crate::kernel::interrupts::pic::pic_controller::send_eoi;
use crate::kernel::interrupts::{
    set_interrupt_mask, IRQ_KEYBOARD, IRQ_MASTER_NR,
};
use x86::io::inb;

const KEYBOARD_DATA_PORT: u16 = 0x60;
const KEYBOARD_CTRL_PORT: u16 = 0x60;

/// 键盘中断处理函数
pub extern "C" fn keyboard_handler(
    vector: u32,
    _edi: u32,
    _esi: u32,
    _ebp: u32,
    _esp: u32,
    _ebx: u32,
    _edx: u32,
    _ecx: u32,
    _eax: u32,
    _gs: u32,
    _fs: u32,
    _es: u32,
    _ds: u32,
    _vector0: u32,
    _error_code: u32,
    _eip: u32,
    _cs: u32,
    _eflags: u32,
) {
    assert_eq!(vector, 0x21, "must be 0x21 keyboard interrupt");
    send_eoi(vector);

    let scancode = unsafe {
        // 从键盘数据寄存器读取键盘信息扫描码
        inb(KEYBOARD_DATA_PORT)
    };

    info!("keyboard input {scancode}");
}
pub fn init_keyboard() {
    set_interrupt_handler(
        IRQ_MASTER_NR + IRQ_KEYBOARD as usize,
        keyboard_handler,
    );
    set_interrupt_mask(IRQ_KEYBOARD, true);
}
