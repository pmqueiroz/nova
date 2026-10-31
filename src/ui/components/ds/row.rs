use iced::widget::{Column, Row, container, text};
use iced::{Alignment, Background, Element, Length, Padding};

use crate::ui::theme::tokens::{self, height, space};

pub fn row<'a, Message: 'a>(
  label: impl Into<String>,
  hint: Option<String>,
  control: impl Into<Element<'a, Message>>,
  last: bool,
) -> Element<'a, Message> {
  let mut labels = Column::new().spacing(space::S2).push(
    text(label.into())
      .size(tokens::text::T16)
      .color(tokens::FG_1),
  );
  if let Some(h) = hint {
    labels = labels.push(text(h).size(tokens::text::T14).color(tokens::FG_3));
  }
  let body = container(
    Row::new()
      .spacing(space::S24)
      .align_y(Alignment::Center)
      .push(container(labels).width(Length::Fill))
      .push(control),
  )
  .width(Length::Fill)
  .padding(Padding::from([space::S12, 0.0]))
  .center_y(height::H64);
  let mut col = Column::new().push(body);
  if !last {
    col = col.push(row_divider());
  }
  col.into()
}

pub fn row_divider<'a, Message: 'a>() -> Element<'a, Message> {
  container(text(""))
    .width(Length::Fill)
    .height(1)
    .style(|_| container::Style {
      background: Some(Background::Color(tokens::LINE)),
      ..container::Style::default()
    })
    .into()
}
