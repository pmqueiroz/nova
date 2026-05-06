use iced::{Size, window};

pub mod app_state;
pub mod components;
pub mod theme;
pub mod typography;

pub type Result = iced::Result;

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
    background_color: theme::TRANSPARENT.as_color(),
    text_color: theme::FG.as_color(),
  })
  .run()
}
