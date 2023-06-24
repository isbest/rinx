#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(naked_functions)]
#![feature(asm_const)]

extern crate alloc;

mod drivers;
mod kernel;
mod mm;

use crate::kernel::interrupts::{enable_interrupt, init_interrupt};
use crate::kernel::logger::init_logger;
use crate::kernel::tasks::init_task;
use crate::kernel::time::now_time;
use core::arch::global_asm;
use core::panic::PanicInfo;
use log::info;

pub const KERNEL_MAGIC: u32 = 0x20230604;

global_asm!(include_str!("entry.asm"));

/// 内核入口
/// gdt放在内存映射之前初始化,避免内存被页目录占用
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    // 初始化日志
    init_logger();
    // 初始化中断
    init_interrupt();
    // 初始化任务
    init_task();
    // 开启外中断
    enable_interrupt(true);
    info!("hello world, this is rust kernel");

    loop {
        delay(10000000);
        info!("{}", now_time());
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
