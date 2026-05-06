use iced::widget::text;

pub struct Typography {
  pub color: iced::Color,
  pub size: iced::Pixels,
}

impl Default for Typography {
  fn default() -> Self {
    Self {
      color: iced::Color::WHITE,
      size: 16.into(),
    }
  }
}

impl Typography {
  pub fn as_text<'a>(&self, content: impl Into<String>) -> iced::widget::Text<'a> {
    text(content.into())
      .font(iced::Font::MONOSPACE)
      .color(self.color)
      .size(self.size)
      .into()
  }

  pub fn span<'a>(content: impl Into<String>) -> iced::widget::Text<'a> {
    Typography {
      ..Default::default()
    }
    .as_text(content)
  }
}
