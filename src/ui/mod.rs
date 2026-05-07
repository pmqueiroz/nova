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
  include_bytes!("../../assets/icons/macos/nova-mac-1024.png");

#[cfg(target_os = "linux")]
const ICON_BYTES: &[u8] =
  include_bytes!("../../assets/icons/linux/nova-linux-256.png");

fn app_icon() -> Option<window::Icon> {
  let img = image::load_from_memory(ICON_BYTES).ok()?;
  let rgba = img.into_rgba8();
  let (width, height) = rgba.dimensions();
  window::icon::from_rgba(rgba.into_raw(), width, height).ok()
}

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
    icon: app_icon(),
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
