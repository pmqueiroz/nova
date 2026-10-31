use std::time::Duration;

use iced::Color;

pub mod space {
  pub const S2: f32 = 2.0;
  pub const S4: f32 = 4.0;
  pub const S8: f32 = 8.0;
  pub const S12: f32 = 12.0;
  pub const S16: f32 = 16.0;
  pub const S24: f32 = 24.0;
  pub const S32: f32 = 32.0;
  pub const S48: f32 = 48.0;
  pub const S64: f32 = 64.0;
}

pub mod radius {
  pub const R4: f32 = 4.0;
  pub const R8: f32 = 8.0;
  pub const R12: f32 = 12.0;
  pub const R16: f32 = 16.0;
}

pub mod text {
  pub const T12: f32 = 12.0;
  pub const T14: f32 = 14.0;
  pub const T16: f32 = 16.0;
  pub const T18: f32 = 18.0;
  pub const T24: f32 = 24.0;
}

pub mod height {
  pub const H24: f32 = 24.0;
  pub const H32: f32 = 32.0;
  pub const H40: f32 = 40.0;
  pub const H48: f32 = 48.0;
  pub const H56: f32 = 56.0;
  pub const H64: f32 = 64.0;
}

pub mod duration {
  use super::Duration;

  pub const D1: Duration = Duration::from_millis(120);
  pub const D2: Duration = Duration::from_millis(200);
  pub const D4: Duration = Duration::from_millis(400);
}

pub mod block {
  pub const LINE_HEIGHT: f32 = 1.5;

  pub fn font_size(font_size: f32) -> f32 {
    (font_size - super::space::S2).max(1.0)
  }
}

pub mod font_size {
  pub const MIN: f32 = 14.0;
  pub const MAX: f32 = 20.0;
  pub const STEP: f32 = 2.0;
  pub const DEFAULT: f32 = 16.0;
}

const fn rgb(hex: u32) -> Color {
  Color {
    r: ((hex >> 16) & 0xFF) as f32 / 255.0,
    g: ((hex >> 8) & 0xFF) as f32 / 255.0,
    b: (hex & 0xFF) as f32 / 255.0,
    a: 1.0,
  }
}

const fn white(a: f32) -> Color {
  Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a,
  }
}

pub const WIN_TOP: Color = rgb(0x2A3139);
pub const WIN_BOT: Color = rgb(0x15181F);

pub const SURFACE_1: Color = white(0.04);
pub const SURFACE_2: Color = white(0.08);
pub const SURFACE_3: Color = white(0.12);
pub const INPUT_FOCUS: Color = white(0.10);

pub const LINE: Color = white(0.08);
pub const LINE_2: Color = white(0.14);

pub const FG_1: Color = rgb(0xF0F2F5);
pub const FG_2: Color = rgb(0xCDD1D8);
pub const FG_3: Color = rgb(0x9399A2);

pub const GREEN: Color = rgb(0x88E48C);
pub const RED: Color = rgb(0xFD736D);
pub const BLUE: Color = rgb(0x82BAFF);
pub const YELLOW: Color = rgb(0xF2CD64);
pub const MAGENTA: Color = rgb(0xD695E4);
pub const CYAN: Color = rgb(0x72DEE4);

pub const ON_ACCENT: Color = rgb(0x11161F);

pub const LIGHT_CLOSE: Color = rgb(0xF75E54);
pub const LIGHT_MIN: Color = rgb(0xFAC131);
pub const LIGHT_MAX: Color = rgb(0x61CC69);

pub const SCRIM: Color = Color {
  r: 0x04 as f32 / 255.0,
  g: 0x06 as f32 / 255.0,
  b: 0x09 as f32 / 255.0,
  a: 0.56,
};

pub struct AccentPreset {
  pub name: &'static str,
  pub hex: &'static str,
  pub color: Color,
}

pub const ACCENTS: [AccentPreset; 4] = [
  AccentPreset {
    name: "Blue",
    hex: "#82BAFF",
    color: BLUE,
  },
  AccentPreset {
    name: "Mint",
    hex: "#65DFAF",
    color: rgb(0x65DFAF),
  },
  AccentPreset {
    name: "Amber",
    hex: "#FCBE62",
    color: rgb(0xFCBE62),
  },
  AccentPreset {
    name: "Rose",
    hex: "#FC899D",
    color: rgb(0xFC899D),
  },
];

pub fn with_alpha(c: Color, a: f32) -> Color {
  Color { a: c.a * a, ..c }
}

pub fn mix(a: Color, b: Color, t: f32) -> Color {
  Color {
    r: a.r + (b.r - a.r) * t,
    g: a.g + (b.g - a.g) * t,
    b: a.b + (b.b - a.b) * t,
    a: a.a + (b.a - a.a) * t,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rgb_unpacks_channels() {
    let c = rgb(0xFF8000);
    assert_eq!(c.r, 1.0);
    assert!((c.g - 128.0 / 255.0).abs() < f32::EPSILON);
    assert_eq!(c.b, 0.0);
  }

  #[test]
  fn accent_hex_matches_color() {
    for preset in ACCENTS.iter() {
      let parsed = crate::core::config::parse_hex_color(preset.hex).unwrap();
      assert!((parsed.r - preset.color.r).abs() < 1e-3);
      assert!((parsed.g - preset.color.g).abs() < 1e-3);
      assert!((parsed.b - preset.color.b).abs() < 1e-3);
    }
  }

  #[test]
  fn sizes_are_multiples_of_two() {
    let all = [
      space::S2,
      space::S4,
      space::S8,
      space::S12,
      space::S16,
      space::S24,
      space::S32,
      space::S48,
      space::S64,
      radius::R4,
      radius::R8,
      radius::R12,
      radius::R16,
      text::T12,
      text::T14,
      text::T16,
      text::T18,
      text::T24,
      height::H24,
      height::H32,
      height::H40,
      height::H48,
      height::H56,
      height::H64,
    ];
    for v in all {
      assert_eq!(v % 2.0, 0.0);
    }
  }
}
