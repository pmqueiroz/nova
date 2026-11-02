use iced::widget::{Space, container, row};
use iced::{Background, Border, Element, Length};

use crate::ui::app_state::Message;
use crate::ui::tab::PaneSide;
use crate::ui::theme::tokens::{radius, space};

pub fn split_drop_preview(side: PaneSide) -> Element<'static, Message> {
  let accent = crate::ui::theme::color::runtime().accent;
  let zone = container(Space::new())
    .width(Length::FillPortion(1))
    .height(Length::Fill)
    .style(move |_| container::Style {
      background: Some(Background::Color(accent.scale_alpha(0.12))),
      border: Border {
        color: accent.scale_alpha(0.6),
        width: 1.0,
        radius: radius::R8.into(),
      },
      ..container::Style::default()
    });
  let rest = Space::new().width(Length::FillPortion(1));
  let halves = match side {
    PaneSide::Left => row![zone, rest],
    PaneSide::Right => row![rest, zone],
  };
  container(halves.spacing(space::S8))
    .padding(space::S8)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
