use crate::kernel::system_call::sys_call::sys_sleep;
use crate::kernel::tasks::task::Task;

pub(crate) fn init() -> ! {
    let mut use_stack = [' '; 10];
    use_stack[9] = 'a';
    unsafe { Task::task_to_user_mode(real_init) }
    #[warn(clippy::empty_loop)]
    loop {}
}

fn real_init() -> ! {
    loop {
        sys_sleep(500);
    }
}
