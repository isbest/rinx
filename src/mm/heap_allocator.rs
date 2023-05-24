use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

const KERNEL_HEAP_SIZE: usize = 1024 * 1024 * 4;

static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

pub fn init_heap() {
    unsafe {
        ALLOCATOR
            .lock()
            .init(0x20000 as *mut u8, KERNEL_HEAP_SIZE);
    }
}