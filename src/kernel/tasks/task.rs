//! Rust x86 use System V ABI default
//! caller saved eax, ecx, edx
//! callee saved ebx, esi, edi, ebp, esp
use alloc::alloc::alloc;
use core::alloc::Layout;
use core::arch::asm;
use core::fmt::{Display, Formatter};
use core::mem::size_of;
use core::ptr::{NonNull, Unique};
use core::{mem, ptr};

use x86::bits32::paging::BASE_PAGE_SIZE;

use crate::kernel::interrupts::clock::{JIFFIES, JIFFY};
use crate::kernel::interrupts::{if_enabled, without_interrupt};
use crate::kernel::tasks::{
    DEFAULT_BLOCK_LINKED_LIST, IDLE_TASK, SLEEP_TASK_LIST, TASKS, TASKS_NUMBER,
};
use crate::libs::kernel_linked_list::{LinkedList, Node};
use crate::mm::page::KERNEL_PAGE_DIR;
use crate::KERNEL_MAGIC;

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
    pub ticks: u64,
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

impl Display for TaskState {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            TaskState::TaskInit => f.write_str("INIT"),
            TaskState::TaskRunning => f.write_str("Running"),
            TaskState::TaskReady => f.write_str("Ready"),
            TaskState::TaskBlocked => f.write_str("Blocked"),
            TaskState::TaskSleep => f.write_str("Sleep"),
            TaskState::TaskWaiting => f.write_str("Waiting"),
            TaskState::TaskDied => f.write_str("Died"),
        }
    }
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

        let task_mut = unsafe { task_frame.as_mut() };
        task_mut.ebx = 0x11111111;
        task_mut.esi = 0x22222222;
        task_mut.edi = 0x33333333;
        task_mut.ebp = 0x44444444;
        task_mut.eip = Some(target);

        let task_mut = unsafe { task.as_mut() };
        task_mut.name = name;
        task_mut.priority = priority;
        task_mut.uid = uid;
        task_mut.jiffies = 0;
        task_mut.state = TaskState::TaskReady;
        task_mut.magic_number = KERNEL_MAGIC;
        task_mut.pde = KERNEL_PAGE_DIR;
        // 内核栈
        task_mut.stack = task_frame.as_ptr() as u32;

        task
    }

    pub const fn from_ptr(raw_ptr: usize) -> *mut Task {
        #[allow(unused_mut)]
        let task = raw_ptr as *mut Task;

        task
    }

    pub fn current_task() -> NonNull<Task> {
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

            NonNull::new_unchecked(current)
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
                                || task.as_ref().jiffies
                                    < res_task.as_ref().jiffies
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

    pub unsafe fn block(
        mut task: NonNull<Task>,
        state: TaskState,
        block_list: Option<*mut LinkedList<()>>,
    ) {
        // 必须保证不可中断
        assert!(!if_enabled());
        assert_ne!(state, TaskState::TaskRunning);
        assert_ne!(state, TaskState::TaskReady);

        assert!(task.as_ref().node.next.is_none());
        assert!(task.as_ref().node.prev.is_none());

        // 增加默认的阻塞队列
        if let Some(block_list) = block_list {
            // 头插法
            if let Some(block_list) = block_list.as_mut() {
                block_list.push_front_node(Unique::from(NonNull::from(
                    &task.as_ref().node,
                )));
            }
        } else {
            // 插入默认队列
            DEFAULT_BLOCK_LINKED_LIST.push_front_node(Unique::from(
                NonNull::from(&task.as_ref().node),
            ))
        }

        task.as_mut().state = state;

        let current = Task::current_task();
        // 如果是当前线程自己阻塞了自己,那么需要调度到其他线程
        if current.as_ptr() == task.as_ptr() {
            Task::schedule();
        }
    }

    pub unsafe fn unblock(
        task: Option<NonNull<Task>>,
        block_list: Option<*mut LinkedList<()>>,
    ) {
        if task.is_none() {
            return;
        }

        // shadow
        let mut task = task.unwrap();

        // 必须保证不可中断
        assert!(!if_enabled());

        // 增加默认的阻塞队列
        if let Some(block_list) = block_list {
            // 节点移除队列
            if let Some(block_list) = block_list.as_mut() {
                block_list.unlink_node(NonNull::from(&task.as_ref().node));
            }
        } else {
            DEFAULT_BLOCK_LINKED_LIST
                .unlink_node(NonNull::from(&task.as_ref().node))
        }

        // 确保移出队列
        assert!(task.as_ref().node.next.is_none());
        assert!(task.as_ref().node.prev.is_none());

        // 改为就绪状态
        task.as_mut().state = TaskState::TaskReady;
    }

    pub unsafe fn sleep(ms: u32) {
        // 必须保证不可中断
        assert!(!if_enabled());

        // 计算需要睡眠的时间片
        let mut sleep_ticks = ms as usize / JIFFY;
        // 至少睡一个时间片
        if sleep_ticks == 0 {
            sleep_ticks = 1;
        }

        let mut current = Task::current_task();
        current.as_mut().ticks = *JIFFIES.lock() + sleep_ticks as u64;

        // 确保节点没有被加入其他队列
        assert!(current.as_ref().node.next.is_none());
        assert!(current.as_ref().node.prev.is_none());

        // 插入排序,找到ticks大于自己的第一个节点,然后插入到这个节点的前面
        let anchor = SLEEP_TASK_LIST.lock().find_node(|node| {
            let task = Task::get_task(node);
            if let Some(task) = task {
                // 找到时间片小于等于全局时间片的任务
                task.as_ref().ticks > current.as_ref().ticks
            } else {
                false
            }
        });

        // 没找到就是我是最大的,插入到队列尾部
        if anchor.is_none() {
            SLEEP_TASK_LIST
                .lock()
                .push_back_node(Unique::from(NonNull::from(
                    &current.as_ref().node,
                )));
        } else {
            SLEEP_TASK_LIST.lock().insert_before_node(
                anchor,
                NonNull::from(&current.as_ref().node),
            );
        }

        // 设置状态为sleep,之后任务调度便不会再调度到这个线程,ticks也不会减少
        current.as_mut().state = TaskState::TaskSleep;
        // 主动调度到其他任务
        Task::schedule();
    }

    // 唤醒所有睡觉的任务
    pub unsafe fn wake_up() {
        assert!(!if_enabled());
        let mut current_node = SLEEP_TASK_LIST.lock().front_node();

        while let Some(mut node) = current_node {
            let task = Task::get_task(node);
            if let Some(mut task) = task {
                // 由于插入是有序的,所以只要遇到第一个还没有睡够的任务就可以直接结束了
                if task.as_ref().ticks > *JIFFIES.lock() {
                    break;
                }

                // 先切换到下一个节点
                current_node = node.as_mut().next;

                // 再将节点移出队列
                task.as_mut().ticks = task.as_ref().priority as u64;
                SLEEP_TASK_LIST.lock().unlink_node(node);
                // 确保移出队列
                node.as_mut().next = None;
                node.as_mut().prev = None;

                // 改为就绪状态
                task.as_mut().state = TaskState::TaskReady;
            }
        }
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

    // 通过Node的指针求Task的指针
    pub fn get_task(node_ptr: NonNull<Node<()>>) -> Option<NonNull<Task>> {
        let offset = mem::offset_of!(Task, node);
        unsafe {
            NonNull::new(
                (node_ptr.as_ptr() as *const u8).offset(-(offset as isize))
                    as *mut Task,
            )
        }
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
