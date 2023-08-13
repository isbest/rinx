use crate::kernel::interrupts::handler::set_interrupt_handler;
use crate::kernel::interrupts::pic::pic_controller::send_eoi;
use crate::kernel::interrupts::{
    set_interrupt_mask, IRQ_KEYBOARD, IRQ_MASTER_NR,
};
use crate::kernel::sync::mutex::Mutex;
use crate::print;
use lazy_static::lazy_static;
use pc_keyboard::layouts::Us104Key;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
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

    // 解析键盘的scancode
    parser_scancode(scancode);
}

fn parser_scancode(scancode: u8) {
    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(
                ScancodeSet1::new(),
                Us104Key,
                HandleControl::Ignore
            ));
    }

    let mut keyboard = KEYBOARD.lock();
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("RawKey {:?}", key),
            }
        }
    }
}

pub fn init_keyboard() {
    set_interrupt_handler(
        IRQ_MASTER_NR + IRQ_KEYBOARD as usize,
        keyboard_handler,
    );
    set_interrupt_mask(IRQ_KEYBOARD, true);
}
