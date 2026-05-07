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

static RUNTIME: OnceLock<RuntimeTheme> = OnceLock::new();

pub fn init_runtime(t: RuntimeTheme) {
  RUNTIME.set(t).ok();
}

pub fn runtime() -> &'static RuntimeTheme {
  RUNTIME.get_or_init(|| RuntimeTheme {
    background: BG.as_color(),
    foreground: FG.as_color(),
    accent: ACCENT.as_color(),
    foreground_muted: FG_MUTED.as_color(),
    border: BORDER.as_color(),
    cursor: ACCENT.as_color(),
  })
}

pub const BG: Hue = Hue {
  r: 0x0D,
  g: 0x0D,
  b: 0x0D,
  a: 1.0,
};
pub const BG_HIGH: Hue = Hue {
  r: 0xFF,
  g: 0xFF,
  b: 0xFF,
  a: 0.05,
};
pub const BG_DEEP: Hue = Hue {
  r: 0x08,
  g: 0x08,
  b: 0x08,
  a: 1.0,
};
// pub const BG_PANEL: Hue = Hue { r: 0x16, g: 0x16, b: 0x16, a: 1.0 };
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
// pub const BLUE: Hue = Hue { r: 0x7B, g: 0x93, b: 0xFD, a: 1.0 };
// pub const YELLOW: Hue = Hue { r: 0xF0, g: 0xC0, b: 0x40, a: 1.0 };
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
// pub const FG_DIM: Hue = Hue { r: 0xC8, g: 0xC8, b: 0xC8, a: 1.0 };
pub const FG_MUTED: Hue = Hue {
  r: 0x66,
  g: 0x66,
  b: 0x66,
  a: 1.0,
};
// pub const FG_FAINT: Hue = Hue { r: 0x33, g: 0x33, b: 0x33, a: 1.0 };
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

#[cfg(not(target_os = "windows"))]
pub const TRAFFIC_LIGHT_RED: Hue = Hue {
  r: 0xFF,
  g: 0x60,
  b: 0x5C,
  a: 1.0,
};
#[cfg(not(target_os = "windows"))]
pub const TRAFFIC_LIGHT_YELLOW: Hue = Hue {
  r: 0xFF,
  g: 0xBD,
  b: 0x44,
  a: 1.0,
};
#[cfg(not(target_os = "windows"))]
pub const TRAFFIC_LIGHT_GREEN: Hue = Hue {
  r: 0x00,
  g: 0xCA,
  b: 0x4E,
  a: 1.0,
};
#[cfg(not(target_os = "windows"))]
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
