use crate::drivers::keyboard::read_keyboard;
use crate::kernel::interrupts::{enable_interrupt, without_interrupt};
use crate::print;

pub(crate) fn init() -> u32 {
    enable_interrupt(true);

    let mut buffer = [' '; 1];
    loop {
        without_interrupt(|| {
            read_keyboard(&mut buffer);
            buffer.iter().for_each(|ch| print!("{ch}"));
        });
    }
}
