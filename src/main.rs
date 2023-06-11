#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(core_intrinsics)]

extern crate alloc;

mod drivers;
mod kernel;
mod mm;

use core::arch::global_asm;
use core::panic::PanicInfo;
use log::{debug, error, info, trace, warn};
use crate::kernel::gdt::init_gdt;
use crate::kernel::logger::init_logger;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    init_logger();
    init_gdt();
    info!("hello, this is rust kernel");
    debug!("hello, this is rust kernel");
    warn!("hello, this is rust kernel");
    error!("hello, this is rust kernel");
    trace!("hello, this is rust kernel");

    #[allow(clippy::empty_loop)]
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
