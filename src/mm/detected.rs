use core::slice;

use crate::mm::allocator::init_heap;

const KERNEL_MAGIC: u32 = 0x20230604;
const MEMORY_BASE: u64 = 0x100000;
const ALIGN_MASK: u64 = 0xfff;

pub static mut HEAP_MEMORY_BASE: u64 = 0;
pub static mut HEAP_MEMORY_SIZE: u64 = 0;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Ards {
    pub base: u64,
    pub size: u64,
    pub state: u32,
}

impl Ards {
    pub fn is_usable(&self) -> bool {
        self.state == 1
    }
}

#[no_mangle]
pub unsafe fn memory_init(kernel_magic: u32, addrs_count: *const u32) {
    let count = unsafe { *addrs_count };
    if kernel_magic != KERNEL_MAGIC {
        panic!("invalid kernel magic number..")
    }

    // 计算紧随 addrs_count 地址后面的 Address 数组的地址
    let addrs_slice = unsafe {
        // 计算addrs_count的偏移地址
        let addrs_array = addrs_count.offset(1) as *const Ards;

        // 转换成slice(数组切片)
        slice::from_raw_parts(addrs_array, count as usize)
    };

    // 在这里使用 addrs_array 数组
    for addr in addrs_slice {
        if addr.is_usable() && addr.size > HEAP_MEMORY_SIZE {
            unsafe {
                HEAP_MEMORY_BASE = addr.base;
                HEAP_MEMORY_SIZE = addr.size;
            }
        }
    }

    // 起始地址必须是1M
    assert_eq!(HEAP_MEMORY_BASE, MEMORY_BASE);
    // 必须是4K对齐
    assert_eq!(HEAP_MEMORY_SIZE & ALIGN_MASK, 0);

    // 初始化内存分配器
    init_heap(HEAP_MEMORY_BASE, HEAP_MEMORY_SIZE as usize);
}
