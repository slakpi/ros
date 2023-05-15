pub mod page_allocator;

use page_allocator::PageAllocator;
use crate::peripherals::memory;

const INIT_ALLOCATOR: Option<PageAllocator> = None;
const ALLOCATOR_COUNT: usize = memory::get_max_memory_ranges();

static mut ALLOCATORS: [Option<PageAllocator>; ALLOCATOR_COUNT] = [INIT_ALLOCATOR; ALLOCATOR_COUNT];
