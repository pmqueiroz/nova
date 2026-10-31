use iced::widget::{Row, button, container, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding};

use super::transform::press;
use crate::ui::icons::{Icon, icon};
use crate::ui::theme::font;
use crate::ui::theme::tokens::{self, height, radius, space};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Tone {
  #[default]
  Normal,
  Error,
}

pub fn list_item_style(selected: bool) -> button::Style {
  button::Style {
    background: Some(Background::Color(if selected {
      tokens::SURFACE_2
    } else {
      Color::TRANSPARENT
    })),
    text_color: if selected { tokens::FG_1 } else { tokens::FG_2 },
    border: Border {
      radius: radius::R8.into(),
      ..Border::default()
    },
    ..button::Style::default()
  }
}

pub struct ListItem<'a, Message> {
  pub icon: Icon,
  pub label: String,
  pub hint: Option<String>,
  pub keys: Option<Element<'a, Message>>,
  pub tone: Tone,
}

pub fn list_item<'a, Message: Clone + 'a>(
  item: ListItem<'a, Message>,
  selected: bool,
  on_hover: Message,
  on_press: Message,
) -> Element<'a, Message> {
  let accent = crate::ui::theme::color::runtime().accent;
  let icon_color = match (item.tone, selected) {
    (Tone::Error, _) => tokens::RED,
    (_, true) => accent,
    _ => tokens::FG_3,
  };
  let text_color = if selected { tokens::FG_1 } else { tokens::FG_2 };
  let mut content = Row::new()
    .spacing(space::S16)
    .align_y(Alignment::Center)
    .push(icon(item.icon, 20.0, icon_color))
    .push(
      container(text(item.label).size(tokens::text::T16).color(text_color)).width(Length::Fill),
    );
  if let Some(h) = item.hint {
    content = content.push(text(h).size(tokens::text::T14).font(font::regular()).color(
      if item.tone == Tone::Error {
        tokens::RED
      } else {
        tokens::FG_3
      },
    ));
  }
  if let Some(k) = item.keys {
    content = content.push(k);
  }
  let b = button(
    container(content)
      .height(Length::Fill)
      .center_y(Length::Fill),
  )
  .width(Length::Fill)
  .height(height::H48)
  .padding(Padding::from([0.0, space::S16]))
  .style(move |_t, _s| list_item_style(selected))
  .on_press(on_press);
  press(
    iced::widget::mouse_area(b)
      .on_enter(on_hover.clone())
      .on_move(move |_| on_hover.clone()),
  )
  .into()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn selected_item_is_raised() {
    let s = list_item_style(true);
    assert_eq!(s.background, Some(Background::Color(tokens::SURFACE_2)));
    assert_eq!(s.text_color, tokens::FG_1);
    assert_eq!(list_item_style(false).text_color, tokens::FG_2);
  }
}
