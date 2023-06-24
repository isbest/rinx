//! Rust x86 use System V ABI default
//! caller saved eax, ecx, edx
//! callee saved ebx, esi, edi, ebp, esp
use core::arch::asm;
use core::mem::size_of;

use x86::bits32::paging::BASE_PAGE_SIZE;

type TargetFn = fn() -> u32;

/// 任务,用一页表示一个任务,用栈底信息(页开始的地方表示这个任务)
#[repr(transparent)]
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Task {
    // 栈顶地址,返回地址
    stack: *mut u32,
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
    pub fn create(task: *mut Task, target: TargetFn) -> *mut Task {
        // 计算栈顶地址,栈从高地址向低地址增长
        // 所以加上BASE_PAGE_SIZE来计算栈顶
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
        unsafe {
            // 栈是在页内,因此只需要用sp的值,就能知道栈在哪一页
            // 就能知道是哪个任务
            asm!(
                "movl %esp, %eax",
                "andl $0xfffff000, %eax",
                out("eax") current,
                options(att_syntax)
            );

            &mut *current
        }
    }

    pub fn schedule() {
        let current = Task::current_task();
        unsafe {
            let next = if core::ptr::eq(current, A) { B } else { A };
            task_switch(next);
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

fn thread_a() -> u32 {
    use crate::print;
    loop {
        print!("A");
        Task::schedule();
    }
}

fn thread_b() -> u32 {
    use crate::print;
    loop {
        print!("B");
        Task::schedule();
    }
}

const A: *mut Task = Task::from_ptr(0x106000);
const B: *mut Task = Task::from_ptr(0x107000);

pub fn init_task() {
    Task::create(A, thread_a);
    Task::create(B, thread_b);
    Task::schedule();
}
