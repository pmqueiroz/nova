use std::time::{Duration, Instant};

use iced::widget::text::{Span, Wrapping};
use iced::widget::{Column, Row, column, container, mouse_area, rich_text, row, text};
use iced::{Alignment, Background, Border, Color, Element, Font, Length, Padding};

use crate::core::block::{Block, BlockId, BlockState};
use crate::core::grid::{Cell, CellAttrs};
use crate::core::search::BlockMatch;
use crate::core::selection::BlockSelection;
use crate::ui::app_state::Message;
use crate::ui::components::ds::transform::Transform;
use crate::ui::theme::motion::{self, lerp};
use crate::ui::theme::tokens::{self, duration, radius, space};
use crate::ui::theme::{color, font};

pub const TIMESTAMP_WIDTH: f32 = space::S48;
pub const OUTPUT_DELAY: Duration = Duration::from_millis(60);

#[derive(Clone, Copy, PartialEq)]
struct Style {
  fg: Color,
  bg: Option<Color>,
  bold: bool,
  italic: bool,
  underline: bool,
  strike: bool,
}

fn cell_style(cell: &Cell, default_fg: Color) -> Style {
  let a = cell.attrs;
  let mut fg = cell.fg.unwrap_or(default_fg);
  let mut bg = cell.bg;
  if a.contains(CellAttrs::REVERSE) {
    let new_bg = fg;
    fg = bg.unwrap_or(tokens::WIN_BOT);
    bg = Some(new_bg);
  }
  if a.contains(CellAttrs::DIM) {
    fg.a *= 0.5;
  }
  Style {
    fg,
    bg,
    bold: a.contains(CellAttrs::BOLD),
    italic: a.contains(CellAttrs::ITALIC),
    underline: a.contains(CellAttrs::UNDERLINE),
    strike: a.contains(CellAttrs::STRIKETHROUGH),
  }
}

fn style_font(st: &Style) -> Font {
  match (st.bold, st.italic) {
    (true, true) => font::bold_italic(),
    (true, false) => font::bold(),
    (false, true) => font::italic(),
    (false, false) => font::regular(),
  }
}

pub fn visible_len(cells: &[Cell]) -> usize {
  cells
    .iter()
    .rposition(|c| !c.c.as_ref().trim().is_empty() || c.bg.is_some())
    .map(|i| i + 1)
    .unwrap_or(0)
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct LineMarks {
  pub selection: Option<(usize, usize)>,
  pub url: Option<(usize, usize)>,
  pub matches: Vec<(usize, usize, bool)>,
}

pub fn runs(
  cells: &[Cell],
  default_fg: Color,
  marks: LineMarks,
) -> Vec<(String, Color, Option<Color>, u8)> {
  let accent = color::runtime().accent;
  let mut out: Vec<(String, Style)> = Vec::new();
  let len = visible_len(cells).max(marks.selection.map_or(0, |(_, e)| e.min(cells.len())));
  for (x, cell) in cells[..len].iter().enumerate() {
    if cell.width == 0 {
      continue;
    }
    let mut st = cell_style(cell, default_fg);
    if let Some(&(_, _, current)) = marks.matches.iter().find(|(a, b, _)| x >= *a && x < *b) {
      if current {
        st.fg = tokens::ON_ACCENT;
        st.bg = Some(tokens::YELLOW);
      } else {
        st.bg = Some(tokens::with_alpha(tokens::YELLOW, 0.32));
      }
    }
    if marks.selection.is_some_and(|(a, b)| x >= a && x < b) {
      st.fg = tokens::FG_1;
      st.bg = Some(tokens::with_alpha(accent, 0.32));
    }
    if marks.url.is_some_and(|(a, b)| x >= a && x <= b) {
      st.underline = true;
    }
    match out.last_mut() {
      Some((t, s)) if *s == st => t.push_str(&cell.c),
      _ => out.push((cell.c.to_string(), st)),
    }
  }
  out
    .into_iter()
    .map(|(t, s)| {
      let flags = u8::from(s.bold)
        | (u8::from(s.italic) << 1)
        | (u8::from(s.underline) << 2)
        | (u8::from(s.strike) << 3);
      (t, s.fg, s.bg, flags)
    })
    .collect()
}

pub fn line<'a>(
  cells: &[Cell],
  default_fg: Color,
  size: f32,
  alpha: f32,
  marks: LineMarks,
) -> Element<'a, Message> {
  let line_h = size * tokens::block::LINE_HEIGHT;
  let spans: Vec<Span<'a>> = runs(cells, default_fg, marks)
    .into_iter()
    .map(|(t, fg, bg, flags)| {
      let st = Style {
        fg,
        bg,
        bold: flags & 1 != 0,
        italic: flags & 2 != 0,
        underline: flags & 4 != 0,
        strike: flags & 8 != 0,
      };
      Span::new(t)
        .color(tokens::with_alpha(fg, alpha))
        .background_maybe(bg.map(|b| Background::Color(tokens::with_alpha(b, alpha))))
        .font(style_font(&st))
        .size(size)
        .underline(st.underline)
        .strikethrough(st.strike)
    })
    .collect();
  container(
    rich_text::<(), Message, iced::Theme, iced::Renderer>(spans)
      .size(size)
      .line_height(tokens::block::LINE_HEIGHT)
      .wrapping(Wrapping::None),
  )
  .height(line_h)
  .into()
}

pub fn timestamp<'a>(block: &Block, font_size: f32) -> Element<'a, Message> {
  container(
    text(block.started_at.format("%H:%M").to_string())
      .size(tokens::text::T12)
      .color(tokens::FG_3),
  )
  .width(TIMESTAMP_WIDTH)
  .height(font_size * tokens::block::LINE_HEIGHT)
  .center_y(font_size * tokens::block::LINE_HEIGHT)
  .into()
}

pub fn command_line<'a>(block: &Block, font_size: f32, alpha: f32) -> Element<'a, Message> {
  let (name, rest) = block.split_command();
  let fg = tokens::with_alpha(tokens::FG_1, alpha);
  rich_text::<(), Message, iced::Theme, iced::Renderer>([
    Span::new(name.to_string())
      .font(font::semibold())
      .color(fg)
      .size(font_size),
    Span::new(rest.to_string())
      .font(font::regular())
      .color(fg)
      .size(font_size),
  ])
  .size(font_size)
  .line_height(tokens::block::LINE_HEIGHT)
  .wrapping(Wrapping::WordOrGlyph)
  .width(Length::Fill)
  .into()
}

pub fn card_style(alpha: f32) -> container::Style {
  container::Style {
    background: Some(Background::Color(tokens::with_alpha(
      tokens::SURFACE_1,
      alpha,
    ))),
    border: Border {
      radius: radius::R8.into(),
      ..Border::default()
    },
    ..container::Style::default()
  }
}

pub fn card_padding(compact: bool) -> Padding {
  if compact {
    Padding::from([space::S12, space::S16])
  } else {
    Padding::from([space::S16, space::S24])
  }
}

pub fn dots<'a>(born: Instant, now: Instant) -> Element<'a, Message> {
  let accent = color::runtime().accent;
  [0u64, 120, 240]
    .into_iter()
    .fold(Row::new().spacing(space::S4), |r, delay| {
      let (opacity, y) = motion::dot_phase(born, now, delay);
      let dot = container(iced::widget::Space::new())
        .width(6)
        .height(6)
        .style(move |_| container::Style {
          background: Some(Background::Color(tokens::with_alpha(accent, opacity))),
          border: Border {
            radius: 3.0.into(),
            ..Border::default()
          },
          ..container::Style::default()
        });
      r.push(Transform::new(dot).offset(0.0, y))
    })
    .align_y(Alignment::Center)
    .into()
}

pub fn pending_card<'a>(
  born: Instant,
  now: Instant,
  alpha: f32,
  compact: bool,
) -> Element<'a, Message> {
  container(
    row![
      dots(born, now),
      text("Running")
        .size(tokens::text::T14)
        .color(tokens::with_alpha(tokens::FG_3, alpha)),
    ]
    .spacing(space::S12)
    .align_y(Alignment::Center),
  )
  .width(Length::Fill)
  .padding(card_padding(compact))
  .style(move |_| card_style(alpha))
  .into()
}

pub fn default_output_color(block: &Block) -> Color {
  if block.is_error() {
    tokens::RED
  } else {
    tokens::FG_2
  }
}

pub fn output_card<'a>(
  block: &Block,
  size: f32,
  alpha: f32,
  o: &EntryOptions<'_>,
) -> Element<'a, Message> {
  let fg = default_output_color(block);
  let sel = o.selection.filter(|s| s.block_id == block.id);
  let url = o.hover_url.filter(|(id, ..)| *id == block.id);
  let lines = (0..block.line_count()).fold(Column::new(), |c, i| match block.line(i) {
    Some(cells) => {
      let marks = LineMarks {
        selection: sel.and_then(|s| s.line_range(i, visible_len(cells))),
        url: url.filter(|(_, l, ..)| *l == i).map(|(_, _, a, b)| (a, b)),
        matches: o
          .matches
          .iter()
          .enumerate()
          .filter(|(_, m)| m.block_id == block.id && m.line == i)
          .map(|(k, m)| (m.start, m.end, o.current_match == Some(k)))
          .collect(),
      };
      c.push(line(cells, fg, size, alpha, marks))
    }
    None => c,
  });
  let card = container(lines)
    .width(Length::Fill)
    .padding(card_padding(o.compact))
    .clip(true)
    .style(move |_| card_style(alpha));
  let (pane_id, block_id) = (o.pane_id, block.id);
  mouse_area(card)
    .on_move(move |p| Message::BlockPointer(pane_id, block_id, p))
    .on_press(Message::BlockPress(pane_id, block_id))
    .on_release(Message::BlockRelease)
    .interaction(if url.is_some() {
      iced::mouse::Interaction::Pointer
    } else {
      iced::mouse::Interaction::Text
    })
    .into()
}

pub fn entry_progress(born: Instant, now: Instant, delay: Duration) -> f32 {
  let elapsed = now.saturating_duration_since(born);
  if elapsed <= delay {
    return 0.0;
  }
  let t = (elapsed - delay).as_secs_f32() / duration::D2.as_secs_f32();
  motion::ease(t.clamp(0.0, 1.0))
}

pub fn is_animating(block: &Block, now: Instant) -> bool {
  let showing_dots = block.state == BlockState::Pending
    || (block.state == BlockState::Running && !block.has_output());
  showing_dots || now.saturating_duration_since(block.born) < duration::D2 + OUTPUT_DELAY
}

pub struct EntryOptions<'a> {
  pub pane_id: usize,
  pub font_size: f32,
  pub timestamps: bool,
  pub now: Instant,
  pub selection: Option<BlockSelection>,
  pub hover_url: Option<(BlockId, usize, usize, usize)>,
  pub matches: &'a [BlockMatch],
  pub current_match: Option<usize>,
  pub compact: bool,
}

pub fn block_entry<'a>(block: &Block, o: &EntryOptions<'_>) -> Element<'a, Message> {
  let t_in = entry_progress(block.born, o.now, Duration::ZERO);
  let t_out = entry_progress(block.born, o.now, OUTPUT_DELAY);
  let out_size = tokens::block::font_size(o.font_size);

  let pending = block.state == BlockState::Pending
    || (block.state == BlockState::Running && !block.has_output());
  let output: Option<Element<'a, Message>> = if pending {
    Some(pending_card(block.born, o.now, t_out, o.compact))
  } else if block.has_output() {
    Some(output_card(block, out_size, t_out, o))
  } else {
    None
  };
  let output =
    output.map(|e| Element::from(Transform::new(e).offset(0.0, lerp(space::S8, 0.0, t_out))));

  let mut body = column![command_line(block, o.font_size, t_in)].spacing(space::S8);
  if let Some(out) = output {
    body = body.push(out);
  }

  let content: Element<'a, Message> = if o.timestamps {
    row![timestamp(block, o.font_size), body]
      .spacing(space::S16)
      .into()
  } else {
    body.into()
  };

  let has_out = pending || block.has_output();
  let entry = container(content).padding(Padding {
    bottom: if has_out { space::S8 } else { 0.0 },
    ..Padding::default()
  });
  Transform::new(entry)
    .offset(0.0, lerp(space::S8, 0.0, t_in))
    .into()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::core::block::Session;
  use crate::sys::parser::AnsiExecutor;
  use vte::Parser;

  fn block_with(output: &str) -> Block {
    let mut s = Session::new("~".into(), 20, 5);
    let id = s.submit("echo hi there").unwrap();
    let mut b = s
      .blocks
      .remove(s.blocks.iter().position(|b| b.id == id).unwrap());
    let mut p = Parser::new();
    let mut exec = AnsiExecutor {
      grid: &mut b.output,
      bell_pending: false,
    };
    p.advance(&mut exec, output.as_bytes());
    b
  }

  #[test]
  fn runs_merge_same_style_and_trim_trailing_space() {
    let b = block_with("ab\x1b[31mcd\x1b[0m  ");
    let r = runs(b.line(0).unwrap(), tokens::FG_2, LineMarks::default());
    assert_eq!(r.len(), 2);
    assert_eq!(r[0].0, "ab");
    assert_eq!(r[0].1, tokens::FG_2);
    assert_eq!(r[1].0, "cd");
    assert_eq!(r[1].1, tokens::RED);
  }

  #[test]
  fn selection_and_url_marks_restyle_cells() {
    let b = block_with("abcdef");
    let r = runs(
      b.line(0).unwrap(),
      tokens::FG_2,
      LineMarks {
        selection: Some((1, 3)),
        ..LineMarks::default()
      },
    );
    assert_eq!(r.len(), 3);
    assert_eq!(r[1].0, "bc");
    assert_eq!(r[1].1, tokens::FG_1);
    assert!(r[1].2.is_some());
    let u = runs(
      b.line(0).unwrap(),
      tokens::FG_2,
      LineMarks {
        url: Some((0, 1)),
        ..LineMarks::default()
      },
    );
    assert_eq!(u[0].0, "ab");
    assert_eq!(u[0].3 & 4, 4);
  }

  #[test]
  fn current_match_is_solid_others_tinted() {
    let b = block_with("abab");
    let r = runs(
      b.line(0).unwrap(),
      tokens::FG_2,
      LineMarks {
        matches: vec![(0, 2, true), (2, 4, false)],
        ..LineMarks::default()
      },
    );
    assert_eq!(r[0].2, Some(tokens::YELLOW));
    assert_eq!(r[1].2, Some(tokens::with_alpha(tokens::YELLOW, 0.32)));
  }

  #[test]
  fn reverse_swaps_colors() {
    let b = block_with("\x1b[7mX");
    let r = runs(b.line(0).unwrap(), tokens::FG_2, LineMarks::default());
    assert_eq!(r[0].1, tokens::WIN_BOT);
    assert_eq!(r[0].2, Some(tokens::FG_2));
  }

  #[test]
  fn error_blocks_default_to_red() {
    let mut b = block_with("boom");
    b.exit_code = Some(127);
    assert_eq!(default_output_color(&b), tokens::RED);
    b.exit_code = Some(0);
    assert_eq!(default_output_color(&b), tokens::FG_2);
  }

  #[test]
  fn output_waits_for_delay() {
    let t0 = Instant::now();
    assert_eq!(
      entry_progress(t0, t0 + Duration::from_millis(30), OUTPUT_DELAY),
      0.0
    );
    assert_eq!(
      entry_progress(t0, t0 + Duration::from_millis(400), OUTPUT_DELAY),
      1.0
    );
  }
}
