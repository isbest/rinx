use crate::kernel::system_call::{sys_call, sys_call_1};
use crate::kernel::tasks::task::Task;

/// 系统调用枚举
#[repr(C)]
pub enum SysCall {
    Test,
    Write,
    Yield,
    Sleep,
}

pub fn sys_yield() {
    sys_call(SysCall::Yield);
}

pub fn sys_sleep(ms: usize) {
    sys_call_1(SysCall::Sleep, ms);
}

/// 下面是系统调用的具体实现
pub(crate) extern "C" fn task_yield(
    _: usize,
    _: usize,
    _: usize,
    _: usize,
) -> usize {
    unsafe {
        Task::schedule();
    }
    0
}

pub(crate) extern "C" fn task_sleep(
    ms: usize,
    _: usize,
    _: usize,
    _: usize,
) -> usize {
    unsafe { Task::sleep(ms) }
    0
}
