use super::mailbox;
use crate::support::rpi;

pub fn init_memory(rpi_config: &rpi::RpiConfig) {}

pub fn get_arm_memory() -> (bool, u32, u32) {
  let buf = mailbox::get_buffer_mut();

  buf[0] = 9 * 4;
  buf[1] = mailbox::MBOX_REQUEST;

  buf[2] = mailbox::MBOX_TAG_GET_ARM_MEM;
  buf[3] = 8; // 8 bytes available for response.
  buf[4] = mailbox::MBOX_TAG_REQUEST;
  buf[5] = 0; // Reserved for base address
  buf[6] = 0; // Reserved for memory length

  buf[7] = mailbox::MBOX_TAG_LAST;
  buf[8] = 0;

  if !mailbox::send(mailbox::MBOX_CH_PROP) {
    return (false, 0, 0);
  }

  (true, buf[5], buf[6])
}
