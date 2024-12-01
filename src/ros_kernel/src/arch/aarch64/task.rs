struct TaskContext {
  x19: usize,
  x20: usize,
  x21: usize,
  x22: usize,
  x23: usize,
  x24: usize,
  x25: usize,
  x26: usize,
  x27: usize,
  x28: usize,
  x29: usize, // the frame pointer
  x30: usize, // the link register
  sp: usize,  // the stack pointer
}

pub struct Task {
  task_id: usize,
  context: TaskContext,
}

impl Task {
  pub const fn new(task_id: usize) -> Self {
    Task {
      task_id,
      context: TaskContext {
        x19: 0,
        x20: 0,
        x21: 0,
        x22: 0,
        x23: 0,
        x24: 0,
        x25: 0,
        x26: 0,
        x27: 0,
        x28: 0,
        x29: 0,
        x30: 0,
        sp: 0,
      },
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
  /// The kernel's page table is not modified by this function.
  ///
  /// # Returns
  ///
  /// The virtual address of the mapped page.
  pub fn map_page_local(&mut self, virtual_base: usize, page: usize) -> usize {
    virtual_base + page
  }

  /// Unmaps a page from the kernel's virtual address space.
  ///
  /// # Parameters
  ///
  /// * `task` - The kernel task receiving the mapping.
  ///
  /// # Description
  ///
  /// The kernel's page table is not modified by this function.
  pub fn unmap_page_local(&mut self) {}
}
