use iced::widget::{Row, container, text};
use iced::{Alignment, Background, Border, Element, Padding};

use crate::core::config::{KeyId, ParsedKeybinding};
use crate::ui::theme::font;
use crate::ui::theme::tokens::{self, height, radius, space};

pub fn kbd<'a, Message: 'a>(label: impl Into<String>) -> Element<'a, Message> {
  container(
    text(label.into())
      .size(tokens::text::T12)
      .font(font::semibold())
      .color(tokens::FG_2),
  )
  .height(height::H24)
  .padding(Padding::from([0.0, space::S8]))
  .align_y(Alignment::Center)
  .center_x(iced::Length::Shrink)
  .style(|_| container::Style {
    background: Some(Background::Color(tokens::SURFACE_2)),
    border: Border {
      radius: radius::R4.into(),
      ..Border::default()
    },
    ..container::Style::default()
  })
  .into()
}

pub fn keys<'a, Message: 'a>(labels: Vec<String>) -> Element<'a, Message> {
  labels
    .into_iter()
    .fold(Row::new().spacing(space::S4), |r, l| r.push(kbd(l)))
    .into()
}

pub const IS_MAC: bool = cfg!(target_os = "macos");

pub fn mod_label() -> &'static str {
  if IS_MAC { "⌘" } else { "Ctrl" }
}

pub fn shortcut_label(key: &str) -> String {
  if IS_MAC {
    format!("⌘{key}")
  } else {
    format!("Ctrl {key}")
  }
}

pub fn ctrl_label() -> &'static str {
  if IS_MAC { "⌃" } else { "Ctrl" }
}

pub fn binding_labels(kb: &ParsedKeybinding) -> Vec<String> {
  let mut out = Vec::new();
  if kb.ctrl {
    out.push(ctrl_label().to_string());
  }
  if kb.meta {
    out.push(mod_label().to_string());
  }
  if kb.alt {
    out.push(if IS_MAC { "⌥" } else { "Alt" }.to_string());
  }
  if kb.shift {
    out.push(if IS_MAC { "⇧" } else { "Shift" }.to_string());
  }
  out.push(key_label(&kb.key));
  out
}

fn key_label(key: &KeyId) -> String {
  match key {
    KeyId::Tab => "Tab".to_string(),
    KeyId::Char('-') => "−".to_string(),
    KeyId::Char(c) => c.to_uppercase().collect(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::core::config::parse_keybinding;

  #[test]
  fn labels_follow_modifier_order() {
    let kb = parse_keybinding("ctrl+shift+t").unwrap();
    let l = binding_labels(&kb);
    assert_eq!(l.last().unwrap(), "T");
    assert_eq!(l.len(), 3);
  }

  #[test]
  fn minus_uses_typographic_sign() {
    let kb = parse_keybinding("ctrl+-").unwrap();
    assert_eq!(binding_labels(&kb).last().unwrap(), "−");
  }
}
