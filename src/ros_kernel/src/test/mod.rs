//! Basic Low-Level Module Testing Utilities

pub struct TestContext {
  pub pass_count: u32,
  pub fail_count: u32,
}

impl TestContext {
  pub fn new() -> Self {
    TestContext {
      pass_count: 0,
      fail_count: 0,
    }
  }
}

#[macro_export]
macro_rules! execute_test {
  ($fn:ident) => {
    let mut context = crate::test::TestContext::new();
    debug_print!("* {}:\n", stringify!($fn));
    $fn(&mut context);
    debug_print!(
      "  Pass: {}, Fail: {}\n",
      context.pass_count,
      context.fail_count
    );
  };
}

#[macro_export]
macro_rules! check_eq {
  ($ctx:ident, $act:expr, $exp:expr) => {
    if $act != $exp {
      $ctx.fail_count += 1;
      debug_print!("    FAIL: {} != {} ({} {})\n", $act, $exp, file!(), line!());
    } else {
      $ctx.pass_count += 1;
    }
  };
  ($ctx:ident, $act:expr, $exp:expr, $tag:expr) => {
    if $act != $exp {
      $ctx.fail_count += 1;
      debug_print!(
        "    FAIL: {} != {} ({} {} {})\n",
        $act,
        $exp,
        $tag,
        file!(),
        line!()
      );
    } else {
      $ctx.pass_count += 1;
    }
  };
}
