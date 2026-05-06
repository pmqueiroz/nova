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
  .run()
}
