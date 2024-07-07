//! Kernel Memory Pager

use super::page_allocator::PageAllocator;
use crate::arch::{self, memory};
use crate::support::{bits, range};

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
  let excl_layout = arch::get_exclusion_layout();

  for (i, r) in mem_layout.get_ranges().iter().enumerate() {
    // If the memory area is not large enough for even a single page, skip it.
    if r.size < page_size {
      continue;
    }

    // Calculate the amount of memory needed for the allocator's metadata and
    // verify the memory area is large enough. If not, skip it.
    let size = PageAllocator::calc_metadata_size(r.size);
    let alloc_size = bits::align_up(size, page_size);
    if r.size - page_size < alloc_size {
      continue;
    }

    // Setup a range set based on the current memory area with exclusion regions
    // removed. In addition to the system exclusion regions, exclude the
    // allocator's metadata region from the end of the memory area.
    let meta_range = range::Range {
      base: r.base + r.size - alloc_size,
      size: alloc_size,
    };

    let ptr = (meta_range.base + virtual_base) as *mut u8;

    let mut avail = memory::MemoryConfig::new();
    if !avail.insert_range(*r) {
      continue;
    }

    for e in excl_layout.get_ranges() {
      avail.exclude_range(e);
    }

    avail.exclude_range(&meta_range);

    // Create the allocator.
    unsafe {
      ALLOCATORS[i] = PageAllocator::new(r.base, r.size, ptr, &avail);
    }
  }
}
