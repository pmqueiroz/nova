use iced::widget::{Row, button, container, mouse_area, row, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding};

use crate::core::config::WindowControls;
use crate::ui::app_state::Message;
use crate::ui::components::ds::button::{IconSize, icon_button};
use crate::ui::components::ds::transform::Transform;
use crate::ui::components::traffic_lights::{system_controls, traffic_lights};
use crate::ui::helpers::til_home;
use crate::ui::icons::Icon;
use crate::ui::theme::font;
use crate::ui::theme::tokens::{self, height, space};

pub const TITLE_BAR_HEIGHT: f32 = height::H48;

pub fn title<'a>(pwd: &str, flash: bool, compact: bool) -> Element<'a, Message> {
  let accent = crate::ui::theme::color::runtime().accent;
  row![
    text("Nova")
      .size(tokens::text::T14)
      .font(font::semibold())
      .color(if flash { accent } else { tokens::FG_2 }),
    text(if compact {
      String::new()
    } else {
      til_home(pwd)
    })
    .size(tokens::text::T14)
    .font(font::regular())
    .color(tokens::FG_3),
  ]
  .spacing(space::S8)
  .align_y(Alignment::Center)
  .into()
}

pub fn actions<'a>(update: Option<Element<'a, Message>>) -> Element<'a, Message> {
  Row::new()
    .spacing(space::S4)
    .align_y(Alignment::Center)
    .push(
      mouse_area(icon_button(Icon::Plus, IconSize::Sm, Some(Message::NewTab)))
        .on_right_press(Message::OpenShellPicker),
    )
    .push(icon_button(
      Icon::Sliders,
      IconSize::Sm,
      Some(Message::OpenSettings),
    ))
    .push(update)
    .into()
}

pub fn title_bar<'a>(
  center: Element<'a, Message>,
  cursor_interaction: iced::mouse::Interaction,
  window_controls: &WindowControls,
  update: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
  let system = *window_controls == WindowControls::System;
  let (left, right): (Element<'a, Message>, Element<'a, Message>) = if system {
    (actions(update), system_controls())
  } else {
    (traffic_lights(), actions(update))
  };

  let bar = row![
    container(left)
      .width(Length::Fill)
      .align_x(Alignment::Start)
      .center_y(TITLE_BAR_HEIGHT),
    container(center).center_y(TITLE_BAR_HEIGHT),
    container(right)
      .width(Length::Fill)
      .align_x(Alignment::End)
      .center_y(TITLE_BAR_HEIGHT),
  ]
  .height(TITLE_BAR_HEIGHT)
  .align_y(Alignment::Center);

  let padding = if system {
    Padding {
      top: 0.0,
      right: 0.0,
      bottom: 0.0,
      left: space::S16,
    }
  } else {
    Padding {
      top: 0.0,
      right: space::S16,
      bottom: 0.0,
      left: space::S24,
    }
  };

  mouse_area(
    container(bar)
      .padding(padding)
      .width(Length::Fill)
      .height(TITLE_BAR_HEIGHT),
  )
  .on_press(Message::DragWindow)
  .on_double_click(Message::MaximizeWindow)
  .interaction(cursor_interaction)
  .into()
}

pub fn light<'a>(color: Color, msg: Message) -> Element<'a, Message> {
  let dot = button(iced::widget::Space::new().width(12).height(12))
    .padding(0)
    .on_press(msg)
    .style(move |_t, _s| button::Style {
      background: Some(Background::Color(color)),
      border: Border {
        radius: 6.0.into(),
        ..Border::default()
      },
      ..button::Style::default()
    });
  Transform::new(dot)
    .hover(1.16)
    .press_to(0.84)
    .spring()
    .into()
}
