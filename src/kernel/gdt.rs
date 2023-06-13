use crate::kernel::{len_of_limit, limit_of_type};
use core::slice;
use lazy_static::lazy_static;
use log::{debug, info};
use spin::Mutex;
use x86::dtables::{lgdt, sgdt, DescriptorTablePointer};
use x86::segmentation::Descriptor;

const GDT_SIZE: usize = 128;

// 内核全局描述符
lazy_static! {
    static ref GDT: Mutex<[Descriptor; GDT_SIZE]> = {
        #[allow(unused_mut)]
        let mut gdt = Mutex::new([Descriptor::default(); GDT_SIZE]);
        gdt
    };
}

#[no_mangle]
pub fn init_gdt() {
    let mut gdtr: DescriptorTablePointer<Descriptor> = Default::default();
    unsafe {
        sgdt(&mut gdtr);
    }

    debug!("loader gdt len: {}", len_of_limit::<Descriptor>(gdtr.limit));
    let gdt: &[Descriptor] = unsafe {
        slice::from_raw_parts(
            gdtr.base as *mut Descriptor,
            len_of_limit::<Descriptor>(gdtr.limit),
        )
    };

    // 拷贝到内核
    GDT.lock()[..gdt.len()].clone_from_slice(gdt);

    gdtr.base = GDT.lock().as_ptr();
    gdtr.limit = limit_of_type::<[Descriptor; GDT_SIZE]>();

    info!("kernel gdt len: {}", GDT.lock().len());
    info!("kernel gdt base: {:p}", { gdtr.base });

    unsafe {
        lgdt(&gdtr);
    }
}
