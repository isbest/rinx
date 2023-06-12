use core::arch::asm;

#[repr(C)]
pub struct HandlerTable {
    pub handlers: [unsafe extern "C" fn(); 0x20],
}

macro_rules! interrupt_handler {
    // 没有错误码,压入固定的错误码
    ($vector:expr, $name:ident, false) => {
        #[naked]
        #[no_mangle]
        unsafe extern "C" fn $name() {
            asm!(
                "push 0x20230612",
                "push $0",
                "jmp interrupt_entry",
                options(noreturn)
            );
        }
    };
    ($vector:expr, $name:ident, true) => {
        #[naked]
        #[no_mangle]
        unsafe extern "C" fn $name() {
            asm!("push $0", "jmp interrupt_entry", options(noreturn));
        }
    };
}

#[naked]
#[no_mangle]
unsafe extern "C" fn interrupt_entry() {
    asm!(
        // 传递获取中断向量
        "mov eax, [esp]",
        // 调用中断处理函数
        "call [eax * 4 + HANDLER_TABLE]",
        // 恢复栈
        "add esp, 8",
        "iret",
        options(noreturn)
    );
}

// 中断函数生成
interrupt_handler!(0x00, interrupt_handler_0x00, false); // divide by zero
interrupt_handler!(0x01, interrupt_handler_0x01, false); // debug
interrupt_handler!(0x02, interrupt_handler_0x02, false); //non maskable interrupt
interrupt_handler!(0x03, interrupt_handler_0x03, false); // breakpoint

interrupt_handler!(0x04, interrupt_handler_0x04, false); // overflow
interrupt_handler!(0x05, interrupt_handler_0x05, false); // bound range exceeded
interrupt_handler!(0x06, interrupt_handler_0x06, false); // invalid opcode
interrupt_handler!(0x07, interrupt_handler_0x07, false); // device not avilable

interrupt_handler!(0x08, interrupt_handler_0x08, true); // double fault
interrupt_handler!(0x09, interrupt_handler_0x09, false); // coprocessor segment overrun
interrupt_handler!(0x0a, interrupt_handler_0x0a, true); // invalid TSS
interrupt_handler!(0x0b, interrupt_handler_0x0b, true); // segment not present

interrupt_handler!(0x0c, interrupt_handler_0x0c, true); // stack segment fault
interrupt_handler!(0x0d, interrupt_handler_0x0d, true); // general protection fault
interrupt_handler!(0x0e, interrupt_handler_0x0e, true); // page fault
interrupt_handler!(0x0f, interrupt_handler_0x0f, false); // reserved

interrupt_handler!(0x10, interrupt_handler_0x10, false); // x87 floating point exception
interrupt_handler!(0x11, interrupt_handler_0x11, true); // alignment check
interrupt_handler!(0x12, interrupt_handler_0x12, false); // machine check
interrupt_handler!(0x13, interrupt_handler_0x13, false); // SIMD Floating - Point Exception

interrupt_handler!(0x14, interrupt_handler_0x14, false); // Virtualization Exception
interrupt_handler!(0x15, interrupt_handler_0x15, true); // Control Protection Exception
interrupt_handler!(0x16, interrupt_handler_0x16, false); // reserved
interrupt_handler!(0x17, interrupt_handler_0x17, false); // reserved

interrupt_handler!(0x18, interrupt_handler_0x18, false); // reserved
interrupt_handler!(0x19, interrupt_handler_0x19, false); // reserved
interrupt_handler!(0x1a, interrupt_handler_0x1a, false); // reserved
interrupt_handler!(0x1b, interrupt_handler_0x1b, false); // reserved

interrupt_handler!(0x1c, interrupt_handler_0x1c, false); // reserved
interrupt_handler!(0x1d, interrupt_handler_0x1d, false); // reserved
interrupt_handler!(0x1e, interrupt_handler_0x1e, false); // reserved
interrupt_handler!(0x1f, interrupt_handler_0x1f, false); // reserved

#[no_mangle]
pub static mut HANDLER_TABLE: HandlerTable = HandlerTable {
    handlers: [
        interrupt_handler_0x00,
        interrupt_handler_0x01,
        interrupt_handler_0x02,
        interrupt_handler_0x03,
        interrupt_handler_0x04,
        interrupt_handler_0x05,
        interrupt_handler_0x06,
        interrupt_handler_0x07,
        interrupt_handler_0x08,
        interrupt_handler_0x09,
        interrupt_handler_0x0a,
        interrupt_handler_0x0b,
        interrupt_handler_0x0c,
        interrupt_handler_0x0d,
        interrupt_handler_0x0e,
        interrupt_handler_0x0f,
        interrupt_handler_0x10,
        interrupt_handler_0x11,
        interrupt_handler_0x12,
        interrupt_handler_0x13,
        interrupt_handler_0x14,
        interrupt_handler_0x15,
        interrupt_handler_0x16,
        interrupt_handler_0x17,
        interrupt_handler_0x18,
        interrupt_handler_0x19,
        interrupt_handler_0x1a,
        interrupt_handler_0x1b,
        interrupt_handler_0x1c,
        interrupt_handler_0x1d,
        interrupt_handler_0x1e,
        interrupt_handler_0x1f,
    ],
};
