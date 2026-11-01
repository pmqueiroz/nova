use iced::widget::{Column, container, row, text};
use iced::{Element, Length};

use crate::ui::app_state::Message;
use crate::ui::components::ds::list_item::{ListItem, Tone, list_item};
use crate::ui::components::ds::sheet::{self, Sheet};
use crate::ui::icons::{Icon, icon};
use crate::ui::theme::font;
use crate::ui::theme::tokens::{self, space};

pub fn shell_picker<'a>(
  shells: &'a [String],
  selected: usize,
  progress: f32,
  window_height: f32,
) -> Element<'a, Message> {
  let head = sheet::head(row![
    icon(Icon::Terminal, 20.0, tokens::FG_3),
    container(
      text("New tab with shell")
        .size(tokens::text::T16)
        .font(font::semibold())
        .color(tokens::FG_1)
    )
    .width(Length::Fill),
  ]);
  let list = shells
    .iter()
    .enumerate()
    .fold(Column::new().spacing(space::S2), |c, (i, shell)| {
      c.push(list_item(
        ListItem {
          icon: Icon::Terminal,
          label: shell.clone(),
          hint: (i == 0).then(|| "default".to_string()),
          keys: None,
          tone: Tone::Normal,
        },
        i == selected,
        Message::ShellPickerHover(i),
        Message::NewTabWithShell(shell.clone()),
      ))
    });
  sheet::sheet(
    Sheet {
      width: sheet::WIDTH,
      head: Some(head),
      body: container(list).padding(space::S8).into(),
      foot: None,
    },
    progress,
    window_height,
    Message::CloseShellPicker,
  )
}
