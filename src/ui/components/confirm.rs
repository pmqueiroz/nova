use iced::widget::{Column, Row, container, row, text};
use iced::{Element, Length};

use crate::ui::app_state::Message;
use crate::ui::components::ds::button::{Variant, btn};
use crate::ui::components::ds::sheet::{self, Sheet};
use crate::ui::icons::{Icon, icon};
use crate::ui::theme::font;
use crate::ui::theme::tokens::{self, space};

pub struct ConfirmView<'a> {
  pub title: String,
  pub reason: String,
  pub detail: Option<Element<'a, Message>>,
  pub confirm_label: &'static str,
}

pub fn confirm_sheet<'a>(
  v: ConfirmView<'a>,
  progress: f32,
  window_height: f32,
) -> Element<'a, Message> {
  let head = sheet::head(row![
    icon(Icon::Alert, 20.0, tokens::RED),
    container(
      text(v.title)
        .size(tokens::text::T16)
        .font(font::semibold())
        .color(tokens::FG_1)
    )
    .width(Length::Fill),
  ]);

  let body = container(
    Column::new()
      .spacing(space::S16)
      .push(text(v.reason).size(tokens::text::T14).color(tokens::FG_3))
      .push(v.detail),
  )
  .padding(space::S24)
  .width(Length::Fill);

  let foot = sheet::foot(
    Row::new()
      .push(container(text("")).width(Length::Fill))
      .push(
        btn::<Message>(Variant::Ghost)
          .label("Cancel")
          .on_press(Message::ConfirmCancelled),
      )
      .push(
        btn::<Message>(Variant::Primary)
          .label(v.confirm_label)
          .on_press(Message::ConfirmAccepted),
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
    Message::ConfirmCancelled,
  )
}
