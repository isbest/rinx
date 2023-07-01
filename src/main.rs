#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(ptr_internals)]
#![feature(allocator_api)]
#![feature(offset_of)]

extern crate alloc;

mod drivers;
mod kernel;
mod libs;
mod mm;

use crate::kernel::interrupts::{enable_interrupt, init_interrupt};
use crate::kernel::system_call::init_system_call;
use crate::kernel::tasks::init_task;
use core::arch::global_asm;
use core::panic::PanicInfo;
use x86::halt;

pub const KERNEL_MAGIC: u32 = 0x20230604;

global_asm!(include_str!("entry.asm"));

/// 内核入口
/// gdt放在内存映射之前初始化,避免内存被页目录占用
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    // 初始化中断
    init_interrupt();
    // 初始化任务
    init_task();
    // 初始化系统调用
    init_system_call();
    // 开启外中断
    enable_interrupt(true);
    info!("hello world, this is rust kernel");

    #[allow(clippy::empty_loop)]
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
        unsafe {
            halt();
        }
    }
}
