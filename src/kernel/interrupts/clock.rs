use spin::Mutex;

use crate::kernel::interrupts::handler::set_interrupt_handler;
use crate::kernel::interrupts::pic::pic_controller::send_eoi;
use crate::kernel::interrupts::{set_interrupt_mask, IRQ_CLOCK, IRQ_MASTER_NR};
use crate::kernel::tasks::task::Task;
use crate::KERNEL_MAGIC;

/// 计数器0
const PIT_CHAN0_REG: u16 = 0x40;
/// 计数器2
const PIT_CHAN2_REG: u16 = 0x42;
/// 控制字寄存器
const PIT_CTRL_REG: u16 = 0x43;

/// 需要发生的中断频率
pub const HZ: usize = 100;
/// 振荡器频率
const OSCILLATOR: usize = 1193182;
/// 时钟计数器 = 振荡频率 / HZ (振荡多少次发生一次中断)
pub const CLOCK_COUNTER: usize = OSCILLATOR / HZ;
// 每个中断发生的时间间隔
pub const JIFFY: usize = 1000 / HZ;

/// 时间片计数器
pub static JIFFIES: Mutex<u64> = Mutex::new(0);

/// 时钟中断处理函数
pub extern "C" fn clock_handler(
    vector: u32,
    _edi: u32,
    _esi: u32,
    _ebp: u32,
    _esp: u32,
    _ebx: u32,
    _edx: u32,
    _ecx: u32,
    _eax: u32,
    _gs: u32,
    _fs: u32,
    _es: u32,
    _ds: u32,
    _vector0: u32,
    _error_code: u32,
    _eip: u32,
    _cs: u32,
    _eflags: u32,
) {
    use core::ops::AddAssign;
    // 必须是时钟中断
    assert_eq!(vector, 0x20);

    // 通知中断控制器中断处理结束
    send_eoi(vector);

    JIFFIES.lock().add_assign(1);

    let current = Task::current_task();
    unsafe {
        assert_eq!((*current).magic_number, KERNEL_MAGIC, "{:p}", current);
        (*current).jiffies = *JIFFIES.lock();
        (*current).ticks -= 1;

        if (*current).ticks <= 0 {
            (*current).ticks = (*current).priority as i32;
            // 调度任务
            Task::schedule();
        }
    }
}

fn init_pit() {
    use x86::io::{outb, outw};
    unsafe {
        outb(PIT_CTRL_REG, 0b00110100);
        outw(PIT_CHAN0_REG, (CLOCK_COUNTER & 0xff) as u16);
        outw(PIT_CHAN0_REG, ((CLOCK_COUNTER >> 8) as u16) & 0xff);
    }
}

pub fn init_clock() {
    init_pit();
    set_interrupt_handler(IRQ_MASTER_NR + IRQ_CLOCK as usize, clock_handler);
    set_interrupt_mask(IRQ_CLOCK, true);
}
