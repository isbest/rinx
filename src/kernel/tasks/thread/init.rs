use crate::info;
use crate::kernel::interrupts::enable_interrupt;

pub(crate) fn init() -> u32 {
    enable_interrupt(true);

    loop {
        info!("init task");
    }
}
