use std::sync::LazyLock;

use iced::widget::{Column, Id, container, row, text, text_input};
use iced::{Background, Border, Color, Element, Length, Padding};

use crate::ui::app_state::Message;
use crate::ui::components::ds::kbd::{IS_MAC, keys};
use crate::ui::components::ds::list_item::{ListItem, Tone, list_item, list_item_style};
use crate::ui::components::ds::sheet::{self, Sheet};
use crate::ui::icons::{Icon, icon};
use crate::ui::theme::font;
use crate::ui::theme::tokens::{self, height, space};

pub static PALETTE_INPUT_ID: LazyLock<Id> = LazyLock::new(Id::unique);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteAction {
  NewTab,
  CloseTab,
  Clear,
  AskAi,
  FontUp,
  FontDown,
  ToggleTimestamps,
  Settings,
  SplitPane,
  Find,
}

pub struct PaletteItem {
  pub action: PaletteAction,
  pub label: &'static str,
  pub icon: Icon,
  pub keys: &'static [&'static str],
}

fn chord(key: &'static str) -> &'static [&'static str] {
  match (IS_MAC, key) {
    (true, "T") => &["⌘", "T"],
    (true, "W") => &["⌘", "W"],
    (true, "K") => &["⌘", "K"],
    (true, "+") => &["⌘", "+"],
    (true, "−") => &["⌘", "−"],
    (true, ",") => &["⌘", ","],
    (true, "F") => &["⌘", "F"],
    (true, "split") => &["⌘", "⇧", "T"],
    (true, "L") => &["⌃", "L"],
    (false, "T") => &["Ctrl", "T"],
    (false, "W") => &["Ctrl", "W"],
    (false, "K") => &["Ctrl", "K"],
    (false, "+") => &["Ctrl", "+"],
    (false, "−") => &["Ctrl", "−"],
    (false, ",") => &["Ctrl", ","],
    (false, "F") => &["Ctrl", "F"],
    (false, "split") => &["Ctrl", "Shift", "T"],
    (false, "L") => &["Ctrl", "L"],
    _ => &[],
  }
}

pub fn items() -> Vec<PaletteItem> {
  use PaletteAction::*;
  vec![
    PaletteItem {
      action: NewTab,
      label: "New tab",
      icon: Icon::Plus,
      keys: chord("T"),
    },
    PaletteItem {
      action: CloseTab,
      label: "Close tab",
      icon: Icon::Close,
      keys: chord("W"),
    },
    PaletteItem {
      action: Clear,
      label: "Clear screen",
      icon: Icon::Clear,
      keys: chord("L"),
    },
    PaletteItem {
      action: AskAi,
      label: "Ask AI",
      icon: Icon::Sparkle,
      keys: chord("K"),
    },
    PaletteItem {
      action: FontUp,
      label: "Larger text",
      icon: Icon::ZoomIn,
      keys: chord("+"),
    },
    PaletteItem {
      action: FontDown,
      label: "Smaller text",
      icon: Icon::ZoomOut,
      keys: chord("−"),
    },
    PaletteItem {
      action: ToggleTimestamps,
      label: "Toggle timestamps",
      icon: Icon::Clock,
      keys: &[],
    },
    PaletteItem {
      action: Settings,
      label: "Settings",
      icon: Icon::Sliders,
      keys: chord(","),
    },
    PaletteItem {
      action: SplitPane,
      label: "Split pane",
      icon: Icon::Terminal,
      keys: chord("split"),
    },
    PaletteItem {
      action: Find,
      label: "Find",
      icon: Icon::Search,
      keys: chord("F"),
    },
  ]
}

pub fn filtered(query: &str) -> Vec<PaletteItem> {
  let q = query.to_lowercase();
  items()
    .into_iter()
    .filter(|i| i.label.to_lowercase().contains(&q))
    .collect()
}

pub fn palette_filtered_count(query: &str) -> usize {
  filtered(query).len()
}

pub fn palette_action_at(query: &str, selected: usize) -> Option<PaletteAction> {
  filtered(query).get(selected).map(|i| i.action)
}

pub fn clamp_step(selected: usize, delta: i32, count: usize) -> usize {
  if count == 0 {
    return 0;
  }
  (selected as i32 + delta).clamp(0, count as i32 - 1) as usize
}

pub fn sheet_input_style(_status: text_input::Status) -> text_input::Style {
  let accent = crate::ui::theme::color::runtime().accent;
  text_input::Style {
    background: Background::Color(Color::TRANSPARENT),
    border: Border::default(),
    icon: tokens::FG_3,
    placeholder: tokens::FG_3,
    value: tokens::FG_1,
    selection: tokens::with_alpha(accent, 0.32),
  }
}

pub fn command_palette<'a>(
  query: &'a str,
  selected: usize,
  progress: f32,
  window_height: f32,
) -> Element<'a, Message> {
  let head = sheet::head(row![
    icon(Icon::Search, 20.0, tokens::FG_3),
    text_input("Search commands", query)
      .id(PALETTE_INPUT_ID.clone())
      .on_input(Message::PaletteQueryChanged)
      .on_submit(Message::PaletteConfirm)
      .size(tokens::text::T18)
      .font(font::regular())
      .padding(0)
      .width(Length::Fill)
      .style(|_t, s| sheet_input_style(s)),
  ]);

  let list = filtered(query);
  let body: Element<'a, Message> = if list.is_empty() {
    container(
      container(
        text(format!("Nothing matches “{query}”"))
          .size(tokens::text::T16)
          .color(tokens::FG_2),
      )
      .height(height::H48)
      .padding(Padding::from([0.0, space::S16]))
      .center_y(height::H48)
      .width(Length::Fill)
      .style(|_| {
        let b = list_item_style(false);
        container::Style {
          background: b.background,
          border: b.border,
          ..container::Style::default()
        }
      }),
    )
    .padding(space::S8)
    .into()
  } else {
    let col = list
      .into_iter()
      .enumerate()
      .fold(Column::new().spacing(space::S2), |c, (i, it)| {
        let k: Option<Element<'a, Message>> =
          (!it.keys.is_empty()).then(|| keys(it.keys.iter().map(|s| s.to_string()).collect()));
        c.push(list_item(
          ListItem {
            icon: it.icon,
            label: it.label.to_string(),
            hint: None,
            keys: k,
            tone: Tone::Normal,
          },
          i == selected,
          Message::PaletteHover(i),
          Message::PaletteSelectAndConfirm(i),
        ))
      });
    container(col).padding(space::S8).into()
  };

  sheet::sheet(
    Sheet {
      width: sheet::WIDTH,
      head: Some(head),
      body,
      foot: None,
    },
    progress,
    window_height,
    Message::CloseCommandPalette,
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn design_order_then_extras() {
    let labels: Vec<&str> = items().iter().map(|i| i.label).collect();
    assert_eq!(
      &labels[..8],
      &[
        "New tab",
        "Close tab",
        "Clear screen",
        "Ask AI",
        "Larger text",
        "Smaller text",
        "Toggle timestamps",
        "Settings"
      ]
    );
  }

  #[test]
  fn substring_filter_is_case_insensitive() {
    let f = filtered("TAB");
    assert_eq!(f.len(), 2);
    assert!(filtered("zzz").is_empty());
    assert_eq!(palette_action_at("set", 0), Some(PaletteAction::Settings));
  }

  #[test]
  fn navigation_clamps() {
    assert_eq!(clamp_step(0, -1, 5), 0);
    assert_eq!(clamp_step(4, 1, 5), 4);
    assert_eq!(clamp_step(2, 1, 5), 3);
    assert_eq!(clamp_step(0, 1, 0), 0);
  }
}
