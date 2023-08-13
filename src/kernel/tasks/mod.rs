use crate::kernel::sync::mutex::Mutex;
use core::ptr::Unique;

use crate::kernel::tasks::task::Task;
use crate::kernel::tasks::thread::idle::idle;
use crate::kernel::tasks::thread::init::init;
use crate::libs::kernel_linked_list::LinkedList;
use crate::KERNEL_MAGIC;

pub mod task;
mod thread;

/// 任务数量
const TASKS_NUMBER: usize = 64;
/// 任务列表
static TASKS: Mutex<[Option<Unique<Task>>; TASKS_NUMBER]> =
    Mutex::new([None; TASKS_NUMBER]);
/// 默认的阻塞队列
static mut DEFAULT_BLOCK_LINKED_LIST: LinkedList<()> = LinkedList::new();

/// IDLE任务指针
static mut IDLE_TASK: Unique<Task> = Unique::dangling();

/// 内核用户
const KERNEL_USER: u32 = 0;
/// 普通用户
const NORMAL_USER: u32 = 1000;

/// 睡眠队列
static SLEEP_TASK_LIST: Mutex<LinkedList<()>> = Mutex::new(LinkedList::new());

unsafe fn task_setup() {
    let mut current = Task::current_task();
    current.as_mut().magic_number = KERNEL_MAGIC;
    current.as_mut().ticks = 1;
}

pub fn init_task() {
    unsafe {
        // 初始化0x10000的的任务
        task_setup();
        // idle任务优先级为1,永远不会被调度,除非没有就绪任务
        IDLE_TASK = Task::create(idle, "idle", 1, KERNEL_USER);
        Task::create(init, "init", 5, NORMAL_USER);
    }
}
