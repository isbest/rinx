#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(naked_functions)]
#![feature(asm_const)]

extern crate alloc;

mod drivers;
mod kernel;
mod mm;

use crate::kernel::gdt::init_gdt;
use crate::kernel::interrupts::idt::init_idt;
use crate::kernel::logger::init_logger;
use core::arch::{asm, global_asm};
use core::panic::PanicInfo;
use log::info;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    // 初始化日志
    init_logger();
    // 初始化内核全局描述符
    init_gdt();
    // 初始化中断
    init_idt();

    info!("hello, this is rust kernel");

    bmb!();
    unsafe {
        asm!("mov eax, 1", "xor edx, edx", "div edx", options(noreturn));
    }

    #[allow(clippy::empty_loop)]
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
