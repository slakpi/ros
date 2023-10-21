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
}
