use iced::{
  Alignment, Border, Element, Length, Padding,
  border::Radius,
  widget::{container, image, mouse_area, row, space::horizontal},
};

use crate::ui::{
  app_state::Message, components::traffic_lights, helpers::til_home, theme,
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

pub fn title_bar(window_focused: bool, pwd: &String) -> Element<'static, Message> {
  let controls = traffic_lights(window_focused);

  let mark = image(MARK_HANDLE.clone())
    .width(13)
    .height(13);

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

  #[cfg(target_os = "windows")]
  let title_row = row![horizontal(), brand, horizontal(), controls,]
    .spacing(8)
    .height(40)
    .align_y(Alignment::Center);
  #[cfg(not(target_os = "windows"))]
  let title_row = row![controls, horizontal(), brand, horizontal(),]
    .spacing(8)
    .height(40)
    .align_y(Alignment::Center);

  mouse_area(
    container(title_row)
      .style(move |_| container::Style {
        background: Some(theme::color::BG_DEEP.as_color().into()),
        border: Border {
          color: theme::color::runtime().border,
          radius: Radius {
            top_left: 12.0,
            top_right: 12.0,
            ..Default::default()
          },
          width: 0.5,
        },
        ..container::Style::default()
      })
      .padding(Padding {
        #[cfg(target_os = "windows")]
        left: 46.0 * 3.0,
        #[cfg(not(target_os = "windows"))]
        left: 16.0,
        ..Default::default()
      })
      .width(Length::Fill)
      .center_y(40),
  )
  .on_press(Message::DragWindow)
  .into()
}
