use crate::kernel::interrupts::enable_interrupt;
use core::arch::asm;
use core::ptr;
use core::sync::atomic::AtomicPtr;
use spin::Mutex;

use crate::kernel::tasks::task::Task;
use crate::{delay, KERNEL_MAGIC};

pub mod task;

#[doc(hidden)]
#[allow(clippy::declare_interior_mutable_const)]
const NULL_TASK_PTR: AtomicPtr<Task> = AtomicPtr::new(ptr::null_mut());

const TASKS_NUMBER: usize = 64;
static TASKS: Mutex<[AtomicPtr<Task>; TASKS_NUMBER]> =
    Mutex::new([NULL_TASK_PTR; TASKS_NUMBER]);

fn thread_a() -> u32 {
    use crate::print;

    enable_interrupt(true);
    loop {
        delay(10000000);
        print!("A");
    }
}

fn thread_b() -> u32 {
    use crate::print;

    enable_interrupt(true);
    loop {
        delay(10000000);
        print!("B");
    }
}

fn thread_c() -> u32 {
    use crate::kernel::time::now_time;
    use crate::print;

    enable_interrupt(true);
    loop {
        delay(10000000);
        print!("\n{}\n", now_time());
    }
}

unsafe fn task_setup() {
    let current = Task::current_task();
    (*current).magic_number = KERNEL_MAGIC;
    (*current).ticks = 1;
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
