use crate::drivers::gpu::vga_driver::CONSOLE;
use crate::kernel::system_call::sys_call::SysCall;
use crate::kernel::system_call::sys_call_3;
use core::ptr::slice_from_raw_parts;

#[repr(C)]
pub enum StdFd {
    In,
    Out,
    Err,
}

pub fn sys_write(fd: StdFd, str: &str) {
    sys_call_3(SysCall::Write, fd as _, str.as_ptr() as _, str.len());
}

pub(crate) extern "C" fn write_char(
    _fd: usize,
    ptr: usize,
    len: usize,
    _vector: usize,
) -> usize {
    let slice = unsafe { &*slice_from_raw_parts(ptr as *const u8, len) };
    CONSOLE.lock().write_bytes(slice);
    len
}
