use iced::widget::{Row, button, container, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding, Shadow};

use super::transform::press;
use crate::ui::icons::{Icon, icon};
use crate::ui::theme::tokens::{self, height, radius, space};
use crate::ui::theme::{color, font};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variant {
  Default,
  Ghost,
  Primary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconSize {
  Md,
  Sm,
}

impl IconSize {
  pub fn icon(self) -> f32 {
    match self {
      IconSize::Md => 20.0,
      IconSize::Sm => 16.0,
    }
  }

  pub fn r#box(self) -> f32 {
    match self {
      IconSize::Md => height::H40,
      IconSize::Sm => height::H32,
    }
  }
}

pub fn accent() -> Color {
  color::runtime().accent
}

fn brighten(c: Color, k: f32) -> Color {
  Color {
    r: (c.r * k).min(1.0),
    g: (c.g * k).min(1.0),
    b: (c.b * k).min(1.0),
    a: c.a,
  }
}

pub fn style(variant: Variant, status: button::Status) -> button::Style {
  let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
  let (bg, fg) = match variant {
    Variant::Default => (
      if hovered {
        tokens::SURFACE_3
      } else {
        tokens::SURFACE_2
      },
      tokens::FG_1,
    ),
    Variant::Ghost => (
      if hovered {
        tokens::SURFACE_2
      } else {
        Color::TRANSPARENT
      },
      if hovered { tokens::FG_1 } else { tokens::FG_2 },
    ),
    Variant::Primary => {
      let a = accent();
      (
        if hovered { brighten(a, 1.08) } else { a },
        tokens::ON_ACCENT,
      )
    }
  };
  button::Style {
    background: Some(Background::Color(bg)),
    text_color: fg,
    border: Border {
      radius: radius::R8.into(),
      ..Border::default()
    },
    shadow: Shadow::default(),
    snap: true,
  }
}

pub fn icon_button_style(status: button::Status) -> button::Style {
  let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
  button::Style {
    background: Some(Background::Color(if hovered {
      tokens::SURFACE_2
    } else {
      Color::TRANSPARENT
    })),
    text_color: if hovered { tokens::FG_1 } else { tokens::FG_2 },
    border: Border {
      radius: radius::R8.into(),
      ..Border::default()
    },
    shadow: Shadow::default(),
    snap: true,
  }
}

pub struct Btn<'a, Message> {
  variant: Variant,
  label: Option<String>,
  leading: Option<Icon>,
  trailing: Option<Icon>,
  extra: Option<Element<'a, Message>>,
  on_press: Option<Message>,
}

pub fn btn<'a, Message>(variant: Variant) -> Btn<'a, Message> {
  Btn {
    variant,
    label: None,
    leading: None,
    trailing: None,
    extra: None,
    on_press: None,
  }
}

impl<'a, Message: Clone + 'a> Btn<'a, Message> {
  pub fn label(mut self, label: impl Into<String>) -> Self {
    self.label = Some(label.into());
    self
  }

  pub fn leading(mut self, i: Icon) -> Self {
    self.leading = Some(i);
    self
  }

  pub fn trailing(mut self, i: Icon) -> Self {
    self.trailing = Some(i);
    self
  }

  pub fn extra(mut self, e: impl Into<Element<'a, Message>>) -> Self {
    self.extra = Some(e.into());
    self
  }

  pub fn on_press(mut self, m: Message) -> Self {
    self.on_press = Some(m);
    self
  }

  pub fn into_element(self) -> Element<'a, Message> {
    let variant = self.variant;
    let fg = style(variant, button::Status::Active).text_color;
    let weight = if variant == Variant::Primary {
      font::bold()
    } else {
      font::semibold()
    };
    let mut content = Row::new().spacing(space::S8).align_y(Alignment::Center);
    if let Some(i) = self.leading {
      content = content.push(icon(i, 16.0, fg));
    }
    if let Some(l) = self.label {
      content = content.push(text(l).size(tokens::text::T14).font(weight));
    }
    if let Some(i) = self.trailing {
      content = content.push(icon(i, 16.0, fg));
    }
    if let Some(e) = self.extra {
      content = content.push(e);
    }
    let b = button(
      container(content)
        .height(Length::Fill)
        .align_y(Alignment::Center),
    )
    .height(height::H40)
    .padding(Padding::from([0.0, space::S16]))
    .style(move |_t, s| style(variant, s))
    .on_press_maybe(self.on_press);
    press(b).into()
  }
}

impl<'a, Message: Clone + 'a> From<Btn<'a, Message>> for Element<'a, Message> {
  fn from(b: Btn<'a, Message>) -> Self {
    b.into_element()
  }
}

pub fn icon_button<'a, Message: Clone + 'a>(
  i: Icon,
  size: IconSize,
  on_press: Option<Message>,
) -> Element<'a, Message> {
  let fg = tokens::FG_2;
  let box_size = size.r#box();
  let b = button(
    container(icon(i, size.icon(), fg))
      .width(Length::Fill)
      .height(Length::Fill)
      .align_x(Alignment::Center)
      .align_y(Alignment::Center),
  )
  .width(box_size)
  .height(box_size)
  .padding(0)
  .style(|_t, s| icon_button_style(s))
  .on_press_maybe(on_press);
  press(b).into()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn ghost_is_transparent_until_hovered() {
    let idle = style(Variant::Ghost, button::Status::Active);
    let hover = style(Variant::Ghost, button::Status::Hovered);
    assert_eq!(idle.background, Some(Background::Color(Color::TRANSPARENT)));
    assert_eq!(hover.background, Some(Background::Color(tokens::SURFACE_2)));
    assert_eq!(hover.text_color, tokens::FG_1);
  }

  #[test]
  fn default_hover_steps_surface() {
    let hover = style(Variant::Default, button::Status::Hovered);
    assert_eq!(hover.background, Some(Background::Color(tokens::SURFACE_3)));
  }

  #[test]
  fn primary_uses_on_accent_text() {
    assert_eq!(
      style(Variant::Primary, button::Status::Active).text_color,
      tokens::ON_ACCENT
    );
  }

  #[test]
  fn small_icon_button_is_h32() {
    assert_eq!(IconSize::Sm.r#box(), height::H32);
    assert_eq!(IconSize::Sm.icon(), 16.0);
    assert_eq!(IconSize::Md.r#box(), height::H40);
  }

  #[test]
  fn radius_is_r8() {
    let s = style(Variant::Default, button::Status::Active);
    assert_eq!(s.border.radius, radius::R8.into());
  }
}
