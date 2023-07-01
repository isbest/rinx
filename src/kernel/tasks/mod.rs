use crate::kernel::interrupts::enable_interrupt;
use core::arch::asm;
use core::ptr::Unique;
use spin::Mutex;

use crate::kernel::system_call::sys_call::sys_yield;
use crate::kernel::tasks::task::Task;
use crate::KERNEL_MAGIC;

pub mod task;

const TASKS_NUMBER: usize = 64;
static TASKS: Mutex<[Option<Unique<Task>>; TASKS_NUMBER]> =
    Mutex::new([None; TASKS_NUMBER]);

fn thread_a() -> u32 {
    use crate::print;

    enable_interrupt(true);
    loop {
        print!("A");
        sys_yield();
    }
}

fn thread_b() -> u32 {
    use crate::print;

    enable_interrupt(true);
    loop {
        print!("B");
        sys_yield();
    }
}

fn thread_c() -> u32 {
    use crate::kernel::time::now_time;
    use crate::print;

    enable_interrupt(true);
    loop {
        print!("\n{}\n", now_time());
        sys_yield();
    }
}

unsafe fn task_setup() {
    let mut current = Task::current_task();
    current.as_mut().magic_number = KERNEL_MAGIC;
    current.as_mut().ticks = 1;
}

pub fn init_task() {
    unsafe {
        // 初始化0x10000的的任务
        task_setup();

        // 测试 系统调用
        asm!("mov eax , 0", "int 0x80");

        Task::create(thread_a, "A", 10, 0);
        Task::create(thread_b, "B", 5, 0);
        Task::create(thread_c, "C", 6, 0);
    }
}
