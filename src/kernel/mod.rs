use core::mem::size_of;

pub mod gdt;
pub mod interrupts;
pub mod logger;

// get len of limit
#[inline(always)]
pub fn len_of_limit<T>(limit: u16) -> usize {
    (limit + 1) as usize / size_of::<T>()
}
