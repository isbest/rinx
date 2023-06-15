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
use crate::kernel::interrupts::init_interrupt;
use crate::kernel::logger::init_logger;
use core::arch::{asm, global_asm};
use core::panic::PanicInfo;
use x86::irq::enable;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    // 初始化日志
    init_logger();
    // 初始化内核全局描述符
    init_gdt();
    // 初始化中断
    init_interrupt();
    // 开启外中断
    unsafe {
        enable();
    }

    loop {}
}

#[no_mangle]
fn delay(mut count: u64) {
    while count > 0 {
        count -= 1;
    }
    bmb!();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
