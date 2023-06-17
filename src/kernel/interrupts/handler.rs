use crate::kernel::interrupts::handler_entry::InterruptHandler;
use crate::kernel::interrupts::{ENTRY_SIZE, IDT_SIZE};
use log::error;

// 中断函数表
#[no_mangle]
pub static mut INTERRUPT_HANDLER_TABLE: [InterruptHandler; ENTRY_SIZE] = {
    #[allow(unused_mut)]
    let mut interrupt_handler_table: [InterruptHandler; ENTRY_SIZE] =
        [default_exception_handler; ENTRY_SIZE];

    interrupt_handler_table
};

pub fn set_interrupt_handler(index: usize, f: InterruptHandler) {
    assert!((0..IDT_SIZE).contains(&index));

    unsafe {
        INTERRUPT_HANDLER_TABLE[index] = f;
    }
}

/// 异常的默认处理函数
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn default_exception_handler(
    vector: u32,
    _edi: u32,
    _esi: u32,
    _ebp: u32,
    esp: u32,
    _ebx: u32,
    _edx: u32,
    _ecx: u32,
    _eax: u32,
    gs: u32,
    fs: u32,
    es: u32,
    ds: u32,
    _vector0: u32,
    error_code: u32,
    eip: u32,
    cs: u32,
    eflags: u32,
) {
    use x86::irq::EXCEPTIONS;

    if vector < 22 {
        error!(
            "[EXCEPTION] {}, {}",
            EXCEPTIONS[vector as usize], error_code
        );
    } else {
        error!("[EXCEPTION] {}, {}", EXCEPTIONS[15], error_code);
    }

    error!(
        r#"VECTOR:{}
 ERROR:{}
EFLAGS:{}
    CS:{}
    EIP:{}
    ESP:{}
    DS:{}
    FS:{}
    ES:{}
    GS:{}"#,
        vector, error_code, eflags, cs, eip, esp, ds, fs, es, gs
    );
    #[allow(clippy::empty_loop)]
    loop {}
}
