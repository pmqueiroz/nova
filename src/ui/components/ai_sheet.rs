use std::sync::LazyLock;
use std::time::{Duration, Instant};

use iced::widget::text::Span;
use iced::widget::{Column, Id, Row, container, rich_text, row, text, text_input};
use iced::{Alignment, Background, Border, Element, Length, Padding};

use crate::ui::app_state::Message;
use crate::ui::components::block::dots;
use crate::ui::components::command_palette::sheet_input_style;
use crate::ui::components::ds::button::{IconSize, Variant, btn, icon_button};
use crate::ui::components::ds::kbd::kbd;
use crate::ui::components::ds::list_item::{ListItem, Tone, list_item};
use crate::ui::components::ds::sheet::{self, Sheet};
use crate::ui::icons::{Icon, icon};
use crate::ui::theme::tokens::{self, radius, space};
use crate::ui::theme::{color, font, motion};

pub static AI_INPUT_ID: LazyLock<Id> = LazyLock::new(Id::unique);

pub const TYPE_STEP: Duration = Duration::from_millis(16);
pub const CHARS_PER_STEP: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiKind {
  Generate,
  Diagnose,
  Explain,
}

impl AiKind {
  pub fn title(self) -> &'static str {
    match self {
      AiKind::Generate => "Suggested command",
      AiKind::Diagnose => "Diagnosis",
      AiKind::Explain => "Explain",
    }
  }
}

#[derive(Clone, Debug)]
pub struct AiAnswer {
  pub kind: AiKind,
  pub body: String,
  pub cmd: Option<String>,
  pub loading: bool,
  pub error: bool,
  pub started: Instant,
}

impl AiAnswer {
  pub fn loading(kind: AiKind, now: Instant) -> Self {
    Self {
      kind,
      body: String::new(),
      cmd: None,
      loading: true,
      error: false,
      started: now,
    }
  }

  pub fn revealed(&self, now: Instant) -> usize {
    let steps = now.saturating_duration_since(self.started).as_millis() / TYPE_STEP.as_millis();
    (steps as usize * CHARS_PER_STEP).min(self.body.chars().count())
  }

  pub fn is_done(&self, now: Instant) -> bool {
    !self.loading && self.revealed(now) >= self.body.chars().count()
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AiItem {
  pub kind: AiKind,
  pub label: String,
  pub hint: Option<String>,
}

pub fn ai_items(query: &str, failure: Option<(&str, i32)>) -> Vec<AiItem> {
  let q = query.trim();
  let mut items = Vec::new();
  if !q.is_empty() {
    items.push(AiItem {
      kind: AiKind::Generate,
      label: format!("Suggest a command for “{q}”"),
      hint: None,
    });
  }
  if let Some((cmd, code)) = failure {
    items.push(AiItem {
      kind: AiKind::Diagnose,
      label: format!("Fix “{cmd}”"),
      hint: Some(format!("exit {code}")),
    });
  }
  items.push(AiItem {
    kind: AiKind::Explain,
    label: "Explain last command".to_string(),
    hint: None,
  });
  items
}

pub fn rich_spans<'a>(text: &str, base: iced::Color) -> Vec<Span<'a>> {
  let mut out = Vec::new();
  for (i, part) in text.split('`').enumerate() {
    if part.is_empty() {
      continue;
    }
    if i % 2 == 1 {
      out.push(
        Span::new(part.to_string())
          .font(font::regular())
          .color(tokens::FG_1)
          .background(Background::Color(tokens::SURFACE_2))
          .padding(Padding::from([space::S2, space::S4])),
      );
    } else {
      out.push(
        Span::new(part.to_string())
          .font(font::regular())
          .color(base),
      );
    }
  }
  out
}

fn foot<'a>(answer_view: bool) -> Element<'a, Message> {
  let hint = |k: &'static str, label: &'static str| -> Element<'a, Message> {
    row![
      kbd(k),
      text(label).size(tokens::text::T14).color(tokens::FG_3)
    ]
    .spacing(space::S8)
    .align_y(Alignment::Center)
    .into()
  };
  sheet::foot(
    Row::new()
      .push(hint("↩", if answer_view { "Run" } else { "Select" }))
      .push(hint("esc", if answer_view { "Back" } else { "Close" })),
  )
}

pub fn suggestion<'a>(cmd: &str, on_run: Message) -> Element<'a, Message> {
  let accent = color::runtime().accent;
  container(
    row![
      text("$").size(tokens::text::T16).color(accent),
      container(
        text(cmd.to_string())
          .size(tokens::text::T16)
          .font(font::regular())
          .color(tokens::FG_1)
      )
      .width(Length::Fill),
      btn::<Message>(Variant::Primary)
        .label("Run")
        .trailing(Icon::Enter)
        .on_press(on_run),
    ]
    .spacing(space::S12)
    .align_y(Alignment::Center),
  )
  .padding(Padding {
    top: space::S8,
    right: space::S8,
    bottom: space::S8,
    left: space::S16,
  })
  .width(Length::Fill)
  .style(|_| container::Style {
    background: Some(Background::Color(tokens::SURFACE_1)),
    border: Border {
      radius: radius::R12.into(),
      width: 1.0,
      color: tokens::LINE,
    },
    ..container::Style::default()
  })
  .into()
}

fn answer_body<'a>(a: &AiAnswer, now: Instant) -> Element<'a, Message> {
  let mut col = Column::new().spacing(space::S16);
  if a.loading {
    col = col.push(
      row![
        dots(a.started, now),
        text("Thinking").size(tokens::text::T16).color(tokens::FG_3)
      ]
      .spacing(space::S12)
      .align_y(Alignment::Center),
    );
  } else {
    let shown: String = a.body.chars().take(a.revealed(now)).collect();
    let base = if a.error { tokens::RED } else { tokens::FG_2 };
    let mut spans = rich_spans(&shown, base);
    if !a.is_done(now) && motion::blink_on(a.started, now) {
      spans.push(
        Span::new("▍")
          .color(color::runtime().accent)
          .font(font::regular()),
      );
    }
    col = col.push(
      rich_text::<(), Message, iced::Theme, iced::Renderer>(spans)
        .size(tokens::text::T16)
        .line_height(1.5)
        .width(Length::Fill),
    );
    if a.is_done(now)
      && let Some(cmd) = &a.cmd
    {
      col = col.push(suggestion(cmd, Message::AiRunSuggestion));
    }
  }
  container(col)
    .padding(space::S24)
    .width(Length::Fill)
    .into()
}

pub fn inline_suggestion<'a>(message: &str, cmd: Option<&str>) -> Element<'a, Message> {
  let accent = color::runtime().accent;
  let mut col = Column::new().spacing(space::S12).push(
    row![
      icon(Icon::Sparkle, 16.0, accent),
      text("Nova AI")
        .size(tokens::text::T12)
        .font(font::semibold())
        .color(tokens::FG_3),
    ]
    .spacing(space::S8)
    .align_y(Alignment::Center),
  );
  col = col.push(
    rich_text::<(), Message, iced::Theme, iced::Renderer>(rich_spans(message, tokens::FG_2))
      .size(tokens::text::T14)
      .line_height(1.5)
      .width(Length::Fill),
  );
  if let Some(c) = cmd {
    col = col.push(suggestion(
      c,
      Message::DiagnosticBannerCommand(c.to_string()),
    ));
  }
  container(col)
    .width(Length::Fill)
    .padding(Padding::from([space::S16, space::S24]))
    .style(|_| container::Style {
      background: Some(Background::Color(tokens::SURFACE_1)),
      border: Border {
        radius: radius::R8.into(),
        ..Border::default()
      },
      ..container::Style::default()
    })
    .into()
}

pub struct AiSheetView<'a> {
  pub query: &'a str,
  pub items: Vec<AiItem>,
  pub selected: usize,
  pub answer: Option<&'a AiAnswer>,
  pub now: Instant,
  pub progress: f32,
  pub window_height: f32,
}

pub fn ai_sheet<'a>(v: AiSheetView<'a>) -> Element<'a, Message> {
  let accent = color::runtime().accent;
  let (head, body): (Element<'a, Message>, Element<'a, Message>) =
    match v.answer {
      Some(a) => (
        sheet::head(row![
          icon_button(Icon::Back, IconSize::Md, Some(Message::AiBack)),
          container(
            text(a.kind.title())
              .size(tokens::text::T16)
              .font(font::semibold())
              .color(tokens::FG_1)
          )
          .width(Length::Fill),
          icon(Icon::Sparkle, 20.0, tokens::FG_3),
        ]),
        answer_body(a, v.now),
      ),
      None => (
        sheet::head(row![
          icon(Icon::Sparkle, 20.0, accent),
          text_input("Describe what you want to do", v.query)
            .id(AI_INPUT_ID.clone())
            .on_input(Message::AiOverlayInputChanged)
            .on_submit(Message::AiConfirm)
            .size(tokens::text::T18)
            .font(font::regular())
            .padding(0)
            .width(Length::Fill)
            .style(|_t, s| sheet_input_style(s)),
        ]),
        container(v.items.iter().enumerate().fold(
          Column::new().spacing(space::S2),
          |c, (i, it)| {
            c.push(list_item(
              ListItem {
                icon: match it.kind {
                  AiKind::Generate => Icon::Sparkle,
                  AiKind::Diagnose => Icon::Alert,
                  AiKind::Explain => Icon::Help,
                },
                label: it.label.clone(),
                hint: it.hint.clone(),
                keys: None,
                tone: if it.kind == AiKind::Diagnose {
                  Tone::Error
                } else {
                  Tone::Normal
                },
              },
              i == v.selected,
              Message::AiHover(i),
              Message::AiPick(i),
            ))
          },
        ))
        .padding(space::S8)
        .into(),
      ),
    };
  sheet::sheet(
    Sheet {
      width: sheet::WIDTH,
      head: Some(head),
      body,
      foot: Some(foot(v.answer.is_some())),
    },
    v.progress,
    v.window_height,
    Message::CloseAiOverlay,
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn items_follow_design_rules() {
    let only_explain = ai_items("  ", None);
    assert_eq!(only_explain.len(), 1);
    assert_eq!(only_explain[0].kind, AiKind::Explain);
    let all = ai_items("free port", Some(("clsd", 127)));
    assert_eq!(all[0].label, "Suggest a command for “free port”");
    assert_eq!(all[1].label, "Fix “clsd”");
    assert_eq!(all[1].hint.as_deref(), Some("exit 127"));
    assert_eq!(all[2].kind, AiKind::Explain);
  }

  #[test]
  fn reveal_types_two_chars_per_frame() {
    let t0 = Instant::now();
    let mut a = AiAnswer::loading(AiKind::Explain, t0);
    a.loading = false;
    a.body = "abcdefgh".into();
    assert_eq!(a.revealed(t0), 0);
    assert_eq!(a.revealed(t0 + Duration::from_millis(33)), 4);
    assert!(a.is_done(t0 + Duration::from_millis(80)));
  }

  #[test]
  fn backticks_become_code_spans() {
    let s = rich_spans("run `ls -la` now", tokens::FG_2);
    assert_eq!(s.len(), 3);
    assert_eq!(s[1].text.as_ref(), "ls -la");
    assert!(s[1].highlight.is_some());
  }
}
