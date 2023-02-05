use crate::peripherals::mailbox;
use core::ptr;

const DEFAULT_WIDTH: u32 = 1920;
const DEFAULT_HEIGHT: u32 = 1080;

/// @struct Framebuffer
/// @brief  Defines the properties of a framebuffer.
pub struct Framebuffer {
  pub fb_ptr: *mut u8,
  pub width: u32,
  pub height: u32,
  pub pitch: u32,
  pub _isrgb: u32,
}

/// @var   FRAMEBUFFER
/// @brief The default framebuffer. The kernel is single-threaded, so directly
///        accessing the value is safe.
static mut FRAMEBUFFER: Framebuffer = Framebuffer {
  fb_ptr: ptr::null_mut(),
  width: 0,
  height: 0,
  pitch: 0,
  _isrgb: 0,
};

/// @fn fb_init
/// @brief Initialize the default framebuffer.
pub fn fb_init() {
  // Setup the framebuffer properties and send them to the VideoCore.
  configure_properties();

  // Send the request and get the response from the VideoCore.
  if mailbox::send(mailbox::MBOX_CH_PROP) {
    configure_fb(mailbox::get_buffer());
  }
}

/// @fn get_fb
/// @brief Get a reference to the framebuffer.
pub fn get_fb() -> &'static Framebuffer {
  unsafe {
    debug_assert!(FRAMEBUFFER.fb_ptr.is_null());
    &FRAMEBUFFER
  }
}

/// @fn configure_properties
/// @brief Configure a mailbox message to request a framebuffer from the
///        VideoCore.
fn configure_properties() {
  let buf = mailbox::get_buffer_mut();

  buf[0] = 36 * 4;
  buf[1] = mailbox::MBOX_REQUEST;

  buf[2] = mailbox::MBOX_TAG_SETPHYWH;
  buf[3] = 8;
  buf[4] = mailbox::MBOX_TAG_SET;
  buf[5] = DEFAULT_WIDTH;
  buf[6] = DEFAULT_HEIGHT;

  buf[7] = mailbox::MBOX_TAG_SETVIRTWH;
  buf[8] = 8;
  buf[9] = mailbox::MBOX_TAG_SET;
  buf[10] = DEFAULT_WIDTH;
  buf[11] = DEFAULT_HEIGHT;

  buf[12] = mailbox::MBOX_TAG_SETVIRTOFF;
  buf[13] = 8;
  buf[14] = mailbox::MBOX_TAG_SET;
  buf[15] = 0;
  buf[16] = 0;

  buf[17] = mailbox::MBOX_TAG_SETDEPTH;
  buf[18] = 4;
  buf[19] = mailbox::MBOX_TAG_SET;
  buf[20] = 32;

  buf[21] = mailbox::MBOX_TAG_SETPXLORDR;
  buf[22] = 4;
  buf[23] = mailbox::MBOX_TAG_SET;
  buf[24] = 1;

  buf[25] = mailbox::MBOX_TAG_GETFB;
  buf[26] = 8;
  buf[27] = mailbox::MBOX_TAG_SET;
  buf[28] = 4096;
  buf[29] = 0;

  buf[30] = mailbox::MBOX_TAG_GETPITCH;
  buf[31] = 4;
  buf[32] = mailbox::MBOX_TAG_SET;
  buf[33] = 0;

  buf[34] = mailbox::MBOX_TAG_LAST;
  buf[35] = 0;
}

/// @fn configure_fb
/// @brief Configure the default framebuffer from the VideoCore response.
fn configure_fb(buf: &'static mailbox::Mail) {
  // Verify we have a buffer with 32-bit depth and a pointer. Convert the
  // pointer into an ARM pointer.
  if buf[20] != 32 || buf[28] == 0 {
    return;
  }

  unsafe {
    FRAMEBUFFER = Framebuffer {
      fb_ptr: (buf[28] & 0x3FFFFFFF) as *mut u8,
      width: buf[10],
      height: buf[11],
      pitch: buf[33],
      _isrgb: buf[24],
    };
  }
}
