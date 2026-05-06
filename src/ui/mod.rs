use iced::{Size, window};

pub mod app_state;
pub mod components;
mod helpers;
mod tab;
pub mod theme;
pub mod typography;

pub type Result = iced::Result;

const FIRA_CODE_BYTES: &[u8] = include_bytes!("../../fonts/FiraCode-Regular.ttf");
const FIRA_CODE_BOLD_BYTES: &[u8] = include_bytes!("../../fonts/FiraCode-Bold.ttf");

pub fn start() -> Result {
  iced::application(
    app_state::Nova::default,
    app_state::Nova::update,
    app_state::Nova::view,
  )
  .theme(app_state::Nova::theme)
  .subscription(app_state::Nova::subscription)
  .window(window::Settings {
    size: Size::new(1024.0, 768.0),
    transparent: true,
    decorations: false,
    ..Default::default()
  })
  .style(|_s, _t| iced::theme::Style {
    background_color: theme::color::TRANSPARENT.as_color(),
    text_color: theme::color::FG.as_color(),
  })
  .font(FIRA_CODE_BYTES)
  .font(FIRA_CODE_BOLD_BYTES)
  .default_font(theme::font::REGULAR)
  .antialiasing(true)
  .run()
}
