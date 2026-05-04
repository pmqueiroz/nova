use iced::widget::{column, container, scrollable, text, text_input};
use iced::{Element, Length, Theme};

pub struct Nova {
  history: Vec<String>,
  input: String,
}

impl Default for Nova {
  fn default() -> Self {
    Self {
      history: Vec::new(),
      input: String::new(),
    }
  }
}

#[derive(Debug, Clone)]
pub enum Message {
  InputChanged(String),
  CommandSubmitted,
}

impl Nova {
  pub fn update(&mut self, message: Message) {
    match message {
      Message::InputChanged(new_input) => {
        self.input = new_input;
      }
      Message::CommandSubmitted => {
        if !self.input.trim().is_empty() {
          self.history.push(self.input.clone());
          self.input.clear();
        }
      }
    }
  }

  pub fn view(&self) -> Element<'_, Message> {
    let history = self
      .history
      .iter()
      .fold(column![].spacing(5), |col, entry| {
        col.push(text(entry).size(16))
      });

    let output_area = scrollable(history).height(Length::Fill);

    let input = text_input("$ ", &self.input)
      .on_input(Message::InputChanged)
      .on_submit(Message::CommandSubmitted)
      .padding(10)
      .size(16);

    let layout = column![output_area, input]
      .spacing(10)
      .padding(20)
      .width(Length::Fill)
      .height(Length::Fill);

    container(layout)
      .width(Length::Fill)
      .height(Length::Fill)
      .center_x(Length::Fill)
      .center_y(Length::Fill)
      .into()
  }

  pub fn theme(&self) -> Theme {
    Theme::Dracula
  }
}
