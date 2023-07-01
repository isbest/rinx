use crate::kernel::interrupts::{enable_interrupt, without_interrupt};
use core::ptr;
use core::ptr::Unique;
use spin::Mutex;

use crate::kernel::tasks::task::{Task, TaskState};
use crate::libs::kernel_linked_list::LinkedList;
use crate::{delay, KERNEL_MAGIC};

pub mod task;

/// 任务数量
const TASKS_NUMBER: usize = 64;
/// 任务列表
static TASKS: Mutex<[Option<Unique<Task>>; TASKS_NUMBER]> =
    Mutex::new([None; TASKS_NUMBER]);

/// 阻塞队列
static BLOCK_TASK_LIST: Mutex<LinkedList<()>> = Mutex::new(LinkedList::new());

fn thread_a() -> u32 {
    use crate::print;

    enable_interrupt(true);
    loop {
        delay(1000000);
        print!("A");
        test();
    }
}

fn thread_b() -> u32 {
    use crate::print;

    enable_interrupt(true);
    loop {
        delay(1000000);
        print!("B");
        test();
    }
}

fn thread_c() -> u32 {
    use crate::print;

    enable_interrupt(true);
    loop {
        delay(1000000);
        print!("C");
        test();
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

        Task::create(thread_a, "A", 10, 0);
        Task::create(thread_b, "B", 5, 0);
        Task::create(thread_c, "C", 6, 0);
    }
}

static mut TEMP_TASK: *mut Task = ptr::null_mut();

fn test() {
    without_interrupt(|| unsafe {
        if TEMP_TASK.is_null() {
            let task = Task::current_task();
            TEMP_TASK = task.as_ptr();
            Task::block(task, TaskState::TaskBlocked);
        } else {
            Task::unblock(Unique::new_unchecked(TEMP_TASK));
            TEMP_TASK = ptr::null_mut();
        }
    });
}
