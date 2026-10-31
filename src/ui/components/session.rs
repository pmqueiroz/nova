use std::time::Instant;

use iced::widget::scrollable::Viewport;
use iced::widget::{Column, container, mouse_area, rich_text, scrollable, text::Span};
use iced::{Alignment, Element, Length, Padding};

use crate::core::block::Session;
use crate::ui::app_state::Message;
use crate::ui::components::block::{self, EntryOptions};
use crate::ui::theme::font;
use crate::ui::theme::tokens::{self, space};

pub const PIN_THRESHOLD: f32 = space::S32;

pub fn scroll_id(pane_id: usize) -> iced::widget::Id {
  iced::widget::Id::from(format!("nova-blocks-{pane_id}"))
}

pub fn input_id(pane_id: usize) -> iced::widget::Id {
  iced::widget::Id::from(format!("nova-input-{pane_id}"))
}

pub fn is_pinned(viewport: &Viewport) -> bool {
  let content = viewport.content_bounds().height;
  let visible = viewport.bounds().height;
  let offset = viewport.absolute_offset().y;
  content - offset - visible < PIN_THRESHOLD
}

pub fn scroll_padding(compact: bool) -> Padding {
  if compact {
    Padding {
      top: space::S8,
      right: space::S16,
      bottom: space::S16,
      left: space::S16,
    }
  } else {
    Padding {
      top: space::S16,
      right: space::S32,
      bottom: space::S24,
      left: space::S32,
    }
  }
}

pub fn empty_state<'a>(alpha: f32) -> Element<'a, Message> {
  let dim = tokens::with_alpha(tokens::FG_3, alpha);
  let msg = rich_text::<(), Message, iced::Theme, iced::Renderer>([
    Span::new("Type ").color(dim).font(font::regular()),
    Span::new("help")
      .color(tokens::with_alpha(tokens::FG_1, alpha))
      .font(font::italic()),
    Span::new(" to see what Nova can do")
      .color(dim)
      .font(font::regular()),
  ])
  .size(tokens::text::T16);
  container(msg)
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
    .into()
}

pub struct SessionView<'a> {
  pub pane_id: usize,
  pub session: &'a Session,
  pub font_size: f32,
  pub timestamps: bool,
  pub now: Instant,
  pub shown_at: Instant,
  pub compact: bool,
}

pub fn blocks_area<'a>(v: &SessionView<'a>) -> Element<'a, Message> {
  if v.session.blocks.is_empty() {
    let fade = (v.now.saturating_duration_since(v.shown_at).as_secs_f32()
      / tokens::duration::D4.as_secs_f32())
    .clamp(0.0, 1.0);
    return mouse_area(empty_state(crate::ui::theme::motion::ease(fade)))
      .on_press(Message::FocusInput)
      .into();
  }
  let opts = EntryOptions {
    font_size: v.font_size,
    timestamps: v.timestamps,
    now: v.now,
  };
  let log = v
    .session
    .blocks
    .iter()
    .fold(Column::new().spacing(space::S8), |c, b| {
      c.push(block::block_entry(b, &opts))
    });
  let pane_id = v.pane_id;
  mouse_area(
    scrollable(
      container(log)
        .width(Length::Fill)
        .padding(scroll_padding(v.compact)),
    )
    .id(scroll_id(pane_id))
    .on_scroll(move |vp| Message::BlocksScrolled(pane_id, is_pinned(&vp)))
    .width(Length::Fill)
    .height(Length::Fill),
  )
  .on_press(Message::FocusInput)
  .into()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn ids_are_per_pane() {
    assert_ne!(scroll_id(1), scroll_id(2));
    assert_ne!(scroll_id(1), input_id(1));
  }

  #[test]
  fn compact_padding_is_tighter() {
    assert!(scroll_padding(true).left < scroll_padding(false).left);
  }
}
