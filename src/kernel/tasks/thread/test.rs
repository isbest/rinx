use crate::error;
use crate::kernel::interrupts::enable_interrupt;
use crate::kernel::system_call::sys_call::sys_sleep;

pub(crate) fn test() -> u32 {
    enable_interrupt(true);

    loop {
        error!("test1 task");
        sys_sleep(500);
    }
}
