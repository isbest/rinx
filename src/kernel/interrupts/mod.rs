use crate::kernel::interrupts::entry::Entry;
use core::mem::size_of;
use log::error;
use x86::dtables::{lidt, DescriptorTablePointer};
use crate::kernel::interrupts::handler::HANDLER_TABLE;

pub mod entry;
pub mod handler;

const IDT_SIZE: usize = 0x20;
static mut INTERRUPT_ENTRY: [Entry<unsafe extern "C" fn()>; IDT_SIZE] = [Entry::missing(); IDT_SIZE];

pub unsafe extern "C" fn exception_handler() {
    error!("hello");
    #[allow(clippy::empty_loop)]
    loop {}
}

pub fn init_idt() {
    let mut idt: DescriptorTablePointer<Entry<unsafe extern "C" fn()>> = DescriptorTablePointer::default();

    unsafe {
        for i in 0..IDT_SIZE {
            #[allow(const_item_mutation)]
            INTERRUPT_ENTRY[i].set_handler_fn(HANDLER_TABLE.handlers[i]);
        }

        idt.base = INTERRUPT_ENTRY.as_ptr();
        idt.limit = (size_of::<[Entry<u32>; IDT_SIZE]>() - 1) as u16;
    }

    unsafe {
        lidt(&idt);
    }
}
