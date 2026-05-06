use iced::widget::text;

use crate::ui::theme;

pub enum Weight {
  Normal,
  Bold,
}

pub struct Typography {
  pub color: iced::Color,
  pub size: iced::Pixels,
  pub weight: Weight,
}

impl Default for Typography {
  fn default() -> Self {
    Self {
      color: iced::Color::WHITE,
      size: 16.into(),
      weight: Weight::Normal,
    }
  }
}

impl Typography {
  pub fn as_text<'a>(&self, content: impl Into<String>) -> iced::widget::Text<'a> {
    text(content.into())
      .font(match self.weight {
        Weight::Bold => theme::FIRA_CODE_BOLD_FONT,
        Weight::Normal => theme::FIRA_CODE_FONT,
      })
      .color(self.color)
      .size(self.size)
      .into()
  }
}
