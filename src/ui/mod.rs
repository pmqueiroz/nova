pub mod app_state;
pub mod components;
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
  .run()
}
