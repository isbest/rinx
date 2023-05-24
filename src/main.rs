#![no_std]
#![no_main]

mod drivers;

use core::arch::global_asm;
use core::panic::PanicInfo;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    clear_bss();

    println!("hello world!");
    println!("this is rust kernel!!!");
    println!("yes!!");
    print!("hello");
    print!("\x7f");
    println!("world");
    print!("world");
    print!("\x0D");
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
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
