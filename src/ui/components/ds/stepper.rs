use iced::widget::{Row, container, text};
use iced::{Alignment, Element};

use super::button::icon_button;
use super::seg::track_style;
use crate::ui::icons::Icon;
use crate::ui::theme::tokens::{self, height, space};

pub fn stepper<'a, Message: Clone + 'a>(
  value: impl ToString,
  on_dec: Option<Message>,
  on_inc: Option<Message>,
) -> Element<'a, Message> {
  container(
    Row::new()
      .spacing(space::S4)
      .align_y(Alignment::Center)
      .push(icon_button(Icon::ZoomOut, 16.0, height::H32, on_dec))
      .push(
        container(
          text(value.to_string())
            .size(tokens::text::T16)
            .color(tokens::FG_1),
        )
        .width(space::S48)
        .center_x(space::S48),
      )
      .push(icon_button(Icon::ZoomIn, 16.0, height::H32, on_inc)),
  )
  .padding(space::S4)
  .style(|_| track_style())
  .into()
}
