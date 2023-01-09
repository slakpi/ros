use super::dtb;
use crate::dbg_print;
use crate::peripherals::mailbox;

pub struct RpiConfig {
  board_model: u32,
  board_revision: u32,
  peripheral_base: usize,
  page_size: u32,
  has_dtb: bool,
  dtb_base: usize,
  dtb_size: u32,
}

impl RpiConfig {
  pub fn new(peripheral_base: usize, blob: usize, page_size: u32) -> Self {
    let mut rpi_config = RpiConfig {
      board_model: 0,
      board_revision: 0,
      peripheral_base: peripheral_base,
      page_size: page_size,
      has_dtb: false,
      dtb_base: 0,
      dtb_size: 0,
    };

    (rpi_config.board_model, rpi_config.board_revision) = get_board();

    (rpi_config.has_dtb, rpi_config.dtb_size) = get_dtb(blob);

    if rpi_config.has_dtb {
      rpi_config.dtb_base = blob;
    }

    rpi_config
  }
}

fn get_board() -> (u32, u32) {
  let (ok, model) = get_board_model();

  if !ok {
    dbg_print!("Failed to get board model.\n");
    return (0, 0);
  }

  let (ok, rev) = get_board_revision();

  if !ok {
    dbg_print!("Failed to get board revision.\n");
    return (0, 0);
  }

  dbg_print!("Raspberry Pi Revision {:#x}\n", rev);

  (model, rev)
}


fn get_board_model() -> (bool, u32) {
  let buf = mailbox::get_buffer_mut();

  buf[0] = 8 * 4;
  buf[1] = mailbox::MBOX_REQUEST;

  buf[2] = mailbox::MBOX_TAG_GET_BOARD_MODEL;
  buf[3] = 4; // 4 bytes available for response.
  buf[4] = mailbox::MBOX_TAG_REQUEST;
  buf[5] = 0; // Reserved for board model

  buf[6] = mailbox::MBOX_TAG_LAST;
  buf[7] = 0;

  if !mailbox::send(mailbox::MBOX_CH_PROP) {
    return (false, 0);
  }

  (true, buf[5])
}

fn get_board_revision() -> (bool, u32) {
  let buf = mailbox::get_buffer_mut();

  buf[0] = 8 * 4;
  buf[1] = mailbox::MBOX_REQUEST;

  buf[2] = mailbox::MBOX_TAG_GET_BOARD_REV;
  buf[3] = 4; // 4 bytes available for response.
  buf[4] = mailbox::MBOX_TAG_REQUEST;
  buf[5] = 0; // Reserved for board revision

  buf[6] = mailbox::MBOX_TAG_LAST;
  buf[7] = 0;

  if !mailbox::send(mailbox::MBOX_CH_PROP) {
    return (false, 0);
  }

  (true, buf[5])
}

fn get_dtb(blob: usize) -> (bool, u32) {
  let (valid_dtb, size) = dtb::check_dtb(blob as *const u8);

  if !valid_dtb {
    dbg_print!("Invalid dtb.\n");
  } else {
    dbg_print!("Found valid dtb at {:#x} with size {:#x}\n", blob as usize, size);
  }

  (valid_dtb, size)
}
