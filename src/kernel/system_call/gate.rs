use crate::kernel::system_call::SYS_CALL_SIZE;
use crate::printlnk;

type SystemCall = extern "C" fn(usize, usize, usize, usize) -> usize;

#[no_mangle]
pub static mut SYSTEM_CALL_TABLE: [SystemCall; SYS_CALL_SIZE] = {
    #[allow(unused_mut)]
    let mut interrupt_handler_table: [SystemCall; SYS_CALL_SIZE] =
        [default_sys_call; SYS_CALL_SIZE];

    interrupt_handler_table
};

pub extern "C" fn default_sys_call(
    ebx: usize,
    ecx: usize,
    edx: usize,
    vector: usize,
) -> usize {
    printlnk!("vector:{}, eax:{}, ebx:{} ecx:{}", vector, ebx, ecx, edx);
    0
}
