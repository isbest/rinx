pub mod global;
pub mod interrupts;
pub mod sync;
pub mod system_call;
pub mod tasks;
pub mod time;

#[macro_export]
macro_rules! bmb {
    () => {
        unsafe {
            use core::arch::asm;
            asm!("xchg bx, bx");
        };
    };
}
