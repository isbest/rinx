#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(naked_functions)]
#![feature(asm_const)]

extern crate alloc;

mod drivers;
mod kernel;
mod mm;

use crate::kernel::interrupts::init_interrupt;
use crate::kernel::logger::init_logger;
use crate::kernel::task::init_task;
use crate::kernel::time::now_time;
use core::arch::global_asm;
use core::panic::PanicInfo;
use log::info;
use x86::halt;

global_asm!(include_str!("entry.asm"));

/// 内核入口
/// gdt放在内存映射之前初始化,避免内存被页目录占用
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    // use crate::kernel::interrupts::sti;
    // 初始化日志
    init_logger();
    // 初始化中断
    init_interrupt();
    // 开启外中断
    // sti();

    info!("Hello World!");
    println!("hello world");
    init_task();

    loop {
        delay(10000000);
        info!("{}", now_time());
        unsafe {
            use core::arch::asm;
            asm!("int 0x80");
            // cpu 关机
            halt();
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
