use crate::kernel::interrupts::pic::controller::send_eoi;
use log::debug;

#[allow(clippy::too_many_arguments)]
#[no_mangle]
pub extern "C" fn default_handler(
    vector: u32,
    _edi: u32,
    _esi: u32,
    _ebp: u32,
    _esp: u32,
    _ebx: u32,
    _edx: u32,
    _ecx: u32,
    _eax: u32,
    _gs: u32,
    _fs: u32,
    _es: u32,
    _ds: u32,
    _vector0: u32,
    error_code: u32,
    _eip: u32,
    _cs: u32,
    _eflags: u32,
) {
    // 中断结束
    debug!("[EXTERNAL INTERRUPT] vector: {vector}, error code: {error_code}");
    send_eoi(vector);
}
