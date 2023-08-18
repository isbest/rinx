use crate::kernel::global::KERNEL_CODE_SELECTOR;
use crate::kernel::interrupts::handler::INTERRUPT_HANDLER_TABLE;
use crate::kernel::interrupts::handler_entry::INTERRUPT_HANDLER_ENTRY_TABLE;
use crate::kernel::interrupts::pic::handler::default_external_handler;
use crate::kernel::interrupts::{ENTRY_SIZE, IDT_SIZE, IRQ_MASTER_NR};
use crate::kernel::sync::mutex::Mutex;
use crate::kernel::system_call::system_call;
use lazy_static::lazy_static;
use x86::dtables::{lidt, DescriptorTablePointer};
use x86::segmentation::{
    BuildDescriptor, Descriptor, DescriptorBuilder, GateDescriptorBuilder,
};
use x86::Ring::{Ring0, Ring3};

lazy_static! {
    pub static ref INTERRUPT_ENTRY: Mutex<[Descriptor; IDT_SIZE]> = {
        #[allow(unused_mut)]
        let mut interrupt_entry_table: Mutex<[Descriptor; IDT_SIZE]> =
            Mutex::new([Descriptor::default(); IDT_SIZE]);

        interrupt_entry_table
    };
}

pub fn init_idt() {
    // 初始化外中断默认处理函数
    (IRQ_MASTER_NR..ENTRY_SIZE).for_each(|index| unsafe {
        INTERRUPT_HANDLER_TABLE[index] = default_external_handler;
    });

    let mut interrupt_entry_guard = INTERRUPT_ENTRY.lock();
    (0..ENTRY_SIZE).for_each(|index| {
        interrupt_entry_guard[index] =
            <DescriptorBuilder as GateDescriptorBuilder<u32>>::interrupt_descriptor(
                KERNEL_CODE_SELECTOR,
                INTERRUPT_HANDLER_ENTRY_TABLE[index] as usize as _
            )
                .dpl(Ring0)
                .present()
                .finish();
    });

    // 初始化系统调用
    interrupt_entry_guard[0x80] = <DescriptorBuilder as GateDescriptorBuilder<u32>>::interrupt_descriptor(
        KERNEL_CODE_SELECTOR,
        system_call as usize as _
    )
        .dpl(Ring3)
        .present()
        .finish();

    unsafe {
        lidt(&DescriptorTablePointer::<[Descriptor; IDT_SIZE]>::new(
            &interrupt_entry_guard,
        ));
    }
}
