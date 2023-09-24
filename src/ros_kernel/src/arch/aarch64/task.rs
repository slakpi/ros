struct CpuContext {
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
  fp: usize, // x29, the frame pointer
  pc: usize, // x30, the link register
  sp: usize, // x31, the stack pointer
}

pub struct Task {
  task_id: usize,
  cpu_context: CpuContext,
}

impl Task {
  pub const fn new(task_id: usize) -> Self {
    Task {
      task_id,
      cpu_context: CpuContext {
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
        fp: 0,
        pc: 0,
        sp: 0,
      },
    }
  }
}
