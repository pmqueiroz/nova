use iced::Color;
use std::sync::OnceLock;

pub struct RuntimeTheme {
  pub background: Color,
  pub foreground: Color,
  pub accent: Color,
  pub foreground_muted: Color,
  pub border: Color,
  pub cursor: Color,
}

static RUNTIME: OnceLock<std::sync::Mutex<RuntimeTheme>> = OnceLock::new();

pub fn init_runtime(t: RuntimeTheme) {
  RUNTIME.set(std::sync::Mutex::new(t)).ok();
}

pub fn runtime() -> std::sync::MutexGuard<'static, RuntimeTheme> {
  RUNTIME
    .get_or_init(|| {
      std::sync::Mutex::new(RuntimeTheme {
        background: BG.as_color(),
        foreground: FG.as_color(),
        accent: ACCENT.as_color(),
        foreground_muted: FG_MUTED.as_color(),
        border: BORDER.as_color(),
        cursor: ACCENT.as_color(),
      })
    })
    .lock()
    .unwrap()
}

pub fn update_runtime(t: RuntimeTheme) {
  if let Some(m) = RUNTIME.get() {
    *m.lock().unwrap() = t;
  }
}
pub const BG: Hue = Hue {
  r: 0x22,
  g: 0x22,
  b: 0x22,
  a: 1.0,
};
pub const BG_DEEP: Hue = Hue {
  r: 0x14,
  g: 0x14,
  b: 0x14,
  a: 1.0,
};
pub const BG_HIGH: Hue = Hue {
  r: 0xFF,
  g: 0xFF,
  b: 0xFF,
  a: 0.05,
};
pub const ACCENT: Hue = Hue {
  r: 0x3E,
  g: 0xCF,
  b: 0x8E,
  a: 1.0,
};
pub const ACCENT_DIM: Hue = Hue {
  r: 0x3E,
  g: 0xCF,
  b: 0x8E,
  a: 0.75,
};
pub const RED: Hue = Hue {
  r: 0xFF,
  g: 0x5F,
  b: 0x57,
  a: 1.0,
};
pub const FG: Hue = Hue {
  r: 0xE5,
  g: 0xE5,
  b: 0xE5,
  a: 1.0,
};
pub const FG_MUTED: Hue = Hue {
  r: 0x66,
  g: 0x66,
  b: 0x66,
  a: 1.0,
};
pub const BORDER: Hue = Hue {
  r: 0xFF,
  g: 0xFF,
  b: 0xFF,
  a: 0.07,
};
pub const TRANSPARENT: Hue = Hue {
  r: 0x00,
  g: 0x00,
  b: 0x00,
  a: 0.0,
};

pub const TRAFFIC_LIGHT_RED: Hue = Hue {
  r: 0xFF,
  g: 0x60,
  b: 0x5C,
  a: 1.0,
};
pub const TRAFFIC_LIGHT_YELLOW: Hue = Hue {
  r: 0xFF,
  g: 0xBD,
  b: 0x44,
  a: 1.0,
};
pub const TRAFFIC_LIGHT_GREEN: Hue = Hue {
  r: 0x00,
  g: 0xCA,
  b: 0x4E,
  a: 1.0,
};
pub const TRAFFIC_LIGHT_INACTIVE: Hue = Hue {
  r: 0x4C,
  g: 0x4C,
  b: 0x4C,
  a: 1.0,
};

pub struct Hue {
  pub r: u8,
  pub g: u8,
  pub b: u8,
  pub a: f32,
}

impl Hue {
  pub fn as_color(&self) -> Color {
    Color {
      r: self.r as f32 / 255.0,
      g: self.g as f32 / 255.0,
      b: self.b as f32 / 255.0,
      a: self.a,
    }
  }
}
