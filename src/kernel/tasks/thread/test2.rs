use crate::info;
use crate::kernel::interrupts::enable_interrupt;
use crate::kernel::system_call::sys_call::sys_sleep;

pub(crate) fn test2() -> u32 {
    enable_interrupt(true);

    loop {
        info!("test2 task");
        sys_sleep(1000);
    }
}
