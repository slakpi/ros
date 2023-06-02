use super::PageAllocator;

pub fn run_tests() {
  test_allocator_construction();
}

fn test_allocator_construction() {
  // Block Size (Pages)       Bytes Required      Valid Bits
  // -------------------------------------------------------
  // 1024                       1                 1
  //  512                       1                 2
  //  256                       1                 4
  //  128                       1                 *
  //   64                       2                 *
  //   32                       4                 *
  //   16                       8                 *
  //    8                      16                 *
  //    4                      32                 *
  //    2                      64                 *
  //    1                     128                 *
  // -------------------------------------------------------
  //                          257 bytes total for metadata
  let page_size = 4096usize;
  let block_size = page_size * 1024;
  let exp_size = 257usize;

  let size = PageAllocator::calc_size(page_size, block_size);
  debug_assert!(size == exp_size, "Calculated size was {} instead of {}.", size, exp_size);
}