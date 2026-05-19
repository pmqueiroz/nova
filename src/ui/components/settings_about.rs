use iced::{
  ContentFit, Element, Length,
  alignment::Horizontal,
  widget::{column, container, svg, text},
};

use crate::ui::{app_state::Message, theme};

const BRAND_SVG: &[u8] = include_bytes!("../../../assets/brand/nova-brand.svg");

static BRAND_HANDLE: std::sync::LazyLock<svg::Handle> =
  std::sync::LazyLock::new(|| svg::Handle::from_memory(BRAND_SVG));

pub(super) fn about_tab<'a>() -> Element<'a, Message> {
  let fg_muted = theme::color::runtime().foreground_muted;

  container(
    column![
      svg(BRAND_HANDLE.clone())
        .width(260)
        .content_fit(ContentFit::Contain),
      text(format!("v{}", env!("CARGO_PKG_VERSION")))
        .font(theme::font::regular())
        .size(12)
        .color(fg_muted),
    ]
    .spacing(12)
    .align_x(Horizontal::Center),
  )
  .width(Length::Fill)
  .height(Length::Fill)
  .align_x(Horizontal::Center)
  .align_y(iced::alignment::Vertical::Center)
  .into()
}
