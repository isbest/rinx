use core::ptr::NonNull;
use lazy_static::lazy_static;
use pc_keyboard::layouts::Us104Key;
use pc_keyboard::{DecodedKey, HandleControl, KeyCode, Keyboard, ScancodeSet1};
use x86::io::{inb, outb};

use crate::kernel::interrupts::handler::set_interrupt_handler;
use crate::kernel::interrupts::pic::pic_controller::send_eoi;
use crate::kernel::interrupts::{
    set_interrupt_mask, IRQ_KEYBOARD, IRQ_MASTER_NR,
};
use crate::kernel::sync::mutex::Mutex;
use crate::kernel::tasks::task::{Task, TaskState};
use crate::libs::circular_queue::CircularQueue;

const KEYBOARD_DATA_PORT: u16 = 0x60;
const KEYBOARD_CTRL_PORT: u16 = 0x60;
/// 设置LED状态
const KEYBOARD_CMD_LED: u8 = 0xED;
const KEYBOARD_CMD_ACK: u8 = 0xFA;

/// 键盘中断处理函数
pub extern "C" fn keyboard_handler(
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
    assert_eq!(vector, 0x21, "must be 0x21 keyboard interrupt");

    let scancode = unsafe {
        // 从键盘数据寄存器读取键盘信息扫描码
        inb(KEYBOARD_DATA_PORT)
    };

    // 解析键盘的scancode
    parser_scancode(scancode);

    send_eoi(vector);
}

/// 键盘大小写锁定
static mut CAPSLOCK_STATE: bool = false;
/// 键盘的缓冲区
static mut KEYBOARD_BUFFER: CircularQueue<char, 60> = CircularQueue::new();
/// 等待读入键盘的任务
static mut WAITER: Option<NonNull<Task>> = None;

/// 解析扫描码,PS/2键盘驱动的关键,利用`pc_keyboard`crate实现
fn parser_scancode(scancode: u8) {
    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(
                ScancodeSet1::new(),
                Us104Key,
                HandleControl::Ignore
            ));
    }

    let mut keyboard = KEYBOARD.lock();
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => unsafe {
                    // 压入队列
                    KEYBOARD_BUFFER.enqueue(character);

                    // 等待的任务存在,则去唤醒这个任务
                    if WAITER.is_some() {
                        // 必须要调用take,消费掉这个阻塞的任务
                        Task::unblock(WAITER.take(), None);
                    }
                },
                DecodedKey::RawKey(key) => {
                    if KeyCode::CapsLock == key {
                        unsafe {
                            CAPSLOCK_STATE = !CAPSLOCK_STATE;
                            set_led(CAPSLOCK_STATE);
                        }
                    }
                }
            }
        }
    }
}

/// 读取键盘缓存的方法
pub fn read_keyboard(buffer: &mut [char]) {
    let mut nr = 0usize;
    // KEYBOARD_BUFFER 不应该上锁,或者说应该上读写锁
    // 如果读先获取锁,且缓冲区没有东西,那么读就永远不能获取锁,造成死锁
    while nr < buffer.len() {
        unsafe {
            while KEYBOARD_BUFFER.is_empty() {
                let current_task = Task::current_task();
                WAITER = Some(current_task);
                // 阻塞当前任务,等待输入
                Task::block(current_task, TaskState::TaskWaiting, None);
            }

            // 到这buffer就不可能是空了
            buffer[nr] = KEYBOARD_BUFFER.dequeue().unwrap().unwrap();
            nr += 1;
        }
    }
}

// 等待缓冲区为空
#[inline(always)]
fn keyboard_wait() {
    let mut state: u8 = 0;
    while state & 0x02 != 0 {
        unsafe {
            state = inb(KEYBOARD_CTRL_PORT);
        }
    }
}

// 等待键盘回复ack
#[inline(always)]
fn keyboard_ack() {
    let mut state: u8 = 0;
    while state != KEYBOARD_CMD_ACK {
        unsafe {
            state = inb(KEYBOARD_DATA_PORT);
        }
    }
}

// 设置键盘LED灯状态
fn set_led(state: bool) {
    let led: u8 = if state { 1 << 2 } else { 0 };
    keyboard_wait();
    unsafe {
        outb(KEYBOARD_DATA_PORT, KEYBOARD_CMD_LED);
    }
    keyboard_ack();
    keyboard_wait();
    unsafe {
        outb(KEYBOARD_DATA_PORT, led);
    }
    keyboard_ack();
}

pub fn init_keyboard() {
    set_interrupt_handler(
        IRQ_MASTER_NR + IRQ_KEYBOARD as usize,
        keyboard_handler,
    );
    set_interrupt_mask(IRQ_KEYBOARD, true);
}
