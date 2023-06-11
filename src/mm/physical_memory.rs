use core::slice;

use crate::mm::heap_allocator::init_heap;

const KERNEL_MAGIC: u32 = 0x20230604;
const MEMORY_BASE: u64 = 0x100000;
const ALIGN_MASK: u64 = 0xfff;

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

pub fn page_idx(addr: u64) -> u64 {
    addr >> 12
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

    let mut memory_base: u64 = 0;
    let mut memory_size: u64 = 0;

    // 在这里使用 addrs_array 数组
    for addr in addrs_slice {
        if addr.is_usable() && addr.size > memory_size {
            memory_base = addr.base;
            memory_size = addr.size;
        }
    }

    // 起始地址必须是1M
    assert_eq!(memory_base, MEMORY_BASE);
    // 必须是4K对齐
    assert_eq!(memory_size & ALIGN_MASK, 0);

    // 初始化内存分配器
    init_heap(memory_base, memory_size as usize);
}
