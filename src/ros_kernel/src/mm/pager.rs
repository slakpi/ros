//! Kernel Memory Pager

use super::buddy_allocator::BuddyPageAllocator;
use crate::arch::{self, memory};
use crate::support::{bits, range};
use crate::sync;
use core::ptr;

/// We need to have at least as many allocators as we have memory ranges. The
/// allocator only works on contiguous blocks of memory.
const MAX_ALLOCATORS: usize = memory::MAX_MEM_RANGES;

/// A convenience initializer for the allocator array.
const INIT_ALLOCATOR: Option<sync::SpinLock<BuddyPageAllocator>> = None;

/// List of available page allocators.
///
///   TODO: This is pretty inefficient. There will be lock contention for every
///         allocation / free. If the zones are split among the cores,
///         contention can be reduced to situations where a core runs out of
///         pages in its block.
static mut ALLOCATORS: [Option<sync::SpinLock<BuddyPageAllocator>>; MAX_ALLOCATORS] =
  [INIT_ALLOCATOR; MAX_ALLOCATORS];

/// Initializes the page allocators for the given memory layout.
///
/// # Description
///
/// Each allocator reserves enough memory at the end of its associated block for
/// its allocation flags. The required memory is page-aligned.
///
///   TODO: We're going to need separate zones for DMA.
pub fn init() {
  let page_size = arch::get_page_size();
  let virtual_base = arch::get_kernel_virtual_base();
  let mem_layout = arch::get_memory_layout();

  for (i, r) in mem_layout.get_ranges().iter().enumerate() {
    // If the memory area is not large enough for even a single page, skip it.
    if r.size < page_size {
      continue;
    }

    // Calculate the amount of memory needed for the allocator's metadata and
    // verify the memory area is large enough. If not, skip it.
    let size = BuddyPageAllocator::calc_metadata_size(r.size);
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

    avail.exclude_range(&meta_range);

    if let Some(allocator) = BuddyPageAllocator::new(r.base, r.size, ptr, &avail) {
      unsafe {
        ALLOCATORS[i] = Some(sync::SpinLock::new(allocator));
      }
    }
  }
}

/// Allocate a contiguous block of physical pages.
///
/// # Parameters
///
/// * `pages` - The number of pages to allocate.
///
/// # Description
///
/// The block will come from the first available zone that has available pages.
///
///   TODO: This needs to be more sophisticated to allow for allocating pages
///         for DMA.
///
/// # Returns
///
/// A tuple with the physical base address of the allocation, the actual number
/// of pages allocated, and the zone from which the pages originated. None if a
/// block of the required size could not be allocated.
pub fn allocate(pages: usize) -> Option<(usize, usize, usize)> {
  unsafe {
    for (zone, lock) in ptr::addr_of!(ALLOCATORS)
      .as_ref()
      .unwrap()
      .iter()
      .enumerate()
    {
      if let Some(lock) = lock {
        let mut allocator = lock.lock();
        if let Some((base, pages)) = allocator.allocate(pages) {
          return Some((base, pages, zone));
        }
      }
    }
  }

  None
}

/// Free a contiguous block of physical pages.
///
/// # Parameters
///
/// * `base` - The physical base address of the block of pages.
/// * `pages` - The number of pages to free.
/// * `zone` - The zone from which the pages originated.
pub fn free(base: usize, pages: usize, zone: usize) {
  assert!(zone < MAX_ALLOCATORS);

  unsafe {
    let lock = ALLOCATORS[zone].as_ref().unwrap();
    let mut allocator = lock.lock();
    allocator.free(base, pages);
  }
}
