//! ARMv7a Task Structure

/// ARMv7a CPU register context.
struct CpuContext {
  r4: usize,
  r5: usize,
  r6: usize,
  r7: usize,
  r8: usize,
  r10: usize,
  fp: usize, // r11, the frame pointer
  sp: usize, // r13, the stack pointer
  pc: usize, // r14, the link register
}

/// ARMv7a thread task.
/// TODO: Not sure if a fixed-size array as part of the task structure is going
///       to be the final form, but this is all hidden behind architecture
///       abstraction anyway. The mechanics of using the Level 2 page table as
///       a local mapping stack won't change.
pub struct Task {
  task_id: usize,
  cpu_context: CpuContext,
  local_mappings: [usize; 1024],
  local_map_count: usize,
}

impl Task {
  pub const fn new(task_id: usize) -> Self {
    Task {
      task_id,
      cpu_context: CpuContext {
        r4: 0,
        r5: 0,
        r6: 0,
        r7: 0,
        r8: 0,
        r10: 0,
        fp: 0,
        sp: 0,
        pc: 0,
      },
      local_mappings: [0; 1024],
      local_map_count: 0,
    }
  }

  /// Maps a page into the kernel's virtual address space.
  ///
  /// # Parameters
  ///
  /// * `task` - The kernel task receiving the mapping.
  /// * `virtual_base` - The kernel segment base address.
  /// * `page` - The physical address of the page to map.
  ///
  /// # Description
  ///
  /// If the page is in low memory, the function simply returns the virtual
  /// address of the mapped page without modifying the kernel's page table.
  ///
  /// Otherwise, the function maps the page to the next available virtual
  /// address in the task's local mappings. The mappings are thread-local, so
  /// the function is thread safe.
  ///
  ///   NOTE: The Linux implementation ensures the thread is pinned to the same
  ///         CPU for the duration of temporary mappings.
  ///
  /// The function will panic if no more pages can be mapped into the thread's
  /// local mappings.
  ///
  /// # Returns
  ///
  /// The virtual address of the mapped page.
  pub fn map_page_local(&mut self, virtual_base: usize, page: usize) -> usize {
    debug_assert!(false);
    0
  }

  /// Unmaps a page from the kernel's virtual address space.
  ///
  /// # Parameters
  ///
  /// * `task` - The kernel task receiving the mapping.
  ///
  /// # Description
  ///
  /// If the page is in low memory or if no pages have been mapped into the
  /// thread's local mappings, the function simply returns without modifying
  /// the kernel's page table.
  ///
  /// Otherwise, the function unmaps the page from the task's local mappings.
  /// The mappings are thread-local, so the function is thread safe.
  pub fn unmap_page_local(&mut self) {
    debug_assert!(false);
  }
}
