#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

global_asm!(include_str!("entry.asm"));

static HELLO: &[u8] = b"Hello World!";

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    let vga_buffer = 0xb8000 as *mut u8;

    clear_bss();

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }
    loop {}
}

// 清空bss段
fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|bytes| {
        unsafe {
            (bytes as *mut u8).write_volatile(0);
        }
    })
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
