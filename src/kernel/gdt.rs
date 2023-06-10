use core::arch::asm;
use crate::println;
use core::slice;
use lazy_static::lazy_static;
use spin::Mutex;
use x86::dtables::{sgdt, DescriptorTablePointer, lgdt};
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
    unsafe { asm!("xchg bx,bx") };
    unsafe {
        sgdt(&mut gdtr);
    }

    let gdt: &[Descriptor] =
        unsafe { slice::from_raw_parts(gdtr.base as *const Descriptor, 3) };

    println!("gdt len:{}", gdt.len());

    // 拷贝到内核
    GDT.lock()[..gdt.len()].clone_from_slice(gdt);

    gdtr.base = GDT.lock().as_ptr();
    gdtr.limit = GDT_SIZE as u16;

    unsafe {
        lgdt(&gdtr)
    }
}
