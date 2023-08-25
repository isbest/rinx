pub mod color;
pub mod vga_driver;

/// vga text mode buffer width
const BUFFER_WIDTH: usize = 80;
/// vga text mode buffer height
const BUFFER_HEIGHT: usize = 25;
/// vga buffer memory address
const VGA_BUFFER_ADDR: usize = 0xb8000;
const VGA_INDEX_REGISTER: u16 = 0x3D4;
const VGA_DATA_REGISTER: u16 = 0x3D5;

#[macro_export]
macro_rules! printlnk {
    () => (print!("\n"));
    ($($arg:tt)*) => ($crate::printk!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! printk {
    ($($arg:tt)*) => ($crate::drivers::gpu::vga_driver::_print(format_args!($($arg)*)));
}
