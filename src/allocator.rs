pub struct SimpleAllocator {
    start: usize,
    size: usize,
}

impl SimpleAllocator {
    pub const fn new(start: usize, size: usize) -> Self {
        SimpleAllocator { start, size }
    }

    pub fn alloc(&self, _size: usize) -> *mut u8 {
        self.start as *mut u8
    }
}

pub static ALLOCATOR: SimpleAllocator = SimpleAllocator::new(0x100000, 0x10000);
