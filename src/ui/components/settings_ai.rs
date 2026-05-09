use iced::{
  Border, Element, Padding, Shadow,
  border::Radius,
  widget::{column, container, overlay::menu, pick_list, text, text_input},
};

use crate::core::config::{self, AiProvider};
use crate::ui::{app_state::Message, theme};
use super::{input_style, setting_row};

pub(super) fn ai_tab<'a>(settings: &'a config::Config) -> Element<'a, Message> {
  let mut col = column![].spacing(20);

  let provider_list: Element<'a, Message> = pick_list(
    [AiProvider::Anthropic, AiProvider::OpenAi].as_slice(),
    Some(settings.ai.provider.clone()),
    Message::SettingsAiProviderChanged,
  )
  .font(theme::font::REGULAR)
  .text_size(12)
  .style(|_t, status| {
    let rt = theme::color::runtime();
    let (border_c, fg, fg_muted) = (rt.border, rt.foreground, rt.foreground_muted);
    let accent = rt.accent;
    drop(rt);
    let active_border = match status {
      pick_list::Status::Opened { .. } | pick_list::Status::Hovered => accent,
      _ => border_c,
    };
    pick_list::Style {
      text_color: fg,
      background: theme::color::BG_HIGH.as_color().into(),
      border: Border { color: active_border, radius: Radius::new(4.0), width: 1.0 },
      handle_color: fg_muted,
      placeholder_color: fg_muted,
    }
  })
  .menu_style(|_t| {
    let rt = theme::color::runtime();
    let (fg, accent, border_c) = (rt.foreground, rt.accent, rt.border);
    drop(rt);
    menu::Style {
      background: theme::color::BG_DEEP.as_color().into(),
      border: Border { color: border_c, radius: Radius::new(6.0), width: 1.0 },
      text_color: fg,
      selected_text_color: theme::color::BG_DEEP.as_color(),
      selected_background: accent.into(),
      shadow: Shadow::default(),
    }
  })
  .into();

  col = col.push(setting_row("Provider", "AI provider backend", provider_list));

  col = col.push(setting_row(
    "Model",
    "Model name to use",
    text_input("e.g. claude-haiku-4-5-20251001", &settings.ai.model)
      .on_input(Message::SettingsAiModelChanged)
      .font(theme::font::REGULAR)
      .size(12)
      .style(input_style)
      .padding(Padding::from([6, 10]))
      .into(),
  ));

  let base_url_val: &'a str = settings.ai.base_url.as_deref().unwrap_or("");
  col = col.push(setting_row(
    "API Key",
    "Authentication key (masked)",
    text_input("sk-...", &settings.ai.api_key)
      .secure(true)
      .on_input(Message::SettingsAiApiKeyChanged)
      .font(theme::font::REGULAR)
      .size(12)
      .style(input_style)
      .padding(Padding::from([6, 10]))
      .into(),
  ));

  col = col.push(setting_row(
    "Base URL",
    "Custom endpoint (optional)",
    text_input("https://api.anthropic.com", base_url_val)
      .on_input(Message::SettingsAiBaseUrlChanged)
      .font(theme::font::REGULAR)
      .size(12)
      .style(input_style)
      .padding(Padding::from([6, 10]))
      .into(),
  ));

  let (status_text, status_color) = if settings.ai.api_key.is_empty() {
    ("AI features disabled — enter an API key to enable.", iced::Color { r: 0.9, g: 0.3, b: 0.3, a: 1.0 })
  } else {
    ("AI features enabled.", theme::color::runtime().accent)
  };

  col = col.push(
    container(text(status_text).font(theme::font::REGULAR).size(11).color(status_color))
      .padding(Padding::from([4, 0])),
  );

  col.into()
}
