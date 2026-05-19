use base64::Engine;

#[derive(Default)]
pub enum ApcState {
  #[default]
  None,
  SawEsc,
  Collecting(Vec<u8>),
  CollectingSawEsc(Vec<u8>),
}

impl ApcState {
  pub fn advance(&mut self, byte: u8) -> (Vec<u8>, Option<Vec<u8>>) {
    match self {
      ApcState::None => {
        if byte == 0x1b {
          *self = ApcState::SawEsc;
          (vec![], None)
        } else {
          (vec![byte], None)
        }
      }
      ApcState::SawEsc => {
        if byte == b'_' {
          *self = ApcState::Collecting(Vec::new());
          (vec![], None)
        } else {
          let pass = vec![0x1b, byte];
          *self = ApcState::None;
          (pass, None)
        }
      }
      ApcState::Collecting(buf) => {
        if byte == 0x1b {
          *self = ApcState::CollectingSawEsc(std::mem::take(buf));
          (vec![], None)
        } else if byte == 0x9c {
          let content = std::mem::take(buf);
          *self = ApcState::None;
          (vec![], Some(content))
        } else {
          buf.push(byte);
          (vec![], None)
        }
      }
      ApcState::CollectingSawEsc(buf) => {
        if byte == b'\\' {
          let content = std::mem::take(buf);
          *self = ApcState::None;
          (vec![], Some(content))
        } else if byte == 0x1b {
          buf.push(0x1b);
          (vec![], None)
        } else {
          let mut new_buf = std::mem::take(buf);
          new_buf.push(0x1b);
          new_buf.push(byte);
          *self = ApcState::Collecting(new_buf);
          (vec![], None)
        }
      }
    }
  }
}

pub struct PendingKittyImage {
  pub format: u32,
  pub width: u32,
  pub height: u32,
  pub id: u32,
  pub chunks: Vec<Vec<u8>>,
}

pub struct KittyApcCommand {
  pub action: u8,
  pub format: u32,
  pub width: u32,
  pub height: u32,
  pub id: u32,
  pub more: bool,
  pub quiet: u8,
  pub data: Vec<u8>,
}

pub fn parse_kitty_apc(content: &[u8]) -> Option<KittyApcCommand> {
  if content.first() != Some(&b'G') {
    return None;
  }
  let rest = &content[1..];

  let (keys_bytes, data_bytes) = match rest.iter().position(|&b| b == b';') {
    Some(sep) => (&rest[..sep], &rest[sep + 1..]),
    None => (rest, &[] as &[u8]),
  };

  let keys_str = std::str::from_utf8(keys_bytes).ok()?;

  let mut action = b'T';
  let mut format = 32u32;
  let mut width = 0u32;
  let mut height = 0u32;
  let mut id = 0u32;
  let mut more = false;
  let mut quiet = 0u8;
  let mut medium = b'd';

  for kv in keys_str.split(',') {
    let mut parts = kv.splitn(2, '=');
    let key = match parts.next() {
      Some(k) => k.trim(),
      None => continue,
    };
    let val = parts.next().unwrap_or("").trim();
    match key {
      "a" => action = val.bytes().next().unwrap_or(b'T'),
      "f" => format = val.parse().unwrap_or(32),
      "s" => width = val.parse().unwrap_or(0),
      "v" => height = val.parse().unwrap_or(0),
      "i" => id = val.parse().unwrap_or(0),
      "m" => more = val == "1",
      "q" => quiet = val.parse().unwrap_or(0),
      "t" => medium = val.bytes().next().unwrap_or(b'd'),
      _ => {}
    }
  }

  if medium != b'd' {
    return None;
  }

  Some(KittyApcCommand {
    action,
    format,
    width,
    height,
    id,
    more,
    quiet,
    data: data_bytes.to_vec(),
  })
}

pub fn decode_kitty_image(
  prior_chunks: &[Vec<u8>],
  final_data: &[u8],
  format: u32,
  width: u32,
  height: u32,
) -> Option<(Vec<u8>, u32, u32)> {
  let mut combined = Vec::new();
  for chunk in prior_chunks {
    combined.extend_from_slice(chunk);
  }
  combined.extend_from_slice(final_data);

  let raw = base64::engine::general_purpose::STANDARD
    .decode(&combined)
    .ok()
    .or_else(|| {
      base64::engine::general_purpose::STANDARD_NO_PAD
        .decode(&combined)
        .ok()
    })?;

  match format {
    32 => {
      if width == 0 || height == 0 || raw.len() != (width * height * 4) as usize {
        return None;
      }
      Some((raw, width, height))
    }
    24 => {
      if width == 0 || height == 0 || raw.len() != (width * height * 3) as usize {
        return None;
      }
      let rgba: Vec<u8> = raw
        .chunks_exact(3)
        .flat_map(|p| [p[0], p[1], p[2], 255u8])
        .collect();
      Some((rgba, width, height))
    }
    100 => {
      let img = image::load_from_memory(&raw).ok()?.to_rgba8();
      let w = img.width();
      let h = img.height();
      Some((img.into_raw(), w, h))
    }
    _ => None,
  }
}
