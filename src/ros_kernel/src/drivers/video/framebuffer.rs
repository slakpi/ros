use crate::dbg_print;
use crate::peripherals::mailbox;
use core::ptr;

const DEFAULT_WIDTH: u32 = 1920;
const DEFAULT_HEIGHT: u32 = 1080;

pub struct Framebuffer {
  pub fb_ptr: *mut u8,
  pub width: u32,
  pub height: u32,
  pub pitch: u32,
  pub _isrgb: u32,
}

static mut FRAMEBUFFER: Framebuffer = Framebuffer {
  fb_ptr: ptr::null_mut(),
  width: 0,
  height: 0,
  pitch: 0,
  _isrgb: 0,
};

pub fn fb_init() {
  // Setup the framebuffer properties and send them to the VideoCore.
  configure_properties();

  // Send the request and get the response from the VideoCore.
  if mailbox::send(mailbox::MBOX_CH_PROP) {
    let buf = mailbox::get_buffer();

    // Verify we have a buffer with 32-bit depth and a pointer. Conver the
    // pointer into an ARM pointer.
    unsafe {
      if buf[20] == 32 && buf[28] != 0 {
        FRAMEBUFFER = Framebuffer {
          fb_ptr: (buf[28] & 0x3FFFFFFF) as *mut u8,
          width: buf[10],
          height: buf[11],
          pitch: buf[33],
          _isrgb: buf[24],
        };

        dbg_print!(
          "Initialized framebuffer at {}x{}.\n",
          FRAMEBUFFER.width,
          FRAMEBUFFER.height
        );
      } else {
        dbg_print!("Failed to initialize framebuffer.\n");
      }
    }
  }
}

pub fn get_fb() -> &'static Framebuffer {
  unsafe {
    assert!(FRAMEBUFFER.fb_ptr != ptr::null_mut());
    &FRAMEBUFFER
  }
}

fn configure_properties() {
  let buf = mailbox::get_buffer_mut();

  buf[0] = 36 * 4;
  buf[1] = mailbox::MBOX_REQUEST;

  buf[2] = mailbox::MBOX_TAG_SETPHYWH;
  buf[3] = 8;
  buf[4] = 8;
  buf[5] = DEFAULT_WIDTH;
  buf[6] = DEFAULT_HEIGHT;

  buf[7] = mailbox::MBOX_TAG_SETVIRTWH;
  buf[8] = 8;
  buf[9] = 8;
  buf[10] = DEFAULT_WIDTH;
  buf[11] = DEFAULT_HEIGHT;

  buf[12] = mailbox::MBOX_TAG_SETVIRTOFF;
  buf[13] = 8;
  buf[14] = 8;
  buf[15] = 0;
  buf[16] = 0;

  buf[17] = mailbox::MBOX_TAG_SETDEPTH;
  buf[18] = 4;
  buf[19] = 4;
  buf[20] = 32;

  buf[21] = mailbox::MBOX_TAG_SETPXLORDR;
  buf[22] = 4;
  buf[23] = 4;
  buf[24] = 1;

  buf[25] = mailbox::MBOX_TAG_GETFB;
  buf[26] = 8;
  buf[27] = 8;
  buf[28] = 4096;
  buf[29] = 0;

  buf[30] = mailbox::MBOX_TAG_GETPITCH;
  buf[31] = 4;
  buf[32] = 4;
  buf[33] = 0;

  buf[34] = mailbox::MBOX_TAG_LAST;
  buf[35] = 0;
}
