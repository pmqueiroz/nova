use iced::Color;

pub const BG: Hue = Hue {
  r: 0.05098, // 0x0D
  g: 0.05098,
  b: 0.05098,
  a: 1.0,
};
pub const BG_DEEP: Hue = Hue {
  r: 0.03137, // 0x08
  g: 0.03137,
  b: 0.03137,
  a: 1.0,
};
// pub const BG_PANEL: Hue = Hue {
//   r: 0.08627, // 0x16
//   g: 0.08627,
//   b: 0.08627,
//   a: 1.0,
// };
pub const ACCENT: Hue = Hue {
  r: 0.24314, // 0x3E
  g: 0.81176, // 0xCF
  b: 0.55686, // 0x8E
  a: 1.0,
};
// pub const ACCENT_DIM: Hue = Hue {
//   r: 0.24314, // 0x3E
//   g: 0.81176, // 0xCF
//   b: 0.55686, // 0x8E
//   a: 0.15,
// };
// pub const BLUE: Hue = Hue {
//   r: 0.48235, // 0x7B
//   g: 0.57647, // 0x93
//   b: 0.99216, // 0xFD
//   a: 1.0,
// };
// pub const YELLOW: Hue = Hue {
//   r: 0.94118, // 0xF0
//   g: 0.75294, // 0xC0
//   b: 0.25098, // 0x40
//   a: 1.0,
// };
// pub const RED: Hue = Hue {
//   r: 1.0,     // 0xFF
//   g: 0.37255, // 0x5F
//   b: 0.34118, // 0x57
//   a: 1.0,
// };
pub const FG: Hue = Hue {
  r: 0.89804, // 0xE5
  g: 0.89804,
  b: 0.89804,
  a: 1.0,
};
// pub const FG_DIM: Hue = Hue {
//   r: 0.78431, // 0xC8
//   g: 0.78431,
//   b: 0.78431,
//   a: 1.0,
// };
pub const FG_MUTED: Hue = Hue {
  r: 0.4, // 0x66
  g: 0.4,
  b: 0.4,
  a: 1.0,
};
// pub const FG_FAINT: Hue = Hue {
//   r: 0.2, // 0x33
//   g: 0.2,
//   b: 0.2,
//   a: 1.0,
// };
pub const BORDER: Hue = Hue {
  r: 1.0, // 0xFF
  g: 1.0,
  b: 1.0,
  a: 0.06,
};
pub const TRANSPARENT: Hue = Hue {
  r: 0.0, // 0x00
  g: 0.0,
  b: 0.0,
  a: 0.0,
};
// macos traffic light colors
pub const TRAFFIC_LIGHT_RED: Hue = Hue {
  r: 1.0,     // 0xFF
  g: 0.37647, // 0x60
  b: 0.36078, // 0x5C
  a: 1.0,
};
pub const TRAFFIC_LIGHT_YELLOW: Hue = Hue {
  r: 1.0,     // 0xFF
  g: 0.74118, // 0xBD
  b: 0.26667, // 0x44
  a: 1.0,
};
pub const TRAFFIC_LIGHT_GREEN: Hue = Hue {
  r: 0.0,     // 0x00
  g: 0.79216, // 0xCA
  b: 0.30588, // 0x4E
  a: 1.0,
};
pub const TRAFFIC_LIGHT_INACTIVE: Hue = Hue {
  r: 0.29804, // 0x4C
  g: 0.29804,
  b: 0.29804,
  a: 1.0,
};

pub struct Hue {
  pub r: f32,
  pub g: f32,
  pub b: f32,
  pub a: f32,
}

impl Hue {
  pub fn as_color(&self) -> Color {
    Color {
      r: self.r,
      g: self.g,
      b: self.b,
      a: self.a,
    }
  }

  pub fn with_alpha(&self, alpha: f32) -> Self {
    Self { a: alpha, ..*self }
  }
}
