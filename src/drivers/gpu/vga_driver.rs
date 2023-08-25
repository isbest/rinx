use crate::kernel::sync::mutex::Mutex;
use core::ptr::Unique;
use core::{fmt, ptr};
use lazy_static::lazy_static;

use crate::drivers::gpu::color::VgaColor;
use crate::drivers::gpu::{
    BUFFER_HEIGHT, BUFFER_WIDTH, VGA_BUFFER_ADDR, VGA_DATA_REGISTER,
    VGA_INDEX_REGISTER,
};
use x86::io::outb;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VgaChar {
    ascii_chara: u8,      // 显示的字符
    color_code: VgaColor, // 颜色和状态
}

impl Default for VgaChar {
    fn default() -> Self {
        Self {
            ascii_chara: b' ',
            color_code: VgaColor::default(),
        }
    }
}

impl From<u8> for VgaChar {
    fn from(value: u8) -> Self {
        VgaChar {
            ascii_chara: value,
            color_code: VgaColor::default(),
        }
    }
}

#[repr(transparent)]
pub struct VgaBuffer {
    pub(crate) chars: [[VgaChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

impl VgaBuffer {
    // 在指定位置写入
    pub(crate) fn write(&mut self, pos: Cursor, vga_char: VgaChar) {
        unsafe {
            ptr::write_volatile::<VgaChar>(
                &self.chars[pos.row][pos.col] as *const VgaChar as *mut VgaChar,
                vga_char,
            );
        }
    }

    // 在指定位置读取
    pub fn read_volatile(&self, pos: Cursor) -> VgaChar {
        unsafe {
            ptr::read_volatile(&self.chars[pos.row][pos.col] as *const VgaChar)
        }
    }

    pub fn scroll_up(&mut self, size: BufferSize) {
        unsafe {
            ptr::copy(
                self.chars.as_ptr().add(1),
                self.chars.as_mut_ptr(),
                size.height - 1,
            );

            // 清空最后一行
            for col in 0..size.width {
                self.write(Cursor::new(size.height - 1, col), b' '.into())
            }
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Cursor {
    row: usize,
    col: usize,
}

impl Cursor {
    fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct BufferSize {
    height: usize,
    width: usize,
}

impl BufferSize {
    pub fn new(height: usize, width: usize) -> Self {
        Self { height, width }
    }
}

pub struct Console {
    buffer: Unique<VgaBuffer>,
    cursor: Cursor,
    size: BufferSize,
}

impl Console {
    pub fn new(ptr: *mut VgaBuffer, size: BufferSize) -> Self {
        Console {
            buffer: unsafe { Unique::new_unchecked(ptr) },
            cursor: Cursor::default(),
            size,
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(*byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// 按字节写入
    pub fn write_byte(&mut self, byte: u8) {
        let col = self.cursor.col;
        let width = self.size.width;

        match byte {
            b'\n' => {
                // 换行
                self.new_line();
            }
            // 删除
            0x08 => {
                self.backspace();
            }
            // 其他
            byte => {
                // 自动换行逻辑
                if col > width - 1 {
                    self.new_line();
                    self._write_byte(byte);
                } else {
                    self._write_byte(byte);
                    // 列+1
                    self.cursor.col += 1;
                }
            }
        }

        // 更新指针
        self.update_cursor();
    }

    fn new_line(&mut self) {
        let row = self.cursor.row;

        // 单纯的换行
        if row < self.size.height - 1 {
            self.cursor.row += 1;
        } else {
            // 到达屏幕底部,需要拷贝内存
            self._scroll_up();
            self.cursor.row = self.size.height - 1;
        }

        self.cursor.col = 0;
    }

    // 删除一个字符
    fn backspace(&mut self) {
        self.write_byte(b' ');
        self.cursor.col -= 1;
    }

    // 对unsafe的封装
    #[inline(always)]
    fn _write_byte(&mut self, byte: u8) {
        unsafe {
            // 插入字符
            self.buffer.as_mut().write(self.cursor, byte.into());
        }
    }

    // 仅仅拷贝内存,不更新cursor
    #[inline(always)]
    fn _scroll_up(&mut self) {
        unsafe {
            self.buffer.as_mut().scroll_up(self.size);
        }
    }

    /// 更新光标位置
    fn update_cursor(&self) {
        let cursor_position = self._get_cursor_pos();

        unsafe {
            outb(VGA_INDEX_REGISTER, 0x0F);
            outb(VGA_DATA_REGISTER, (cursor_position & 0xFF) as u8);
            outb(VGA_INDEX_REGISTER, 0x0E);
            outb(VGA_DATA_REGISTER, ((cursor_position >> 8) & 0xFF) as u8);
        }
    }

    /// 获取指针位置
    fn _get_cursor_pos(&self) -> usize {
        // 计算光标位置 width * x + y
        self.size.width * self.cursor.row + self.cursor.col
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.write_string(text);
        Ok(())
    }
}

lazy_static! {
    pub static ref CONSOLE: Mutex<Console> = Mutex::new(Console::new(
        VGA_BUFFER_ADDR as *mut VgaBuffer,
        BufferSize::new(BUFFER_HEIGHT, BUFFER_WIDTH)
    ));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use crate::kernel::interrupts::without_interrupt;
    use core::fmt::Write;

    without_interrupt(|| {
        CONSOLE.lock().write_fmt(args).unwrap();
    });
}
