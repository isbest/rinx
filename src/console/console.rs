use alloc::collections::VecDeque;
use crate::console::cell::Cell;
use crate::console::text_buffet::TextBuffer;

pub struct Console<T:  TextBuffer> {
    inner: T
}

#[derive(Debug, Default, Clone, Copy)]
struct Cursor {
    row: usize,
    col: usize,
}

struct ConsoleInner<T: TextBuffer> {
    /// cursor
    cursor: Cursor,
    /// Saved cursor
    saved_cursor: Cursor,
    /// current attribute template
    temp: Cell,
    /// character buffer
    buf: T,
    /// auto wrap
    auto_wrap: bool,
    /// Reported data for CSI Device Status Report
    report: VecDeque<u8>,
}