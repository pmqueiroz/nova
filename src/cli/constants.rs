macro_rules! nova_osc_code {
  () => {
    "777"
  };
}

pub const PRIVATE_NOVA_OSC_CODE_BYTES: &[u8] = nova_osc_code!().as_bytes();
pub const OSC_PREFIX: &[u8] = concat!("\x1b]", nova_osc_code!()).as_bytes();
pub const OSC_SUFFIX_BEL: &[u8] = b"\x07";
pub const ENV_IN_NOVA: &str = "NOVA_TERMINAL";
