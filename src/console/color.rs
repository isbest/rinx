/// indexing a color list.
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum NamedColor {
    /// Black.
    Black = 0,
    /// Red.
    Red = 1,
    /// Green.
    Green = 2,
    /// Yellow.
    Yellow = 3,
    /// Blue.
    Blue = 4,
    /// Magenta.
    Magenta = 5,
    /// Cyan.
    Cyan = 6,
    /// White.
    White = 7,
    /// Bright black.
    BrightBlack = 8,
    /// Bright red.
    BrightRed = 9,
    /// Bright green.
    BrightGreen = 10,
    /// Bright yellow.
    BrightYellow = 11,
    /// Bright blue.
    BrightBlue = 12,
    /// Bright magenta.
    BrightMagenta = 13,
    /// Bright cyan.
    BrightCyan = 14,
    /// Bright white.
    BrightWhite = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Named(NamedColor),
    Indexed(u8),
}
