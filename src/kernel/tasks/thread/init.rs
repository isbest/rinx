use crate::kernel::interrupts::enable_interrupt;
use crate::kernel::system_call::sys_call::sys_sleep;
use crate::warn;

pub(crate) fn init() -> u32 {
    enable_interrupt(true);

    loop {
        warn!("init task");
        sys_sleep(1000);
    }
}
