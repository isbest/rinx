use crate::kernel::interrupts::pic::controller::send_eoi;
use log::debug;

pub fn default_handler(vector: u32, error_code: u32) {
    // 中断结束
    debug!("[EXTERNAL INTERRUPT] vector: {vector}, error code: {error_code}");
    send_eoi(vector);
}
