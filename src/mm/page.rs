use core::slice;

use x86::bits32::paging::{
    PAddr, PDEntry, PDFlags, PTEntry, PTFlags, PAGE_SIZE_ENTRIES,
};
use x86::controlregs::{cr0, cr0_write, cr3_write, Cr0};

use crate::bmb;

/// 页表位置2M的位置
const KERNEL_PAGE_DIR: u32 = 0x200000;
const KERNEL_PAGE_ENTRY: u32 = 0x201000;

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
    page_dir_table.fill(PDEntry::new(PAddr::from(0), PDFlags::empty()));

    // 设置第一个页目录,指向第一个页表
    page_dir_table[0] = PDEntry::new(
        PAddr::from(KERNEL_PAGE_ENTRY.idx_mask()),
        PDFlags::P | PDFlags::RW | PDFlags::US,
    );

    // KERNEL_PAGE_ENTRY 第一个页表的物理内存地址
    let page_entry_table: &mut [PTEntry] = unsafe {
        slice::from_raw_parts_mut(
            KERNEL_PAGE_ENTRY as *mut PTEntry,
            PAGE_SIZE_ENTRIES,
        )
    };

    // 将第一个页表初始化,全部都映射到物理内存的1M以内
    page_entry_table
        .iter_mut()
        .enumerate()
        .for_each(|(index, pt_entry)| {
            *pt_entry = PTEntry::new(
                // 用PAddr包裹页索引对应的物理内存的起始位置
                PAddr::from(index.page()),
                PTFlags::P | PTFlags::RW | PTFlags::US,
            );
        });
    // 0000_0000 0000_0000 0000_0000 1001_0000

    bmb!();
    // 设置页目录
    set_cr3(page_dir_table);
    bmb!();
    // 开启分页
    enable_page();
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
