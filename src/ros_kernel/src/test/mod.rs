pub struct TestContext {
  pass_count: u32,
  fail_count: u32,
}

impl TestContext {
  pub fn new() -> Self {
    TestContext {
      pass_count: 0,
      fail_count: 0,
    }
  }

  pub fn log_pass(&mut self) {
    self.pass_count += 1;
  }

  pub fn log_fail(&mut self) {
    self.fail_count += 1;
  }

  pub fn get_pass_count(&self) -> u32 {
    self.pass_count
  }

  pub fn get_fail_count(&self) -> u32 {
    self.fail_count
  }
}

#[macro_export]
macro_rules! execute_test {
  ($fn:ident) => {
    let mut context = crate::test::TestContext::new();
    $fn(&mut context);
    debug_print!(
      "{}: Pass: {}, Fail: {}\n",
      stringify!($fn),
      context.get_pass_count(),
      context.get_fail_count(),
    );
  };
}

#[macro_export]
macro_rules! check_eq {
  ($c:ident, $a:expr, $b:expr) => {
    if $a != $b {
      $c.log_fail();
      debug_print!("FAIL {} {}: {} != {}\n", file!(), line!(), $a, $b);
    } else {
      $c.log_pass();
    }
  };
}
