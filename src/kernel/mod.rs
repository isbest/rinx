use core::mem::size_of;

pub mod gdt;
pub mod interrupts;
pub mod logger;

/// 从limit获取长度
#[inline(always)]
pub fn len_of_limit<T>(limit: u16) -> usize {
    (limit + 1) as usize / size_of::<T>()
}

/// 获取类型的limit
#[inline(always)]
pub fn limit_of_type<T>() -> u16 {
    (size_of::<T>() - 1) as u16
}

#[macro_export]
macro_rules! bmb {
    () => {
        unsafe {
            use core::arch::asm;
            asm!("xchg bx, bx",)
        };
    };
}
