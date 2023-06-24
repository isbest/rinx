use core::mem::size_of;
use core::slice;

use crate::mm::allocator::init_heap;
use crate::mm::detected::HEAP_MEMORY_BASE;
use x86::bits32::paging::{
    pd_index, PAddr, PDEntry, PDFlags, PTEntry, PTFlags, VAddr,
    PAGE_SIZE_ENTRIES,
};
use x86::controlregs::{cr0, cr0_write, cr3_write, Cr0};
use x86::tlb::flush;

/// 0x1000到0x7c00都是可用区域
/// 内核页目录的位置设置为0x1000 4KB位置
/// 第一页页表存储到 0x2000 8KB的位置
/// 第三页页表存储到 0x3000 12KB的位置
/// 0x1000 是前期loader的位置,加载完之后,内存就可以另作他用了,嘿嘿
pub const KERNEL_PAGE_DIR: u32 = 0x1000;

/// 内核页表索引
const KERNEL_PAGE_TABLE: KernelPageTableType = [0x2000, 0x3000];

/// 内核页目录索引的类型
type KernelPageTableType = [u32; 2];

/// 内核的内存空间,内核的页表数*1M,一个页表可以映射1M内存
pub const KERNEL_MEMORY_SIZE: usize =
    size_of::<KernelPageTableType>() * 0x100000;

#[no_mangle]
pub fn init_mem_mapping() {
    // 页目录
    let page_dir_table: &mut [PDEntry] = unsafe {
        slice::from_raw_parts_mut(
            KERNEL_PAGE_DIR as *mut PDEntry,
            PAGE_SIZE_ENTRIES,
        )
    };

    // 页目录全部初始化为0,避免被别的地方初始化过
    // 巨坑,内核的GDT位于0x11a3,这个0初始化,会导致抹掉GDT
    // 所以初始化顺序先初始化了内核GDT
    page_dir_table.fill(PDEntry::new(PAddr::from(0), PDFlags::empty()));

    let mut index: usize = 0;
    // 开始映射内核的页表
    KERNEL_PAGE_TABLE.iter().enumerate().for_each(
        |(kernel_pd_index, page_addr)| {
            // 通过页地址获取页表
            let page_entry_table: &mut [PTEntry] = unsafe {
                slice::from_raw_parts_mut(
                    *page_addr as *mut PTEntry,
                    PAGE_SIZE_ENTRIES,
                )
            };

            page_dir_table[kernel_pd_index] = PDEntry::new(
                PAddr::from((*page_addr).idx_mask()),
                PDFlags::P | PDFlags::RW | PDFlags::US,
            );

            // 跳过页表0的位置,第0页不映射
            page_entry_table.iter_mut().for_each(|pt_entry| {
                if index == 0 {
                    index += 1;
                    return;
                }

                *pt_entry = PTEntry::new(
                    // 用PAddr包裹页索引对应的物理内存的起始位置
                    PAddr::from(index.page()),
                    PTFlags::P | PTFlags::RW | PTFlags::US,
                );
                index += 1;
            });
        },
    );

    // // 将页表的最后一个初始化成自己,方便在启用分页后修改页表
    if let Some(last_entry) = page_dir_table.last_mut() {
        *last_entry = PDEntry::new(
            PAddr::from(KERNEL_PAGE_DIR.idx_mask()),
            PDFlags::P | PDFlags::RW | PDFlags::US,
        );
    }

    // 设置页目录
    set_cr3(page_dir_table);
    // 开启分页
    enable_page();

    unsafe {
        // 初始化内存分配器,虚拟地址1-8M是内核的
        init_heap(HEAP_MEMORY_BASE, 0x800000);
    }
}

/// 开启虚拟内存后,获取页目录
pub fn get_page_dir_table() -> &'static mut [PDEntry] {
    unsafe {
        slice::from_raw_parts_mut(0xFFFFF000 as *mut PDEntry, PAGE_SIZE_ENTRIES)
    }
}

pub fn get_page_entry_table(addr: u32) -> &'static mut [PTEntry] {
    unsafe {
        slice::from_raw_parts_mut(
            (0xffc00000 | pd_index(VAddr(addr))) as *mut PTEntry,
            1,
        )
    }
}

pub fn flash_tlb(addr: usize) {
    unsafe {
        flush(addr);
    }
}

pub fn set_cr3(page_dir_table: &mut [PDEntry]) {
    unsafe {
        cr3_write(page_dir_table.as_ptr() as u64);
    }
}

// 开启分页
#[inline(always)]
pub fn enable_page() {
    unsafe {
        cr0_write(cr0() | Cr0::CR0_ENABLE_PAGING);
    }
}

// 获取页索引
pub trait PageIndex {
    // 传入物理地址,返回页索引
    fn idx(&self) -> Self;

    // 传入页索引,返回页索引对应的内存的起始位置
    fn page(&self) -> Self;

    // 将低12位置零,高20位表示页索引
    fn idx_mask(&self) -> Self;
}

macro_rules! impl_page_index {
    ($h:ty) => {
        impl PageIndex for $h {
            fn idx(&self) -> Self {
                // 低12位置零
                self >> 12
            }

            fn page(&self) -> Self {
                // 低12位置零
                self << 12
            }

            fn idx_mask(&self) -> Self {
                // 低12位置零
                self & !0xfff
            }
        }
    };
}

impl_page_index!(u32);
impl_page_index!(u64);
impl_page_index!(usize);
