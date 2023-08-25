use core::ptr;
use core::ptr::NonNull;

use x86::io::outb;

/// vga text mode buffer width
const BUFFER_WIDTH: usize = 80;
/// vga text mode buffer height
const BUFFER_HEIGHT: usize = 25;
/// vga buffer memory address
const VGA_BUFFER_ADDR: usize = 0xb8000;
const VGA_INDEX_REGISTER: u16 = 0x3D4;
const VGA_DATA_REGISTER: u16 = 0x3D5;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VgaChar {
    ascii_chara: u8, // 显示的字符
    color_code: u8,  // 颜色和状态
}

impl From<u8> for VgaChar {
    fn from(value: u8) -> Self {
        VgaChar {
            ascii_chara: value,
            color_code: 0,
        }
    }
}

#[repr(transparent)]
pub struct VgaBuffer {
    pub(crate) chars: [[VgaChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

impl VgaBuffer {
    // 在指定位置写入
    pub(crate) fn write(&mut self, pos: (usize, usize), vga_char: VgaChar) {
        unsafe {
            ptr::write_volatile::<VgaChar>(
                &self.chars[pos.0][pos.1] as *const VgaChar as *mut VgaChar,
                vga_char,
            );
        }
    }

    // 在指定位置读取
    pub fn read_volatile(&self, pos: (usize, usize)) -> VgaChar {
        unsafe {
            ptr::read_volatile(&self.chars[pos.0][pos.1] as *const VgaChar)
        }
    }
}

pub struct Console {
    /// buffer
    buffer: NonNull<VgaBuffer>,
    /// (row, col) 光标的位置,也是即将要写入字符串的位置
    cursor: (usize, usize),
    /// (width, height)
    size: (usize, usize),
    // todo maybe more state
}

impl Console {
    pub fn new(ptr: *mut VgaBuffer, size: (usize, usize)) -> Self {
        Console {
            buffer: unsafe { NonNull::new_unchecked(ptr) },
            cursor: (0, 0),
            size,
        }
    }

    /// 更新光标位置
    fn update_cursor(&self) {
        let cursor_position = self.get_cursor_pos();

        unsafe {
            outb(VGA_INDEX_REGISTER, 0x0F);
            outb(VGA_DATA_REGISTER, (cursor_position & 0xFF) as u8);
            outb(VGA_INDEX_REGISTER, 0x0E);
            outb(VGA_DATA_REGISTER, ((cursor_position >> 8) & 0xFF) as u8);
        }
    }

    /// 获取指针位置
    fn get_cursor_pos(&self) -> usize {
        // 计算光标位置 width * x + y
        self.size.0 * self.cursor.0 + self.cursor.1
    }

    /// 按字节写入
    pub fn write_byte(&mut self, byte: u8) {
        let (row, col) = self.cursor;
        let (width, height) = self.size;

        match byte {
            b'\n' => {
                // 换行
            }
            // 删除
            b' ' => {}
            // 其他
            byte => {
                // 自动换行逻辑
                if col > width - 1 {
                    // 新的一行
                    self.cursor.0 += 1;
                    // 列置为0
                    self.cursor.1 = 0;

                    unsafe {
                        // 插入字符
                        self.buffer.as_mut().write((row + 1, 1), byte.into());
                    }
                }
            }
        }
    }
}
