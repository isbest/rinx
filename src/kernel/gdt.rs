use core::intrinsics::size_of;
use core::slice;
use lazy_static::lazy_static;
use log::{debug, info};
use spin::Mutex;
use x86::dtables::{lgdt, sgdt, DescriptorTablePointer};
use x86::segmentation::Descriptor;

const GDT_SIZE: u16 = 128;
const GDT_STRUCT_SIZE: usize = size_of::<Descriptor>();

// 内核全局描述符
lazy_static! {
    static ref GDT: Mutex<[Descriptor; GDT_SIZE as usize]> = {
        #[allow(unused_mut)]
        let mut gdt = Mutex::new([Descriptor::default(); GDT_SIZE as usize]);
        gdt
    };
}

#[no_mangle]
pub fn init_gdt() {
    let mut gdtr: DescriptorTablePointer<Descriptor> = Default::default();
    unsafe {
        sgdt(&mut gdtr);
    }

    debug!("loader gdt len: {}", get_len(gdtr.limit));
    debug!("loader gdt base: {:p}", { gdtr.base });
    let gdt: &[Descriptor] =
        unsafe { slice::from_raw_parts(gdtr.base as *mut Descriptor, get_len(gdtr.limit)) };

    // 拷贝到内核
    GDT.lock()[..gdt.len()].clone_from_slice(gdt);

    gdtr.base = GDT.lock().as_ptr();
    gdtr.limit = get_limit(GDT.lock().len());

    info!("kernel gdt len: {}", GDT.lock().len());
    info!("kernel gdt base: {:p}", { gdtr.base });

    unsafe {
        lgdt(&gdtr);
    }
}

// get GDT limit from GDT slice
fn get_limit(len: usize) -> u16 {
    (len * GDT_STRUCT_SIZE - 1) as u16
}

// get GDT length form gdt_ptr.limit
fn get_len(limit: u16) -> usize {
    (limit + 1) as usize / GDT_STRUCT_SIZE
}
