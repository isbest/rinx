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
use crate::kernel::interrupts::{init_interrupt, sti};
use crate::kernel::logger::init_logger;
use crate::kernel::time::now_time;
use core::arch::global_asm;
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
    init_interrupt();
    // 开启外中断
    sti();

    loop {
        unsafe {
            use core::arch::asm;
            delay(10000000);
            info!("{}", now_time());
            asm!("int 0x80");
        }
    }
}

#[no_mangle]
fn delay(mut count: i64) {
    while count > 0 {
        count -= 1;
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
