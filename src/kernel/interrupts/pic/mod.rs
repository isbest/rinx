pub mod controller;
pub mod handler;

/// 主片控制端口
pub const PIC_M_CTRL: u16 = 0x20;
/// 主片数据端口
pub const PIC_M_DATA: u16 = 0x21;
/// 从片控制端口
pub const PIC_S_CTRL: u16 = 0xa0;
/// 从片控制端口
pub const PIC_S_DATA: u16 = 0xa1;
/// 通知中断控制器处理结束
pub const PIC_EOI: u8 = 0x20;
