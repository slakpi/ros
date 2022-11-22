use super::base;

pub fn put(val: i32, to: usize) {
  let addr = base::get_peripheral_register_addr(to);
  unsafe {
    *addr = val;
  }
}

pub fn get(from: usize) -> i32 {
  let addr = base::get_peripheral_register_addr(from);
  unsafe { *addr }
}

pub fn delay(count: u64) {
  let mut c = count;
  while c > 0 {
    c -= 1;
  }
}
