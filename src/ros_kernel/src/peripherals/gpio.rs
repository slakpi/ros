use super::base;

// BCM2835, BCM2836, BCM2837, and BCM2711
pub const GPFSEL0: usize = 0x00200000;
pub const GPFSEL1: usize = 0x00200004;
pub const GPFSEL2: usize = 0x00200008;
pub const GPFSEL3: usize = 0x0020000C;
pub const GPFSEL4: usize = 0x00200010;
pub const GPFSEL5: usize = 0x00200014;
pub const GPSET0: usize = 0x0020001C;
pub const GPSET1: usize = 0x00200020;
pub const GPCLR0: usize = 0x00200028;
pub const GPCLR1: usize = 0x0020002C;
pub const GPLEV0: usize = 0x00200034;
pub const GPLEV1: usize = 0x00200038;
pub const GPEDS0: usize = 0x00200040;
pub const GPEDS1: usize = 0x00200044;
pub const GPREN0: usize = 0x0020004C;
pub const GPREN1: usize = 0x00200050;
pub const GPFEN0: usize = 0x00200058;
pub const GPFEN1: usize = 0x0020005C;
pub const GPHEN0: usize = 0x00200064;
pub const GPHEN1: usize = 0x00200068;
pub const GPLEN0: usize = 0x00200070;
pub const GPLEN1: usize = 0x00200074;
pub const GPAREN0: usize = 0x0020007C;
pub const GPAREN1: usize = 0x00200080;
pub const GPAFEN0: usize = 0x00200088;
pub const GPAFEN1: usize = 0x0020008C;
pub const GPPUD: usize = 0x00200094;
pub const GPPUDCLK0: usize = 0x00200098;
pub const GPPUDCLK1: usize = 0x0020009C;

// BCM2711
pub const GPIO_PUP_PDN_CNTRL_REG0: usize = 0x002000E4;
pub const GPIO_PUP_PDN_CNTRL_REG1: usize = 0x002000E8;
pub const GPIO_PUP_PDN_CNTRL_REG2: usize = 0x002000EC;
pub const GPIO_PUP_PDN_CNTRL_REG3: usize = 0x002000F0;

/// @enum GPIOPin
/// @brief GPIO pin identifiers.
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum GPIOPin {
  GPIO0,
  GPIO1,
  GPIO2,
  GPIO3,
  GPIO4,
  GPIO5,
  GPIO6,
  GPIO7,
  GPIO8,
  GPIO9,
  GPIO10,
  GPIO11,
  GPIO12,
  GPIO13,
  GPIO14,
  GPIO15,
  GPIO16,
  GPIO17,
  GPIO18,
  GPIO19,
  GPIO20,
  GPIO21,
  GPIO22,
  GPIO23,
  GPIO24,
  GPIO25,
  GPIO26,
  GPIO27,
  GPIO28,
  GPIO29,
  GPIO30,
  GPIO31,
  GPIO32,
  GPIO33,
  GPIO34,
  GPIO35,
  GPIO36,
  GPIO37,
  GPIO38,
  GPIO39,
  GPIO40,
  GPIO41,
  GPIO42,
  GPIO43,
  GPIO44,
  GPIO45,
  GPIO46,
  GPIO47,
  GPIO48,
  GPIO49,
  GPIO50,
  GPIO51,
  GPIO52,
  GPIO53,
  GPIO54,
  GPIO55,
  GPIO56,
  GPIO57,
}

/// @enum GPIOPinFunction
/// @brief GPIO pin functions. Refer to the BCM283x and BCM2711 datasheets for
///        the alternate function assignments.
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum GPIOPinFunction {
  Input = 0,
  Output = 1,
  AltFn0 = 4,
  AltFn1 = 5,
  AltFn2 = 6,
  AltFn3 = 7,
  AltFn4 = 3,
  AltFn5 = 2,
}

pub const GPIO_DELAY: u64 = 150;

/// @fn set_pin_function(pin: GPIOPin, func: GPIOPinFunction)
/// @brief Changes the function assignment for a GPIO pin.
/// @param[in] pin  The pin to change.
/// @param[in] func The new function assginment. 
pub fn set_pin_function(pin: GPIOPin, func: GPIOPinFunction) {
  let pin_val = pin as u8;
  let shift = (pin_val % 10) * 3;
  let reg = match pin_val / 10 {
    0 => GPFSEL0,
    1 => GPFSEL1,
    2 => GPFSEL2,
    3 => GPFSEL3,
    4 => GPFSEL4,
    5 => GPFSEL5,
    _ => {
      assert!(false, "Invalid GPIO register."); // Should never happen
      0
    }
  };

  let mut val = base::peripheral_reg_get(reg);
  val &= !(3 << shift);
  val |= (func as u32) << shift;
  base::peripheral_reg_put(val, reg);
}

/// @fn write_to_pin(pin: GPIOPin, val: bool)
/// @brief Write to a GPIO pin.
/// @param[in] pin The pin to write.
/// @param[in] val The truth value to write.
pub fn write_to_pin(pin: GPIOPin, val: bool) {
  let pin_val = pin as u8;
  let shift = pin_val % 32;
  let reg = match pin_val / 32 {
    0 => {
      if val {
        GPCLR0
      } else {
        GPSET0
      }
    }
    1 => {
      if val {
        GPCLR1
      } else {
        GPSET1
      }
    }
    _ => {
      assert!(false, "Invalid GPIO register."); // Should never happen
      0
    }
  };

  base::peripheral_reg_put(1 << shift, reg);
}
