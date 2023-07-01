//! Rust x86 use System V ABI default
//! caller saved eax, ecx, edx
//! callee saved ebx, esi, edi, ebp, esp
use alloc::alloc::alloc;
use core::alloc::Layout;
use core::arch::asm;
use core::mem::size_of;
use core::ptr;
use core::ptr::{NonNull, Unique};

use crate::kernel::interrupts::{if_enabled, without_interrupt};
use crate::kernel::tasks::{BLOCK_TASK_LIST, IDLE_TASK, TASKS, TASKS_NUMBER};
use crate::libs::kernel_linked_list::Node;
use crate::mm::page::KERNEL_PAGE_DIR;
use crate::KERNEL_MAGIC;
use x86::bits32::paging::BASE_PAGE_SIZE;

type TargetFn = fn() -> u32;

/// 任务,用一页表示一个任务,用栈底信息(页开始的地方表示这个任务)
/// 按照4096个字节对齐
#[repr(C)]
#[repr(align(4096))]
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Task {
    // 内核栈地址
    pub stack: u32,
    // 阻塞队列
    pub node: Node<()>,
    // 任务状态
    pub state: TaskState,
    // 优先级
    pub priority: u32,
    // 剩余时间片
    pub ticks: i32,
    // 上次执行时全局时间片
    pub jiffies: u64,
    // 任务名
    pub name: &'static str,
    // 用户id
    pub uid: u32,
    // 页目录物理地址
    pub pde: u32,
    // 魔数
    pub magic_number: u32,
}

#[repr(C)]
#[allow(clippy::enum_variant_names)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TaskState {
    TaskInit,
    TaskRunning,
    TaskReady,
    TaskBlocked,
    TaskSleep,
    TaskWaiting,
    TaskDied,
}

/// 任务上下文,切换前保存,切换后恢复
pub struct TaskFrame {
    edi: u32,
    esi: u32,
    ebx: u32,
    ebp: u32,
    eip: Option<TargetFn>,
}

impl Task {
    pub fn create(
        target: TargetFn,
        name: &'static str,
        priority: u32,
        uid: u32,
    ) -> Unique<Task> {
        // 计算栈顶地址,栈从高地址向低地址增长
        // 所以加上BASE_PAGE_SIZE来计算栈顶
        let mut task = Task::get_free_task();
        let mut task_frame = Task::get_task_frame(task);
        unsafe {
            task_frame.as_mut().ebx = 0x11111111;
            task_frame.as_mut().esi = 0x22222222;
            task_frame.as_mut().edi = 0x33333333;
            task_frame.as_mut().ebp = 0x44444444;
            task_frame.as_mut().eip = Some(target);

            task.as_mut().name = name;
            task.as_mut().priority = priority;
            task.as_mut().uid = uid;
            task.as_mut().jiffies = 0;
            task.as_mut().state = TaskState::TaskReady;
            task.as_mut().magic_number = KERNEL_MAGIC;
            task.as_mut().pde = KERNEL_PAGE_DIR;
            // 内核栈
            task.as_mut().stack = task_frame.as_ptr() as u32;
        }
        task
    }

    pub const fn from_ptr(raw_ptr: usize) -> *mut Task {
        #[allow(unused_mut)]
        let task = raw_ptr as *mut Task;

        task
    }

    pub fn current_task() -> Unique<Task> {
        let current: *mut Task;
        // 栈是在页内,因此只需要用sp的值,就能知道栈在哪一页
        // 就能知道是哪个任务
        unsafe {
            asm!(
            "movl %esp, %eax",
            "andl $0xfffff000, %eax",
            out("eax") current,
            options(att_syntax)
            );

            Unique::new_unchecked(current)
        }
    }

    pub unsafe fn schedule() {
        // 必须保证不可中断 if 为0表示关闭外中断
        assert!(!if_enabled());

        let mut current = Task::current_task();
        // 查找就绪的任务
        let next = Task::task_search(TaskState::TaskReady);

        // 不能是默认值
        assert!(next.is_some(), "next task can not be null");

        let mut next = next.unwrap();
        assert_eq!(
            next.as_ref().magic_number,
            KERNEL_MAGIC,
            "next task:{:p} stack overflow",
            next
        );

        // 修改当前任务从Running -> Ready
        if current.as_mut().state == TaskState::TaskRunning {
            current.as_mut().state = TaskState::TaskReady
        }

        next.as_mut().state = TaskState::TaskRunning;

        if ptr::eq(next.as_ptr(), current.as_ptr()) {
            return;
        }

        task_switch(next.as_ptr());
    }

    pub fn get_free_task() -> Unique<Task> {
        let task_layout =
            Layout::from_size_align(size_of::<Task>(), BASE_PAGE_SIZE)
                .expect("init task error");

        let free_task =
            unsafe { Unique::new_unchecked(alloc(task_layout) as *mut Task) };

        let pos = TASKS.lock().iter().position(Option::is_none);

        // 不能写在一行,垃圾spin.lock会造成死锁
        if let Some(index) = pos {
            TASKS.lock()[index] = Some(free_task);
        };

        free_task
    }

    pub unsafe fn task_search(state: TaskState) -> Option<Unique<Task>> {
        let mut result = None;
        without_interrupt(|| {
            let current_task = Task::current_task();

            (0..TASKS_NUMBER).for_each(|index| {
                let task = TASKS.lock()[index];

                if let Some(task) = task {
                    if task.as_ptr() == current_task.as_ptr() {
                        return;
                    }

                    if task.as_ref().state != state {
                        return;
                    }

                    if result.is_none()
                        || result.is_some_and(|res_task: Unique<Task>| {
                            res_task.as_ref().ticks < task.as_ref().ticks
                                || res_task.as_ref().jiffies
                                    < task.as_ref().jiffies
                        })
                    {
                        result = Some(task);
                    }
                };
            });

            // 没有就绪任务,则切换到idle任务
            if result.is_none() {
                result = Some(IDLE_TASK);
            }

            result
        })
    }

    pub unsafe fn block(mut task: Unique<Task>, state: TaskState) {
        // 必须保证不可中断
        assert!(!if_enabled());
        assert_ne!(state, TaskState::TaskRunning);
        assert_ne!(state, TaskState::TaskReady);

        assert!(task.as_ref().node.next.is_none());
        assert!(task.as_ref().node.prev.is_none());

        // 头插法
        BLOCK_TASK_LIST
            .lock()
            .push_front_node(Unique::from(NonNull::from(&task.as_ref().node)));

        task.as_mut().state = state;

        let current = Task::current_task();
        // 如果是当前线程自己阻塞了自己,那么需要调度到其他线程
        if current.as_ptr() == task.as_ptr() {
            Task::schedule();
        }
    }

    pub unsafe fn unblock(mut task: Unique<Task>) {
        // 必须保证不可中断
        assert!(!if_enabled());

        BLOCK_TASK_LIST
            .lock()
            .unlink_node(NonNull::from(&task.as_ref().node));
        // 确保移出队列
        assert!(task.as_ref().node.next.is_none());
        assert!(task.as_ref().node.prev.is_none());

        // 改为就绪状态
        task.as_mut().state = TaskState::TaskReady;
    }
}

/// private func
impl Task {
    fn get_task_frame(task: Unique<Task>) -> Unique<TaskFrame> {
        // 计算上下文的地址
        // 栈是从高地址向低地址增长的,任务是从一页的起始位置开始分配的
        // 把一页的末尾(高地址)用来保存任务上下文,那么上下文的起始地址就是内核栈的栈底
        let stack =
            task.as_ptr() as usize + BASE_PAGE_SIZE - size_of::<TaskFrame>();
        let frame = stack as *mut TaskFrame;
        unsafe { Unique::new_unchecked(frame) }
    }
}

/// 任务切换
#[naked]
#[link_section = ".text"]
pub unsafe extern "C" fn task_switch(next: *mut Task) {
    asm!(
        "pushl %ebp",
        "movl %esp, %ebp",
        "pushl %ebx",
        "pushl %esi",
        "pushl %edi",
        "movl %esp, %eax",
        // current
        "andl $0xfffff000, %eax",
        "movl %esp, (%eax)",
        // next
        "movl 8(%ebp), %eax",
        "movl (%eax), %esp",
        "popl %edi",
        "popl %esi",
        "popl %ebx",
        "popl %ebp",
        "ret",
        options(noreturn, att_syntax)
    );
}
