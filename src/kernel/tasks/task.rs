//! Rust x86 use System V ABI default
//! caller saved eax, ecx, edx
//! callee saved ebx, esi, edi, ebp, esp
use alloc::alloc::alloc;
use core::alloc::Layout;
use core::arch::asm;
use core::mem::size_of;
use core::ptr;
use core::sync::atomic::Ordering;

use crate::kernel::interrupts::{eflags_if, without_interrupt};
use crate::kernel::tasks::{TASKS, TASKS_NUMBER};
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
    // 栈顶地址,返回地址
    // 内核栈
    pub stack: *mut u32,
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
    pub unsafe fn create(
        target: TargetFn,
        name: &'static str,
        priority: u32,
        uid: u32,
    ) -> *mut Task {
        // 计算栈顶地址,栈从高地址向低地址增长
        // 所以加上BASE_PAGE_SIZE来计算栈顶
        let task = Task::get_free_task();
        let mut stack = task as usize + BASE_PAGE_SIZE;

        // 指向task frame的指针,结构体是从低地址向高地址的
        stack -= size_of::<TaskFrame>();
        unsafe {
            // 减去函数指针的位置
            // 将栈顶保存成函数的上下问,所以将栈顶减去函数上下文的大小
            let frame = stack as *mut TaskFrame;
            (*frame).ebx = 0x11111111;
            (*frame).esi = 0x22222222;
            (*frame).edi = 0x33333333;
            (*frame).ebp = 0x44444444;
            (*frame).eip = Some(target);
        }

        unsafe {
            (*task).name = name;
            (*task).priority = priority;
            (*task).uid = uid;
            (*task).jiffies = 0;
            (*task).state = TaskState::TaskReady;
            (*task).magic_number = KERNEL_MAGIC;
            (*task).pde = KERNEL_PAGE_DIR;

            // 修改返回地址
            (*task).stack = stack as *mut u32;
        }

        task
    }

    pub const fn from_ptr(raw_ptr: usize) -> *mut Task {
        #[allow(unused_mut)]
        let task = raw_ptr as *mut Task;

        task
    }

    pub fn current_task() -> *mut Task {
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

            &mut *current
        }
    }

    pub unsafe fn schedule() {
        // 不需保证不可中断
        assert!(!eflags_if());

        let current = Task::current_task();
        // 查找就绪的任务
        let next = Task::task_search(TaskState::TaskReady);

        // 不能是默认值
        assert!(!next.is_null(), "next task can not be null {:p}", next);
        // 不能栈溢出
        assert_eq!(
            (*next).magic_number,
            KERNEL_MAGIC,
            "next task:{:p} stack overflow",
            next
        );

        // 修改当前任务从Running -> Ready
        if (*current).state == TaskState::TaskRunning {
            (*current).state = TaskState::TaskReady
        }

        (*next).state = TaskState::TaskRunning;

        if ptr::eq(next, current) {
            return;
        }

        task_switch(next);
    }

    pub unsafe fn get_free_task() -> *mut Task {
        let task_layout =
            Layout::from_size_align(size_of::<Task>(), BASE_PAGE_SIZE)
                .expect("init task error");

        let free_task = unsafe { alloc(task_layout) as *mut Task };

        // 0初始化
        ptr::write_bytes(free_task, 0, 1);

        for index in 0..TASKS_NUMBER {
            let task = TASKS.lock()[index].load(Ordering::Relaxed);
            if task.is_null() {
                TASKS.lock()[index].swap(free_task, Ordering::Relaxed);
                break;
            }
        }

        free_task
    }

    pub unsafe fn task_search(state: TaskState) -> *mut Task {
        without_interrupt(|| {
            // 0初始化
            let mut result: *mut Task = ptr::null_mut();
            let current_task = Task::current_task();

            (0..TASKS_NUMBER).for_each(|index| {
                let task = TASKS.lock()[index].load(Ordering::Relaxed);

                if task.is_null() || task == current_task {
                    return;
                }

                if (*task).state != state {
                    return;
                }

                if result.is_null()
                    || (*result).ticks < (*task).ticks
                    || (*task).jiffies < (*result).jiffies
                {
                    result = task;
                }
            });

            result
        })
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
