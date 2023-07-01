use crate::kernel::system_call::{sys_call, sys_call_1};
use crate::kernel::tasks::task::Task;

/// 系统调用枚举
pub enum SysCall {
    Test = 1,
    Yield = 2,
    Sleep = 3,
}

pub fn sys_yield() {
    sys_call(SysCall::Yield);
}

pub fn sys_sleep(ms: u32) {
    sys_call_1(SysCall::Sleep, ms);
}

/// 下面是系统调用的具体实现
pub extern "C" fn task_yield(_: u32, _: u32, _: u32, _: u32) -> u32 {
    unsafe {
        Task::schedule();
    }
    0
}

pub extern "C" fn task_sleep(ms: u32, _: u32, _: u32, _: u32) -> u32 {
    unsafe { Task::sleep(ms) }
    0
}
