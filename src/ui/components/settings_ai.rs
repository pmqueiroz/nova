use iced::{
  Element, Padding,
  widget::{column, container, pick_list, text, text_input},
};

use super::{input_style, setting_row};
use crate::core::config::{self, AiProvider};
use crate::ui::{app_state::Message, theme};

pub(super) fn ai_tab<'a>(settings: &'a config::Config) -> Element<'a, Message> {
  let mut col = column![].spacing(20);

  let provider_list: Element<'a, Message> = pick_list(
    [AiProvider::Anthropic, AiProvider::OpenAi].as_slice(),
    Some(settings.ai.provider.clone()),
    Message::SettingsAiProviderChanged,
  )
  .font(theme::font::REGULAR)
  .text_size(12)
  .into();

  col = col.push(setting_row(
    "Provider",
    "AI provider backend",
    provider_list,
  ));

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
    (
      "AI features disabled — enter an API key to enable.",
      iced::Color {
        r: 0.9,
        g: 0.3,
        b: 0.3,
        a: 1.0,
      },
    )
  } else {
    ("AI features enabled.", theme::color::runtime().accent)
  };

  col = col.push(
    container(
      text(status_text)
        .font(theme::font::REGULAR)
        .size(11)
        .color(status_color),
    )
    .padding(Padding::from([4, 0])),
  );

  col.into()
}
