use crate::kernel::system_call::SYS_CALL_SIZE;
use crate::println;

type SystemCall = extern "C" fn(u32, u32, u32, u32) -> u32;

#[no_mangle]
pub static mut SYSTEM_CALL_TABLE: [SystemCall; SYS_CALL_SIZE] = {
    #[allow(unused_mut)]
    let mut interrupt_handler_table: [SystemCall; SYS_CALL_SIZE] =
        [default_sys_call; SYS_CALL_SIZE];

    interrupt_handler_table
};

pub extern "C" fn default_sys_call(
    ebx: u32,
    ecx: u32,
    edx: u32,
    vector: u32,
) -> u32 {
    println!("vector:{}, eax:{}, ebx:{} ecx:{}", vector, ebx, ecx, edx);
    0
}
