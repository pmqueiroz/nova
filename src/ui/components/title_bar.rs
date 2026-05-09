use iced::{
  Alignment, Border, Element, Length, Padding,
  alignment::Horizontal,
  border::Radius,
  widget::{button, container, image, mouse_area, row, text},
};

use crate::core::config::WindowControls;
use crate::ui::{
  app_state::Message,
  components::traffic_lights::{system_controls, traffic_lights},
  helpers::til_home,
  theme,
  typography::{Typography, Weight},
};

const MARK_PNG: &[u8] = include_bytes!("../../../assets/icons/mark/nova-mark-24.png");

static MARK_HANDLE: std::sync::LazyLock<image::Handle> =
  std::sync::LazyLock::new(|| image::Handle::from_bytes(MARK_PNG));

fn nova_label() -> impl Into<Element<'static, Message>> {
  let text = |s: &'static str, accent: bool| {
    Typography {
      color: if accent {
        theme::color::runtime().accent
      } else {
        theme::color::runtime().foreground
      },
      size: 12.into(),
      weight: Weight::Bold,
    }
    .as_text(s)
  };

  row![text("no", false), text("v", true), text("a", false),].spacing(0)
}

fn settings_button() -> Element<'static, Message> {
  button(text("󰒓").size(14))
    .style(move |_t, status| button::Style {
      text_color: match status {
        button::Status::Hovered => theme::color::runtime().accent,
        _ => theme::color::runtime().foreground_muted,
      },
      background: Some(theme::color::TRANSPARENT.as_color().into()),
      ..Default::default()
    })
    .on_press(Message::OpenSettings)
    .padding(Padding::from([4, 6]))
    .into()
}

pub fn title_bar(
  window_focused: bool,
  pwd: &str,
  maximized: bool,
  cursor_interaction: iced::mouse::Interaction,
  window_controls: &WindowControls,
) -> Element<'static, Message> {
  let controls = match window_controls {
    WindowControls::TrafficLights => traffic_lights(window_focused),
    WindowControls::System => system_controls(window_focused),
  };

  let mark = image(MARK_HANDLE.clone()).width(13).height(13);

  let pwd_text = Typography {
    color: theme::color::runtime().foreground_muted,
    size: 12.into(),
    ..Default::default()
  }
  .as_text(format!("- {}", til_home(pwd)));

  let brand = row![mark, nova_label().into(), pwd_text,]
    .spacing(6)
    .height(40)
    .align_y(Alignment::Center);

  let title_row = if *window_controls == WindowControls::System {
    row![
      container(settings_button())
        .width(Length::FillPortion(1))
        .center_y(40)
        .padding(Padding {
          left: 8.0,
          ..Default::default()
        }),
      container(brand)
        .center_x(Length::FillPortion(2))
        .center_y(40),
      container(controls)
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .align_x(Horizontal::Right),
    ]
    .height(40)
  } else {
    row![
      container(controls)
        .width(Length::FillPortion(1))
        .center_y(40)
        .padding(Padding {
          left: 8.0,
          ..Default::default()
        }),
      container(brand)
        .center_x(Length::FillPortion(2))
        .center_y(40),
      container(settings_button())
        .width(Length::FillPortion(1))
        .center_y(40)
        .align_x(Horizontal::Right)
        .padding(Padding {
          right: 8.0,
          ..Default::default()
        }),
    ]
    .height(40)
  };

  let corner_radius = if maximized {
    Radius::default()
  } else {
    Radius {
      top_left: 12.0,
      top_right: 12.0,
      ..Default::default()
    }
  };

  mouse_area(
    container(title_row)
      .style(move |_| container::Style {
        background: Some(theme::color::BG_DEEP.as_color().into()),
        border: Border {
          color: theme::color::runtime().border,
          radius: corner_radius,
          width: 0.5,
        },
        ..container::Style::default()
      })
      .padding(Padding::default())
      .width(Length::Fill)
      .height(40),
  )
  .on_press(Message::DragWindow)
  .on_double_click(Message::MaximizeWindow)
  .interaction(cursor_interaction)
  .into()
}
