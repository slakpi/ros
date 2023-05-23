//! Kernel Paging Interface

use super::page_allocator::PageAllocator;
use crate::arch;
use crate::arch::bits;
use crate::debug_print;
use crate::peripherals::memory;

/// We need to have at least as many allocators as we have memory ranges. The
/// allocator only works on contiguous blocks of memory.
const MAX_ALLOCATORS: usize = memory::MAX_MEM_RANGES;

/// A convenience initializer for the allocator array.
const INIT_ALLOCATOR: Option<PageAllocator> = None;

/// List of available page allocators.
static mut ALLOCATORS: [Option<PageAllocator>; MAX_ALLOCATORS] = [INIT_ALLOCATOR; MAX_ALLOCATORS];

/// Initializes the page allocators for the given memory layout.
///
/// # Description
///
/// Each allocator reserves enough memory at the end of its associated block for
/// its allocation flags. The required memory is page-aligned.
pub fn init() {
  let page_size = arch::get_page_size();
  let virtual_base = arch::get_kernel_virtual_base();
  let mem_layout = arch::get_memory_layout();

  for (i, r) in mem_layout.get_ranges().iter().enumerate() {
    let alloc_size = bits::align_up(PageAllocator::calc_size(page_size, r.size), page_size);
    assert!(alloc_size < r.size);

    let ptr = (r.base + virtual_base + r.size - alloc_size) as *mut u8;

    unsafe {
      ALLOCATORS[i] = Some(PageAllocator::new(page_size, r.base, r.size, ptr));
    }
  }
}
