use std::sync::LazyLock;

use iced::widget::{
  Column, Row, button, column, container, image, pick_list, row, scrollable, text, text_input,
};
use iced::{Alignment, Background, Border, Color, ContentFit, Element, Length, Padding};

use crate::core::config::{self, AiProvider, BellType, CursorStyle, WindowControls};
use crate::ui::app_state::{ColorField, Message, SettingsTab};
use crate::ui::components::ds::button::{IconSize, Variant, btn, icon_button};
use crate::ui::components::ds::kbd::{binding_labels, keys};
use crate::ui::components::ds::row::row as setting_row;
use crate::ui::components::ds::seg::seg;
use crate::ui::components::ds::sheet::{self, Sheet};
use crate::ui::components::ds::stepper::stepper;
use crate::ui::components::ds::swatch::swatch;
use crate::ui::components::ds::toggle::toggle;
use crate::ui::icons::Icon;
use crate::ui::theme::tokens::{self, font_size, height, radius, space};
use crate::ui::theme::{color, font};

const BRAND_PNG: &[u8] =
  include_bytes!("../../../assets/brand/lockup/nova-lockup-for-dark-256h.png");

static BRAND_HANDLE: LazyLock<image::Handle> =
  LazyLock::new(|| image::Handle::from_bytes(BRAND_PNG));

pub const TABS: [(SettingsTab, &str); 5] = [
  (SettingsTab::Appearance, "Appearance"),
  (SettingsTab::Shell, "Shell"),
  (SettingsTab::Keys, "Keys"),
  (SettingsTab::Ai, "AI"),
  (SettingsTab::About, "About"),
];

pub const EDITORS: [&str; 3] = ["nano", "vim", "code"];

pub fn field_style(status: text_input::Status) -> text_input::Style {
  let accent = color::runtime().accent;
  let focused = matches!(status, text_input::Status::Focused { .. });
  text_input::Style {
    background: Background::Color(tokens::SURFACE_1),
    border: Border {
      radius: radius::R8.into(),
      width: 1.0,
      color: if focused {
        tokens::with_alpha(accent, 0.48)
      } else {
        tokens::LINE
      },
    },
    icon: tokens::FG_3,
    placeholder: tokens::FG_3,
    value: tokens::FG_1,
    selection: tokens::with_alpha(accent, 0.32),
  }
}

fn field<'a>(
  placeholder: &str,
  value: &str,
  on_input: impl Fn(String) -> Message + 'a,
) -> text_input::TextInput<'a, Message> {
  text_input(placeholder, value)
    .on_input(on_input)
    .size(tokens::text::T14)
    .font(font::regular())
    .padding(Padding::from([space::S8, space::S12]))
    .width(240)
    .style(|_t, s| field_style(s))
}

fn rows<'a>(items: Vec<(String, Option<String>, Element<'a, Message>)>) -> Element<'a, Message> {
  let n = items.len();
  items
    .into_iter()
    .enumerate()
    .fold(Column::new(), |c, (i, (label, hint, control))| {
      c.push(setting_row(label, hint, control, i + 1 == n))
    })
    .into()
}

pub fn step_font(size: f32, delta: f32) -> f32 {
  let snapped = (size / font_size::STEP).round() * font_size::STEP;
  (snapped + delta).clamp(font_size::MIN, font_size::MAX)
}

fn appearance<'a>(s: &'a config::Config, timestamps: bool) -> Element<'a, Message> {
  let size = s.theme.font.size;
  let current_accent = config::parse_hex_color(&s.theme.colors.accent).ok();
  let swatches = tokens::ACCENTS.iter().fold(
    Row::new().spacing(space::S8).align_y(Alignment::Center),
    |r, p| {
      let on = current_accent.is_some_and(|c| {
        (c.r - p.color.r).abs() < 0.01
          && (c.g - p.color.g).abs() < 0.01
          && (c.b - p.color.b).abs() < 0.01
      });
      r.push(swatch(
        p.color,
        on,
        Message::SettingsColorChanged(ColorField::Accent, p.hex.to_string()),
      ))
    },
  );
  let fonts = crate::ui::theme::font::installed_fonts();
  let selected_font = fonts.iter().find(|f| **f == s.theme.font.family).cloned();
  rows(vec![
    (
      "Text size".into(),
      Some("Terminal output and input".into()),
      stepper(
        size.round(),
        (size > font_size::MIN)
          .then(|| Message::SettingsFontSizeChanged(step_font(size, -font_size::STEP))),
        (size < font_size::MAX)
          .then(|| Message::SettingsFontSizeChanged(step_font(size, font_size::STEP))),
      ),
    ),
    (
      "Timestamps".into(),
      Some("Show when each command ran".into()),
      toggle(timestamps, |_| Message::ToggleTimestamps),
    ),
    (
      "Accent".into(),
      Some("Cursor, prompt and highlights".into()),
      swatches.into(),
    ),
    (
      "Font".into(),
      Some("Monospace family for the terminal".into()),
      pick_list(fonts, selected_font, Message::SettingsFontFamilyChanged)
        .placeholder(s.theme.font.family.clone())
        .text_size(tokens::text::T14)
        .padding(Padding::from([space::S8, space::S12]))
        .width(240)
        .style(|_t, st| {
          let hovered = matches!(
            st,
            iced::widget::pick_list::Status::Hovered
              | iced::widget::pick_list::Status::Opened { .. }
          );
          iced::widget::pick_list::Style {
            text_color: tokens::FG_1,
            placeholder_color: tokens::FG_1,
            handle_color: tokens::FG_3,
            background: Background::Color(if hovered {
              tokens::SURFACE_2
            } else {
              tokens::SURFACE_1
            }),
            border: Border {
              radius: radius::R8.into(),
              width: 1.0,
              color: tokens::LINE,
            },
          }
        })
        .menu_style(|_t| iced::overlay::menu::Style {
          background: sheet::window_gradient(),
          border: Border {
            radius: radius::R8.into(),
            width: 1.0,
            color: tokens::LINE_2,
          },
          text_color: tokens::FG_2,
          selected_text_color: tokens::FG_1,
          selected_background: Background::Color(tokens::SURFACE_2),
          shadow: iced::Shadow::default(),
        })
        .into(),
    ),
    (
      "Cursor".into(),
      Some("Shape in full-screen apps".into()),
      seg(
        &s.theme.cursor.style,
        &[
          (CursorStyle::Block, "Block"),
          (CursorStyle::Beam, "Beam"),
          (CursorStyle::Underline, "Underline"),
        ],
        Message::SettingsCursorStyleChanged,
      ),
    ),
    (
      "Cursor blink".into(),
      None,
      toggle(s.theme.cursor.blink, Message::SettingsCursorBlinkToggled),
    ),
  ])
}

fn shell<'a>(s: &'a config::Config, shell_input: &'a str) -> Element<'a, Message> {
  let editor = EDITORS
    .iter()
    .find(|e| **e == s.general.editor)
    .copied()
    .unwrap_or("");
  let shells = s.general.shells.clone().unwrap_or_default();
  let shell_list = shells.iter().enumerate().fold(
    Column::new().spacing(space::S4).align_x(Alignment::End),
    |c, (i, sh)| {
      c.push(
        row![
          text(sh.clone()).size(tokens::text::T14).color(tokens::FG_1),
          icon_button(
            Icon::Close,
            IconSize::Sm,
            Some(Message::SettingsRemoveShell(i))
          ),
        ]
        .spacing(space::S8)
        .align_y(Alignment::Center),
      )
    },
  );
  let shell_list = shell_list.push(
    field(
      "Add shell (e.g. pwsh)",
      shell_input,
      Message::SettingsShellInputChanged,
    )
    .on_submit(Message::SettingsAddShell),
  );
  rows(vec![
    (
      "Editor".into(),
      Some("Opened by git commit, crontab -e".into()),
      seg(
        &editor,
        &[("nano", "nano"), ("vim", "vim"), ("code", "code")],
        |e: &str| Message::SettingsEditorChanged(e.to_string()),
      ),
    ),
    (
      "Bell".into(),
      Some("When a command needs attention".into()),
      seg(
        &s.general.bell,
        &[
          (BellType::None, "Off"),
          (BellType::Blink, "Flash"),
          (BellType::Audio, "Sound"),
        ],
        Message::SettingsBellChanged,
      ),
    ),
    (
      "Window controls".into(),
      None,
      seg(
        &s.general.window_controls,
        &[
          (WindowControls::TrafficLights, "Traffic lights"),
          (WindowControls::System, "System"),
        ],
        Message::SettingsWindowControlsChanged,
      ),
    ),
    (
      "Shells".into(),
      Some("Offered by right-clicking +".into()),
      shell_list.into(),
    ),
    (
      "Initial command".into(),
      Some("Runs in every new tab".into()),
      field(
        "none",
        s.general.initial_command.as_deref().unwrap_or(""),
        Message::SettingsInitialCommandChanged,
      )
      .into(),
    ),
    (
      "Keep finished tabs".into(),
      Some("Wait for a key after the initial command exits".into()),
      toggle(
        s.general.wait_after_command,
        Message::SettingsWaitAfterCommandToggled,
      ),
    ),
  ])
}

pub fn key_entries(s: &config::Config) -> Vec<(usize, &'static str, String)> {
  let kb = &s.keybindings;
  vec![
    (0, "New tab", kb.new_tab.clone()),
    (1, "Close tab", kb.close_tab.clone()),
    (2, "Next tab", kb.next_tab.clone()),
    (3, "Previous tab", kb.prev_tab.clone()),
    (4, "Paste", kb.paste.clone()),
    (5, "Copy", kb.copy.clone()),
    (6, "Command palette", kb.open_palette.clone()),
    (7, "Ask AI", kb.ask_ai.clone()),
    (8, "Settings", kb.settings.clone()),
  ]
}

fn keys_tab<'a>(s: &'a config::Config, recording: Option<usize>) -> Element<'a, Message> {
  let accent = color::runtime().accent;
  rows(
    key_entries(s)
      .into_iter()
      .map(|(idx, label, binding)| {
        let chips: Element<'a, Message> = if recording == Some(idx) {
          text("Press keys…")
            .size(tokens::text::T14)
            .color(accent)
            .into()
        } else {
          match config::parse_keybinding(&binding) {
            Ok(kb) => keys(binding_labels(&kb)),
            Err(_) => text(binding)
              .size(tokens::text::T14)
              .color(tokens::RED)
              .into(),
          }
        };
        let control = button(chips)
          .padding(Padding::from([space::S4, space::S8]))
          .style(|_t, st| button::Style {
            background: matches!(st, button::Status::Hovered | button::Status::Pressed)
              .then_some(Background::Color(tokens::SURFACE_1)),
            border: Border {
              radius: radius::R8.into(),
              ..Border::default()
            },
            ..button::Style::default()
          })
          .on_press(if recording == Some(idx) {
            Message::SettingsCancelRecordKb
          } else {
            Message::SettingsStartRecordKb(idx)
          });
        (label.to_string(), None, control.into())
      })
      .collect(),
  )
}

fn ai_tab<'a>(s: &'a config::Config) -> Element<'a, Message> {
  rows(vec![
    (
      "Provider".into(),
      None,
      seg(
        &s.ai.provider,
        &[
          (AiProvider::Anthropic, "Anthropic"),
          (AiProvider::OpenAi, "OpenAI"),
        ],
        Message::SettingsAiProviderChanged,
      ),
    ),
    (
      "Model".into(),
      None,
      field("model id", &s.ai.model, Message::SettingsAiModelChanged).into(),
    ),
    (
      "API key".into(),
      Some("Stored in your config file".into()),
      field(
        "paste your key",
        &s.ai.api_key,
        Message::SettingsAiApiKeyChanged,
      )
      .secure(true)
      .into(),
    ),
    (
      "Base URL".into(),
      Some("Custom endpoint (optional)".into()),
      field(
        "default",
        s.ai.base_url.as_deref().unwrap_or(""),
        Message::SettingsAiBaseUrlChanged,
      )
      .into(),
    ),
    (
      "Auto-diagnose failures".into(),
      Some("Suggest a fix when a command fails".into()),
      toggle(
        s.ai.diagnostic_banner,
        Message::SettingsDiagnosticBannerToggled,
      ),
    ),
  ])
}

fn about<'a>() -> Element<'a, Message> {
  container(
    column![
      image(BRAND_HANDLE.clone())
        .width(240)
        .content_fit(ContentFit::Contain),
      text(format!("Version {}", env!("CARGO_PKG_VERSION")))
        .size(tokens::text::T14)
        .color(tokens::FG_3),
      btn::<Message>(Variant::Ghost)
        .label("Reset to defaults")
        .on_press(Message::SettingsResetAll),
    ]
    .spacing(space::S16)
    .align_x(Alignment::Center),
  )
  .width(Length::Fill)
  .padding(Padding::from([space::S24, 0.0]))
  .center_x(Length::Fill)
  .into()
}

fn raw<'a>(content: &'a str) -> Element<'a, Message> {
  container(
    text(content)
      .size(tokens::text::T14)
      .font(font::regular())
      .color(tokens::FG_2),
  )
  .padding(Padding::from([space::S12, 0.0]))
  .width(Length::Fill)
  .into()
}

pub struct SettingsView<'a> {
  pub settings: &'a config::Config,
  pub tab: &'a SettingsTab,
  pub shell_input: &'a str,
  pub recording: Option<usize>,
  pub raw_content: &'a str,
  pub config_path: String,
  pub timestamps: bool,
  pub progress: f32,
  pub window_height: f32,
}

pub fn settings_modal<'a>(v: SettingsView<'a>) -> Element<'a, Message> {
  let head = sheet::head(row![
    container(
      text("Settings")
        .size(tokens::text::T16)
        .font(font::semibold())
        .color(tokens::FG_1)
    )
    .width(Length::Fill),
    seg(v.tab, &TABS, Message::SettingsTabSelected),
  ]);

  let content: Element<'a, Message> = match v.tab {
    SettingsTab::Appearance => appearance(v.settings, v.timestamps),
    SettingsTab::Shell => shell(v.settings, v.shell_input),
    SettingsTab::Keys => keys_tab(v.settings, v.recording),
    SettingsTab::Ai => ai_tab(v.settings),
    SettingsTab::About => about(),
    SettingsTab::Raw => raw(v.raw_content),
  };
  let max_body = (v.window_height
    - sheet::top_offset(v.window_height)
    - space::S16
    - height::H64
    - height::H56
    - 2.0)
    .max(height::H64);
  let body = container(
    scrollable(container(content).padding(Padding::from([space::S8, space::S24])))
      .height(Length::Shrink),
  )
  .max_height(max_body);

  let path_button = button(
    text(v.config_path)
      .size(tokens::text::T14)
      .font(font::regular())
      .color(tokens::FG_3),
  )
  .padding(0)
  .style(|_t, st| button::Style {
    text_color: if matches!(st, button::Status::Hovered | button::Status::Pressed) {
      tokens::FG_1
    } else {
      tokens::FG_3
    },
    background: Some(Background::Color(Color::TRANSPARENT)),
    ..button::Style::default()
  })
  .on_press(Message::SettingsTabSelected(SettingsTab::Raw));

  let foot = sheet::foot(
    Row::new()
      .push(container(path_button).width(Length::Fill))
      .push(
        btn::<Message>(Variant::Primary)
          .label("Done")
          .on_press(Message::CloseSettings),
      ),
  );

  sheet::sheet(
    Sheet {
      width: sheet::SETTINGS_WIDTH,
      head: Some(head),
      body: body.into(),
      foot: Some(foot),
    },
    v.progress,
    v.window_height,
    Message::CloseSettings,
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn font_steps_by_two_within_range() {
    assert_eq!(step_font(16.0, 2.0), 18.0);
    assert_eq!(step_font(20.0, 2.0), 20.0);
    assert_eq!(step_font(14.0, -2.0), 14.0);
    assert_eq!(step_font(15.0, 2.0), 18.0);
  }

  #[test]
  fn tabs_follow_design_then_extras() {
    let labels: Vec<&str> = TABS.iter().map(|(_, l)| *l).collect();
    assert_eq!(labels, ["Appearance", "Shell", "Keys", "AI", "About"]);
  }
}
