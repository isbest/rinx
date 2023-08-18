#[macro_export]
macro_rules! info {
    () => ($crate::print_kernel!("\n"));
    ($($arg:tt)*) => {
        $crate::print_kernel!("\x1b[32m");
        $crate::drivers::gpu::vga_buffer::_print(format_args!($($arg)*));
        $crate::print_kernel!("\n");
        $crate::print_kernel!("\x1b[0m");
    };
}

#[macro_export]
macro_rules! warn {
    () => ($crate::print_kernel!("\n"));
    ($($arg:tt)*) => {
        $crate::print_kernel!("\x1b[93m");
        $crate::drivers::gpu::vga_buffer::_print(format_args!($($arg)*));
        $crate::print_kernel!("\n");
        $crate::print_kernel!("\x1b[0m");
    };
}

#[macro_export]
macro_rules! error {
    () => ($crate::print_kernel!("\n"));
    ($($arg:tt)*) => {
        $crate::print_kernel!("\x1b[31m");
        $crate::drivers::gpu::vga_buffer::_print(format_args!($($arg)*));
        $crate::print_kernel!("\n");
        $crate::print_kernel!("\x1b[0m");
    };
}

#[macro_export]
macro_rules! debug {
    () => ($crate::print_kernel!("\n"));
    ($($arg:tt)*) => {
        $crate::print_kernel!("\x1b[36m");
        $crate::drivers::gpu::vga_buffer::_print(format_args!($($arg)*));
        $crate::print_kernel!("\n");
        $crate::print_kernel!("\x1b[0m");
    };
}
#[macro_export]
macro_rules! trace {
    () => ($crate::print_kernel!("\n"));
    ($($arg:tt)*) => {
        $crate::print_kernel!("\x1b[90m");
        $crate::drivers::gpu::vga_buffer::_print(format_args!($($arg)*));
        $crate::print_kernel!("\n");
        $crate::print_kernel!("\x1b[0m");
    };
}
