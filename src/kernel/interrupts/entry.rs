use crate::kernel::interrupts::InterruptEntry;
use core::fmt;
use core::marker::PhantomData;

/// 入口 gate
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Entry<F> {
    /// 低16位
    offset_low: u16,
    /// 代码段选择子
    selector: u16,
    /// 保留位
    reserved: u8,
    /// 选项
    pub options: EntryOptions,
    /// 高16位
    offset_high: u16,
    phantom: PhantomData<F>,
}

impl<T> fmt::Debug for Entry<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Entry")
            .field("offset_low", &self.offset_low)
            .field("selector", &self.selector)
            .field("options", &self.options)
            .field("offset_high", &self.offset_high)
            .field("reserved", &self.reserved)
            .finish()
    }
}

impl<T> PartialEq for Entry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.offset_low == other.offset_low
            && self.selector == other.selector
            && self.options == other.options
            && self.offset_high == other.offset_high
            && self.reserved == other.reserved
    }
}

impl<F> Entry<F> {
    #[inline]
    pub const fn missing() -> Self {
        Entry {
            offset_low: 0,
            selector: 0,
            offset_high: 0,
            options: EntryOptions::minimal(),
            reserved: 0,
            phantom: PhantomData,
        }
    }

    #[inline]
    fn set_handler_addr(&mut self, addr: u32) -> &mut EntryOptions {
        self.offset_low = (addr & 0xffff) as u16;
        self.offset_high = ((addr >> 16) & 0xffff) as u16;

        // 代码段
        self.selector = 1 << 3;

        // 设置存在
        // 使用默认的中断门和内核态设置
        self.options.set_present(true);

        &mut self.options
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EntryOptions(u8);

/// 门的类型
impl EntryOptions {
    /// 小端序存储,低字节存储在低地址,高字节存储在高地址
    /// 因此下面说的位要从左往右数
    /// 中断门, 系统段, 内核特权级默认无效
    /// 一共8位
    /// 4-7表示门类型,1110 是中断门
    /// 第3位表示segment
    /// 第1-2位表示特权级
    /// 第0位表示是否在内存中
    #[inline]
    const fn minimal() -> Self {
        EntryOptions(0b00001110)
    }

    #[inline]
    pub fn set_present(&mut self, present: bool) -> &mut Self {
        if present {
            // 第0位设置成1即可
            self.0 |= 0b1000_0000;
        } else {
            // 不存在将第0位设置成0
            self.0 |= 0b0111_1111;
        }

        self
    }

    #[inline]
    pub fn set_privilege_level(&mut self, dpl: u8) -> &mut Self {
        // 先与0b1001_1111,清空特权级
        // 然后dpl与上0b11取低2两位的特权级
        // 然后左移5位,移动到正确的位置
        // 与运算,设置特权级别
        self.0 = (self.0 & 0b1001_1111) | ((dpl & 0b11) << 5);
        self
    }

    #[inline]
    pub fn set_code_segment(&mut self, code_segment: u16) {
        // 原理同上
        self.0 = (self.0 & 0xF8) | ((code_segment >> 3) as u8);
    }
}

/// 为指定类型实现set_handler_fn的宏
macro_rules! impl_set_handler_fn {
    ($h:ty) => {
        impl Entry<$h> {
            #[inline]
            pub fn set_handler_fn(&mut self, handler: $h) -> &mut EntryOptions {
                self.set_handler_addr(handler as usize as u32)
            }
        }
    };
}

// 为InterruptEntry类型实现
impl_set_handler_fn!(InterruptEntry);
