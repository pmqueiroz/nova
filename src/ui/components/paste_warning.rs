use iced::widget::{Column, Row, container, row, scrollable, text};
use iced::{Background, Border, Element, Length, Padding};

use crate::ui::app_state::Message;
use crate::ui::components::ds::button::{Variant, btn};
use crate::ui::components::ds::sheet::{self, Sheet};
use crate::ui::icons::{Icon, icon};
use crate::ui::theme::font;
use crate::ui::theme::tokens::{self, radius, space};

pub const PREVIEW_MAX_LINES: usize = 8;

pub fn paste_title(content: &str) -> &'static str {
  if content.contains('\n') {
    "Paste multiple lines?"
  } else {
    "Paste this command?"
  }
}

pub fn paste_reason(content: &str) -> &'static str {
  if content.contains('\n') {
    "Each line may run as a separate command."
  } else {
    "It looks like it could change or delete files."
  }
}

pub fn preview_lines(content: &str) -> (Vec<&str>, usize) {
  let all: Vec<&str> = content.lines().collect();
  let extra = all.len().saturating_sub(PREVIEW_MAX_LINES);
  (all.into_iter().take(PREVIEW_MAX_LINES).collect(), extra)
}

pub fn paste_warning_overlay<'a>(
  content: &'a str,
  progress: f32,
  window_height: f32,
) -> Element<'a, Message> {
  let head = sheet::head(row![
    icon(Icon::Alert, 20.0, tokens::RED),
    container(
      text(paste_title(content))
        .size(tokens::text::T16)
        .font(font::semibold())
        .color(tokens::FG_1)
    )
    .width(Length::Fill),
  ]);

  let (lines, extra) = preview_lines(content);
  let mut preview = lines.into_iter().fold(Column::new(), |c, l| {
    c.push(
      text(l.to_string())
        .size(tokens::text::T14)
        .font(font::regular())
        .color(tokens::FG_1),
    )
  });
  if extra > 0 {
    preview = preview.push(
      text(format!("+{extra} more lines"))
        .size(tokens::text::T12)
        .color(tokens::FG_3),
    );
  }

  let body = container(
    Column::new()
      .spacing(space::S16)
      .push(
        text(paste_reason(content))
          .size(tokens::text::T14)
          .color(tokens::FG_3),
      )
      .push(
        container(scrollable(preview).height(Length::Shrink))
          .width(Length::Fill)
          .max_height(240)
          .padding(Padding::from([space::S12, space::S16]))
          .style(|_| container::Style {
            background: Some(Background::Color(tokens::SURFACE_1)),
            border: Border {
              radius: radius::R8.into(),
              ..Border::default()
            },
            ..container::Style::default()
          }),
      ),
  )
  .padding(space::S24)
  .width(Length::Fill);

  let foot = sheet::foot(
    Row::new()
      .push(container(text("")).width(Length::Fill))
      .push(
        btn::<Message>(Variant::Ghost)
          .label("Cancel")
          .on_press(Message::PasteWarningCancel),
      )
      .push(
        btn::<Message>(Variant::Primary)
          .label("Paste")
          .on_press(Message::PasteWarningConfirm),
      ),
  );

  sheet::sheet(
    Sheet {
      width: sheet::WIDTH,
      head: Some(head),
      body: body.into(),
      foot: Some(foot),
    },
    progress,
    window_height,
    Message::PasteWarningCancel,
  )
}

pub fn join_for_input(text: &str) -> String {
  text
    .lines()
    .map(str::trim)
    .filter(|l| !l.is_empty())
    .collect::<Vec<_>>()
    .join("; ")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn titles_depend_on_lines() {
    assert_eq!(paste_title("a\nb"), "Paste multiple lines?");
    assert_eq!(paste_title("rm -rf x"), "Paste this command?");
  }

  #[test]
  fn preview_caps_lines() {
    let text = (0..12)
      .map(|i| i.to_string())
      .collect::<Vec<_>>()
      .join("\n");
    let (lines, extra) = preview_lines(&text);
    assert_eq!(lines.len(), 8);
    assert_eq!(extra, 4);
  }

  #[test]
  fn multi_line_paste_joins_for_single_line_input() {
    assert_eq!(join_for_input("cd x\r\n\nls\n"), "cd x; ls");
  }
}
