use iced::{Size, window};

pub mod app_state;
pub mod components;
mod helpers;
mod tab;
pub mod theme;
pub mod typography;

pub type Result = iced::Result;

const FIRA_CODE_BYTES: &[u8] = include_bytes!("../../assets/fonts/FiraCodeNerdFont-Regular.ttf");
const FIRA_CODE_BOLD_BYTES: &[u8] = include_bytes!("../../assets/fonts/FiraCodeNerdFont-Bold.ttf");

#[cfg(target_os = "windows")]
const ICON_BYTES: &[u8] =
  include_bytes!("../../assets/icons/windows/nova-win-256.png");

#[cfg(target_os = "macos")]
const ICON_BYTES: &[u8] =
  include_bytes!("../../assets/icons/macos/nova-mac-512.png");

#[cfg(target_os = "linux")]
const ICON_BYTES: &[u8] =
  include_bytes!("../../assets/icons/linux/nova-linux-256.png");

fn app_icon() -> Option<window::Icon> {
  let img = image::load_from_memory(ICON_BYTES).ok()?;
  let rgba = img.into_rgba8();
  let (width, height) = rgba.dimensions();
  window::icon::from_rgba(rgba.into_raw(), width, height).ok()
}

fn window_settings() -> window::Settings {
  #[allow(unused_mut)]
  let mut settings = window::Settings {
    size: Size::new(1024.0, 768.0),
    transparent: true,
    decorations: false,
    icon: app_icon(),
    ..Default::default()
  };
  #[cfg(target_os = "windows")]
  {
    settings.platform_specific.corner_preference =
      window::settings::platform::CornerPreference::Round;
  }
  #[cfg(target_os = "linux")]
  {
    settings.platform_specific.application_id = "nova".to_string();
  }
  settings
}

pub fn start() -> Result {
  iced::application(
    app_state::Nova::default,
    app_state::Nova::update,
    app_state::Nova::view,
  )
  .theme(app_state::Nova::theme)
  .subscription(app_state::Nova::subscription)
  .window(window_settings())
  .style(|_s, _t| iced::theme::Style {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    background_color: theme::color::TRANSPARENT.as_color(),
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    background_color: theme::color::BG_DEEP.as_color(),
    text_color: theme::color::runtime().foreground,
  })
  .font(FIRA_CODE_BYTES)
  .font(FIRA_CODE_BOLD_BYTES)
  .default_font(theme::font::REGULAR)
  .antialiasing(true)
  .run()
}
