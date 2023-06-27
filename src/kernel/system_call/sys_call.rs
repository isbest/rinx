use crate::kernel::system_call::sys_call;
use crate::kernel::tasks::task::Task;

/// 系统调用枚举
pub enum SysCall {
    Test = 1,
    Yield = 2,
}

pub fn sys_yield() {
    sys_call(SysCall::Yield);
}

pub extern "C" fn task_yield(_: u32, _: u32, _: u32, _: u32) -> u32 {
    unsafe {
        Task::schedule();
    }
    0
}
