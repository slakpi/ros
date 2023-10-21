//! Task Support
//! TODO: This is all temporary scaffolding to support abstracting page
//!       allocation. This will be redone to support real thread tasks and
//!       processes.

use crate::arch::task;

static mut KERNEL_TASK: task::Task = task::Task::new(0);

pub fn get_kernel_task() -> &'static mut task::Task {
  unsafe { &mut KERNEL_TASK }
}
