#[macro_export]
macro_rules! info {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => {
        $crate::print!("\x1b[32m");
        $crate::print!("{}:{} ", file!(), line!());
        $crate::drivers::gpu::vga_buffer::_print(format_args!($($arg)*));
        $crate::print!("\n");
        $crate::print!("\x1b[0m");
    };
}

#[macro_export]
macro_rules! warn {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => {
        $crate::print!("\x1b[93m");
        $crate::print!("{}:{} ", file!(), line!());
        $crate::drivers::gpu::vga_buffer::_print(format_args!($($arg)*));
        $crate::print!("\n");
        $crate::print!("\x1b[0m");
    };
}

#[macro_export]
macro_rules! error {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => {
        $crate::print!("\x1b[31m");
        $crate::print!("{}:{} ", file!(), line!());
        $crate::drivers::gpu::vga_buffer::_print(format_args!($($arg)*));
        $crate::print!("\n");
        $crate::print!("\x1b[0m");
    };
}

#[macro_export]
macro_rules! debug {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => {
        $crate::print!("\x1b[36m");
        $crate::print!("{}:{} ", file!(), line!());
        $crate::drivers::gpu::vga_buffer::_print(format_args!($($arg)*));
        $crate::print!("\n");
        $crate::print!("\x1b[0m");
    };
}
#[macro_export]
macro_rules! trace {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => {
        $crate::print!("\x1b[90m");
        $crate::print!("{}:{} ", file!(), line!());
        $crate::drivers::gpu::vga_buffer::_print(format_args!($($arg)*));
        $crate::print!("\n");
        $crate::print!("\x1b[0m");
    };
}
