use crate::kernel::interrupts::handler_entry::InterruptHandler;
use crate::kernel::interrupts::ENTRY_SIZE;
use log::error;

// 中断函数表
#[no_mangle]
pub static mut INTERRUPT_HANDLER_TABLE: [InterruptHandler; ENTRY_SIZE] = {
    #[allow(unused_mut)]
    let mut interrupt_handler_table: [InterruptHandler; ENTRY_SIZE] =
        [exception_handler; ENTRY_SIZE];

    interrupt_handler_table
};

/// 异常的默认处理函数
pub fn exception_handler(vector_index: u32, error_code: u32) {
    use x86::irq::EXCEPTIONS;
    if vector_index < 22 {
        error!(
            "[EXCEPTION] {}, {}",
            EXCEPTIONS[vector_index as usize], error_code
        );
    } else {
        error!("[EXCEPTION] {}, {}", EXCEPTIONS[15], error_code);
    }

    #[allow(clippy::empty_loop)]
    loop {}
}
