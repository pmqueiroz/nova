use iced::Element;
use iced::widget::text;

use crate::ui::{app_state::Message, theme};

pub fn section_label<'a>(label: &'static str) -> Element<'a, Message> {
  let fg_muted = theme::color::runtime().foreground_muted;
  text(label)
    .font(theme::font::BOLD)
    .size(10)
    .color(fg_muted)
    .into()
}
