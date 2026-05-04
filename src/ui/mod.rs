pub mod app_state;

pub type Result = iced::Result;

pub fn start() -> iced::Result {
  iced::application(
    app_state::Nova::default,
    app_state::Nova::update,
    app_state::Nova::view,
  )
  .theme(app_state::Nova::theme)
  .run()
}
