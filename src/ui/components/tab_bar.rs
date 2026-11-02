use iced::widget::{Row, Space, button, container, mouse_area, row, scrollable, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding, mouse};

use crate::ui::app_state::Message;
use crate::ui::components::ds::seg::track_style;
use crate::ui::components::ds::transform::Transform;
use crate::ui::helpers::dir_name;
use crate::ui::icons::{Icon, icon};
use crate::ui::tab::Tab;
use crate::ui::theme::motion::lerp;
use crate::ui::theme::tokens::{self, height, radius, space};

pub const TAB_MIN_WIDTH: f32 = 96.0;
pub const TRACK_MAX_WIDTH: f32 = 560.0;

pub fn tab_width(label: &str) -> f32 {
  let text_w = label.chars().count() as f32 * tokens::text::T14 * tokens::block::CHAR_WIDTH_RATIO;
  (space::S12 + text_w + space::S8 + height::H24 + space::S8)
    .ceil()
    .max(TAB_MIN_WIDTH)
}

pub fn track_max_width(window_width: f32) -> f32 {
  TRACK_MAX_WIDTH.min(window_width / 2.0)
}

pub fn tab_style(active: bool) -> container::Style {
  container::Style {
    background: active.then_some(Background::Color(tokens::SURFACE_2)),
    border: Border {
      radius: radius::R8.into(),
      ..Border::default()
    },
    ..container::Style::default()
  }
}

pub fn tab_text_color(active: bool, hovered: bool) -> Color {
  if active || hovered {
    tokens::FG_1
  } else {
    tokens::FG_3
  }
}

fn close_button<'a>(index: usize, visible: bool) -> Element<'a, Message> {
  let glyph: Element<'a, Message> = if visible {
    icon(Icon::Close, 12.0, tokens::FG_2).into()
  } else {
    Space::new().width(12).height(12).into()
  };
  button(
    container(glyph)
      .center_x(Length::Fill)
      .center_y(Length::Fill),
  )
  .width(height::H24)
  .height(height::H24)
  .padding(0)
  .on_press_maybe(visible.then_some(Message::CloseTab(index)))
  .style(move |_t, s| button::Style {
    background: (visible && matches!(s, button::Status::Hovered | button::Status::Pressed))
      .then_some(Background::Color(tokens::SURFACE_3)),
    border: Border {
      radius: radius::R4.into(),
      ..Border::default()
    },
    ..button::Style::default()
  })
  .into()
}

pub struct TabView<'a> {
  pub tabs: &'a [Tab],
  pub active: usize,
  pub hovered: Option<usize>,
  pub dragging: Option<usize>,
  pub now: std::time::Instant,
  pub window_width: f32,
}

pub fn tab_bar<'a>(v: TabView<'a>) -> Element<'a, Message> {
  let items = v
    .tabs
    .iter()
    .enumerate()
    .fold(Row::new().spacing(space::S4), |r, (i, tab)| {
      let active = i == v.active;
      let hovered = v.hovered == Some(i);
      let title = dir_name(&tab.pwd);
      let label = text(title.clone())
        .size(tokens::text::T14)
        .color(tab_text_color(active, hovered))
        .wrapping(text::Wrapping::None);
      let body = container(
        row![
          container(label).width(Length::Fill),
          close_button(i, active || hovered),
        ]
        .spacing(space::S8)
        .align_y(Alignment::Center),
      )
      .height(height::H32)
      .padding(Padding {
        top: 0.0,
        right: space::S8,
        bottom: 0.0,
        left: space::S12,
      })
      .center_y(height::H32)
      .style(move |_| tab_style(active));
      let sized = container(body).width(tab_width(&title));
      let appear = tab.appear.interpolate(0.0, 1.0, v.now);
      let popped = Transform::new(sized)
        .scale(lerp(0.96, 1.0, appear))
        .offset(0.0, lerp(space::S8, 0.0, appear))
        .press();
      let area = mouse_area(popped)
        .on_press(Message::TabPressed(i))
        .on_middle_press(Message::CloseTab(i))
        .on_enter(Message::TabHovered(Some(i)))
        .on_exit(Message::TabHovered(None))
        .interaction(if v.dragging == Some(i) {
          mouse::Interaction::Grabbing
        } else {
          mouse::Interaction::Pointer
        });
      r.push(container(area).width(Length::Shrink))
    });

  container(
    scrollable(items)
      .direction(scrollable::Direction::Horizontal(
        scrollable::Scrollbar::hidden(),
      ))
      .width(Length::Shrink),
  )
  .padding(space::S4)
  .max_width(track_max_width(v.window_width))
  .style(|_| track_style())
  .into()
}

pub fn reorder(tabs_len: usize, from: usize, to: usize) -> Option<(usize, usize)> {
  (from != to && from < tabs_len && to < tabs_len).then_some((from, to))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn tab_width_has_floor() {
    assert_eq!(tab_width("~"), TAB_MIN_WIDTH);
    assert!(tab_width("~/projects/nova-terminal") > TAB_MIN_WIDTH);
  }

  #[test]
  fn track_is_capped() {
    assert_eq!(track_max_width(2000.0), 560.0);
    assert_eq!(track_max_width(800.0), 400.0);
  }

  #[test]
  fn active_tab_is_raised() {
    assert_eq!(
      tab_style(true).background,
      Some(Background::Color(tokens::SURFACE_2))
    );
    assert_eq!(tab_style(false).background, None);
  }

  #[test]
  fn text_brightens_on_hover() {
    assert_eq!(tab_text_color(false, false), tokens::FG_3);
    assert_eq!(tab_text_color(false, true), tokens::FG_1);
  }

  #[test]
  fn reorder_rejects_noop_and_out_of_range() {
    assert_eq!(reorder(3, 1, 1), None);
    assert_eq!(reorder(3, 0, 5), None);
    assert_eq!(reorder(3, 0, 2), Some((0, 2)));
  }
}
