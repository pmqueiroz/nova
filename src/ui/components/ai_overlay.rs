use std::sync::LazyLock;

use iced::{
  Border, Color, Element, Length, Padding,
  border::Radius,
  widget::{
    Id, button, column, container, mouse_area, row, rule, scrollable, space, stack, text,
    text_input,
  },
};

use crate::ui::{app_state::Message, theme};

pub static AI_INPUT_ID: LazyLock<Id> = LazyLock::new(Id::unique);

enum Segment {
  Text(String),
  Code { lang: String, content: String },
}

fn parse_segments(response: &str) -> Vec<Segment> {
  let mut segments = Vec::new();
  let mut remaining = response;

  while !remaining.is_empty() {
    match remaining.find("```") {
      None => {
        let t = remaining.trim().to_string();
        if !t.is_empty() {
          segments.push(Segment::Text(t));
        }
        break;
      }
      Some(0) => {
        remaining = &remaining[3..];
        let lang_end = remaining.find('\n').unwrap_or(remaining.len());
        let lang = remaining[..lang_end].trim().to_string();
        remaining = if lang_end < remaining.len() {
          &remaining[lang_end + 1..]
        } else {
          ""
        };
        match remaining.find("```") {
          Some(end) => {
            let content = remaining[..end].trim_end_matches('\n').to_string();
            if !content.trim().is_empty() {
              segments.push(Segment::Code { lang, content });
            }
            remaining = &remaining[end + 3..];
            if remaining.starts_with('\n') {
              remaining = &remaining[1..];
            }
          }
          None => {
            let content = remaining.trim_end_matches('\n').to_string();
            if !content.trim().is_empty() {
              segments.push(Segment::Code { lang, content });
            }
            break;
          }
        }
      }
      Some(pos) => {
        let t = remaining[..pos].trim().to_string();
        if !t.is_empty() {
          segments.push(Segment::Text(t));
        }
        remaining = &remaining[pos..];
      }
    }
  }

  segments
}

pub fn ai_overlay<'a>(
  input: &'a str,
  response: Option<&'a str>,
  is_loading: bool,
  is_error: bool,
) -> Element<'a, Message> {
  let rt = theme::color::runtime();
  let fg = rt.foreground;
  let fg_muted = rt.foreground_muted;
  let accent = rt.accent;
  let bg = rt.background;
  let border_c = rt.border;
  drop(rt);

  let red = Color {
    r: 0.9,
    g: 0.3,
    b: 0.3,
    a: 1.0,
  };

  let backdrop = mouse_area(
    container(iced::widget::Space::new())
      .width(Length::Fill)
      .height(Length::Fill)
      .style(|_| container::Style {
        background: Some(
          Color {
            a: 0.55,
            ..Color::BLACK
          }
          .into(),
        ),
        ..Default::default()
      }),
  )
  .on_press(Message::CloseAiOverlay);

  let header = container(
    row![
      text("Ask AI").size(13).color(fg).font(theme::font::REGULAR),
      space::horizontal(),
      button(text("×").size(14).color(fg_muted))
        .style(|_t, _s| button::Style {
          background: Some(Color::TRANSPARENT.into()),
          ..Default::default()
        })
        .on_press(Message::CloseAiOverlay)
        .padding(Padding::from([2, 6])),
    ]
    .align_y(iced::alignment::Vertical::Center),
  )
  .padding(Padding {
    top: 12.0,
    bottom: 12.0,
    left: 16.0,
    right: 8.0,
  })
  .width(Length::Fill);

  let input_area = container(
    row![
      text_input("Ask anything about your terminal…", input)
        .id(AI_INPUT_ID.clone())
        .on_input(Message::AiOverlayInputChanged)
        .on_submit(Message::AiSubmit)
        .size(13)
        .padding(Padding::from([0, 4]))
        .style(move |_t, _status| text_input::Style {
          background: iced::Background::Color(Color::TRANSPARENT),
          border: Border::default(),
          icon: fg_muted,
          placeholder: fg_muted,
          value: fg,
          selection: Color { a: 0.3, ..accent },
        }),
      button(text("Send").size(12).color(accent))
        .style(move |_t, status| button::Style {
          background: Some(
            match status {
              button::Status::Hovered => Color { a: 0.15, ..accent },
              _ => Color::TRANSPARENT,
            }
            .into(),
          ),
          border: Border {
            color: accent,
            width: 1.0,
            radius: Radius::new(4.0)
          },
          text_color: accent,
          ..Default::default()
        })
        .on_press(Message::AiSubmit)
        .padding(Padding {
          top: 5.0,
          bottom: 3.0,
          left: 10.0,
          right: 10.0
        }),
    ]
    .spacing(8)
    .align_y(iced::alignment::Vertical::Center),
  )
  .padding(Padding {
    top: 10.0,
    bottom: 10.0,
    left: 16.0,
    right: 12.0,
  })
  .width(Length::Fill);

  let show_response = is_loading || response.is_some();

  let mut modal_col = column![header, rule::horizontal(1), input_area];

  if show_response {
    let response_content: Element<'a, Message> = if is_loading {
      text("Thinking…")
        .size(12)
        .color(fg_muted)
        .font(theme::font::REGULAR)
        .into()
    } else if let Some(content) = response {
      if is_error {
        text(content)
          .size(12)
          .color(red)
          .font(theme::font::REGULAR)
          .width(Length::Fill)
          .into()
      } else {
        let segments = parse_segments(content);
        let mut col = column![].spacing(10).width(Length::Fill);
        for segment in segments {
          let widget: Element<'a, Message> = match segment {
            Segment::Text(t) => text(t)
              .size(12)
              .color(fg)
              .font(theme::font::REGULAR)
              .width(Length::Fill)
              .into(),
            Segment::Code { lang, content } => {
              let label = if lang.is_empty() {
                "code".to_string()
              } else {
                lang
              };
              let code = content.trim().to_string();
              let copy_code = code.clone();
              let run_code = code.clone();

              container(column![
                container(
                  row![
                    text(label)
                      .size(10)
                      .color(fg_muted)
                      .font(theme::font::REGULAR),
                    space::horizontal(),
                    button(text("Copy").size(10).color(fg_muted))
                      .style(move |_t, status| button::Style {
                        background: Some(match status {
                          button::Status::Hovered => Color {
                            a: 0.12,
                            ..fg_muted
                          }
                          .into(),
                          _ => Color::TRANSPARENT.into(),
                        }),
                        border: Border {
                          color: border_c,
                          width: 1.0,
                          radius: Radius::new(3.0),
                        },
                        text_color: fg_muted,
                        ..Default::default()
                      })
                      .on_press(Message::CopyCodeBlock(copy_code))
                      .padding(Padding::from([2, 6])),
                    button(text("Run").size(10).color(accent))
                      .style(move |_t, status| button::Style {
                        background: Some(match status {
                          button::Status::Hovered => Color { a: 0.15, ..accent }.into(),
                          _ => Color::TRANSPARENT.into(),
                        }),
                        border: Border {
                          color: accent,
                          width: 1.0,
                          radius: Radius::new(3.0),
                        },
                        text_color: accent,
                        ..Default::default()
                      })
                      .on_press(Message::RunCodeInTerminal(run_code))
                      .padding(Padding::from([2, 6])),
                  ]
                  .spacing(6)
                  .align_y(iced::alignment::Vertical::Center),
                )
                .padding(Padding {
                  top: 6.0,
                  bottom: 6.0,
                  left: 10.0,
                  right: 6.0
                })
                .width(Length::Fill),
                rule::horizontal(1),
                container(
                  text(code)
                    .size(11)
                    .color(fg)
                    .font(theme::font::REGULAR)
                    .width(Length::Fill),
                )
                .padding(Padding::from([8, 10]))
                .width(Length::Fill),
              ])
              .style(move |_| container::Style {
                background: Some(Color { a: 0.06, ..fg }.into()),
                border: Border {
                  color: border_c,
                  width: 1.0,
                  radius: Radius::new(4.0),
                },
                ..Default::default()
              })
              .width(Length::Fill)
              .into()
            }
          };
          col = col.push(widget);
        }
        col.into()
      }
    } else {
      iced::widget::Space::new().into()
    };

    let response_pane = container(
      scrollable(
        container(response_content)
          .padding(Padding::from([0, 4]))
          .width(Length::Fill),
      )
      .height(350)
      .direction(scrollable::Direction::Vertical(
        scrollable::Scrollbar::new()
          .width(4)
          .margin(4)
          .scroller_width(4),
      )),
    )
    .padding(Padding {
      top: 12.0,
      bottom: 12.0,
      left: 16.0,
      right: 12.0,
    })
    .width(Length::Fill);

    modal_col = modal_col.push(rule::horizontal(1)).push(response_pane);
  }

  let modal = mouse_area(
    container(
      container(modal_col)
        .style(move |_| container::Style {
          background: Some(bg.into()),
          border: Border {
            color: border_c,
            width: 1.0,
            radius: Radius::new(8.0),
          },
          ..Default::default()
        })
        .width(600),
    )
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .width(Length::Fill)
    .height(Length::Fill),
  )
  .on_press(Message::NoOp);

  stack![backdrop, modal].into()
}
