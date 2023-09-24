use crate::arch::task;

static mut KERNEL_TASK: task::Task = task::Task::new(0);

pub fn get_kernel_task() -> &'static mut task::Task {
  unsafe { &mut KERNEL_TASK }
}
