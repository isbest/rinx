use core::slice;
use x86::dtables::{DescriptorTablePointer, sgdt};

const GDT_SIZE: u16 = 128;

#[derive(Debug)]
#[repr(C, packed)]
pub struct Descriptor {
    value: u64,
}

impl Descriptor {
    pub fn new(value: u64) -> Self {
        Descriptor { value }
    }

    pub fn set_limit_low(&mut self, limit_low: u16) {
        self.value = (self.value & !0xFFFF) | u64::from(limit_low);
    }

    pub fn get_limit_low(&self) -> u16 {
        (self.value & 0xFFFF) as u16
    }

    pub fn set_base_low(&mut self, base_low: u16) {
        self.value = (self.value & !0xFFFFFF) | (u64::from(base_low) << 16);
    }

    pub fn get_base_low(&self) -> u16 {
        ((self.value >> 16) & 0xFFFFFF) as u16
    }

    pub fn set_type(&mut self, type_: u8) {
        self.value = (self.value & !(0xF << 40)) | (u64::from(type_) << 40);
    }

    pub fn get_type(&self) -> u8 {
        ((self.value >> 40) & 0xF) as u8
    }

    pub fn set_segment(&mut self, segment: bool) {
        if segment {
            self.value |= 1 << 44;
        } else {
            self.value &= !(1 << 44);
        }
    }

    pub fn get_segment(&self) -> bool {
        (self.value >> 44) & 1 == 1
    }

    pub fn set_dpl(&mut self, dpl: u8) {
        self.value = (self.value & !(0x3 << 45)) | (u64::from(dpl) << 45);
    }

    pub fn get_dpl(&self) -> u8 {
        ((self.value >> 45) & 0x3) as u8
    }

    pub fn set_present(&mut self, present: bool) {
        if present {
            self.value |= 1 << 47;
        } else {
            self.value &= !(1 << 47);
        }
    }

    pub fn get_present(&self) -> bool {
        (self.value >> 47) & 1 == 1
    }

    pub fn set_limit_high(&mut self, limit_high: u8) {
        self.value = (self.value & !(0xF << 48)) | (u64::from(limit_high) << 48);
    }

    pub fn get_limit_high(&self) -> u8 {
        ((self.value >> 48) & 0xF) as u8
    }

    pub fn set_available(&mut self, available: bool) {
        if available {
            self.value |= 1 << 52;
        } else {
            self.value &= !(1 << 52);
        }
    }

    pub fn get_available(&self) -> bool {
        (self.value >> 52) & 1 == 1
    }

    pub fn set_long_mode(&mut self, long_mode: bool) {
        if long_mode {
            self.value |= 1 << 53;
        } else {
            self.value &= !(1 << 53);
        }
    }

    pub fn get_long_mode(&self) -> bool {
        (self.value >> 53) & 1 == 1
    }

    pub fn set_big(&mut self, big: bool) {
        if big {
            self.value |= 1 << 54;
        } else {
            self.value &= !(1 << 54);
        }
    }

    pub fn get_big(&self) -> bool {
        (self.value >> 54) & 1 == 1
    }

    pub fn set_granularity(&mut self, granularity: bool) {
        if granularity {
            self.value |= 1 << 55;
        } else {
            self.value &= !(1 << 55);
        }
    }

    pub fn get_granularity(&self) -> bool {
        (self.value >> 55) & 1 == 1
    }

    pub fn set_base_high(&mut self, base_high: u8) {
        self.value = (self.value & !(0xFF << 56)) | (u64::from(base_high) << 56);
    }

    pub fn get_base_high(&self) -> u8 {
        ((self.value >> 56) & 0xFF) as u8
    }
}

#[no_mangle]
pub fn init_gdt() {
    let mut gdtr: DescriptorTablePointer<Descriptor> = Default::default();
    unsafe { sgdt(&mut gdtr); }

    let _: &[Descriptor] = unsafe {
        slice::from_raw_parts(gdtr.base as *const Descriptor, gdtr.limit as usize)
    };
    // todo
}