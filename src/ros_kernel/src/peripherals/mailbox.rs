use super::base;

pub const MBOX_REQUEST: u32 = 0;

pub const _MBOX_CH_POWER: u32 = 0;
pub const _MBOX_CH_FB: u32 = 1;
pub const _MBOX_CH_VUART: u32 = 2;
pub const _MBOX_CH_VCHIQ: u32 = 3;
pub const _MBOX_CH_LEDS: u32 = 4;
pub const _MBOX_CH_BTNS: u32 = 5;
pub const _MBOX_CH_TOUCH: u32 = 6;
pub const _MBOX_CH_COUNT: u32 = 7;
pub const MBOX_CH_PROP: u32 = 8;

pub const _MBOX_TAG_REQUEST: u32 = 0;
pub const _MBOX_TAG_TEST: u32 = 4;
pub const MBOX_TAG_SET: u32 = 8;

pub const _MBOX_TAG_GET_BOARD_MODEL: u32 = 0x10001;
pub const _MBOX_TAG_GET_BOARD_REV: u32 = 0x10002;
pub const _MBOX_TAG_GET_ARM_MEM: u32 = 0x10005;

pub const _MBOX_TAG_SETPOWER: u32 = 0x28001;
pub const _MBOX_TAG_SETCLKRATE: u32 = 0x38002;

pub const MBOX_TAG_SETPHYWH: u32 = 0x48003;
pub const MBOX_TAG_SETVIRTWH: u32 = 0x48004;
pub const MBOX_TAG_SETVIRTOFF: u32 = 0x48009;
pub const MBOX_TAG_SETDEPTH: u32 = 0x48005;
pub const MBOX_TAG_SETPXLORDR: u32 = 0x48006;
pub const MBOX_TAG_GETFB: u32 = 0x40001;
pub const MBOX_TAG_GETPITCH: u32 = 0x40008;

pub const MBOX_TAG_LAST: u32 = 0;

const VIDEOCORE_MBOX: usize = 0x0000B880;

const MBOX_READ: usize = VIDEOCORE_MBOX;
const _MBOX_POLL: usize = VIDEOCORE_MBOX + 0x10;
const _MBOX_SENDER: usize = VIDEOCORE_MBOX + 0x14;
const MBOX_STATUS: usize = VIDEOCORE_MBOX + 0x18;
const _MBOX_CONFIG: usize = VIDEOCORE_MBOX + 0x1C;
const MBOX_WRITE: usize = VIDEOCORE_MBOX + 0x20;

const _MBOX_RESPONSE: u32 = 0x80000000;
const MBOX_FULL: u32 = 0x80000000;
const MBOX_EMPTY: u32 = 0x40000000;

pub const MAIL_SIZE: usize = 128;

pub type Mail = [u32; MAIL_SIZE];

/// @struct Align16
/// @brief  Dummy struct to force 16-byte alignment.
#[repr(align(16))]
struct _Align16;

/// @struct  MailWrapper
/// @brief   Alignment wrapper for the mail data.
/// @details The Mailbox peripheral requires the data pointer to be a 32-bit
///          pointer where the 28 most-significant bits are the address and the
///          4-bit channel is in the 4 least-significant bits. Thus, the array
///          must be aligned on a 16-byte boundary.
struct _MailWrapper {
  _alignment: [_Align16; 0],
  mail: Mail,
}

/// @var   MAIL
/// @brief Mail data.
static mut MAIL: _MailWrapper = _MailWrapper {
  _alignment: [],
  mail: [0; MAIL_SIZE],
};

/// @fn get_buffer
/// @brief Get a reference to the static mailbox message buffer.
pub fn get_buffer() -> &'static Mail {
  unsafe { &MAIL.mail }
}

/// @fn get_buffer_mut
/// @brief Get a mutable reference to the static mailbox message buffer.
pub fn get_buffer_mut() -> &'static mut Mail {
  unsafe { &mut MAIL.mail }
}

/// @fn send
/// @brief   Send a request to the GPU mailbox.
/// @param[in] channel The mailbox channel.
/// @returns True if the request succeeds.
pub fn send(channel: u32) -> bool {
  // Wait for the GPU to empty the mailbox.
  while (base::peripheral_reg_get(MBOX_STATUS) & MBOX_FULL) != 0 {}

  // Write the buffer address and channel to the mailbox.
  let packed_addr = pack_address_and_channel(channel);
  base::peripheral_reg_put(packed_addr, MBOX_WRITE);

  loop {
    // Wait for the GPU to deposit data into the mailbox.
    while (base::peripheral_reg_get(MBOX_STATUS) & MBOX_EMPTY) != 0 {}

    // Discard any responses not related to our request.
    if base::peripheral_reg_get(MBOX_READ) == packed_addr {
      return true;
    }
  }
}

/// @fn pack_address_and_channel
/// @brief   Packs the channel number into the buffer address for MBOX_WRITE.
/// @details The buffer address must by 16-byte aligned so that the least-
///          significant 4 bits are 0. The VideoCore mailbox expect to find the
///          channel number in those 4 bits.
/// @param[in] channel The channel to pack into the address.
/// @returns The 32-bit packed address and channel.
fn pack_address_and_channel(channel: u32) -> u32 {
  unsafe {
    let raw_ptr = &MAIL.mail as *const u32;
    let raw_addr = (raw_ptr as usize) & (0xfffffff0usize);
    (raw_addr as u32) | (channel & 0xf)
  }
}
