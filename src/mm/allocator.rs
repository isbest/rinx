use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap(base: u64, size: usize) {
    unsafe {
        ALLOCATOR.lock().init(base as *mut u8, size);
    }
}
