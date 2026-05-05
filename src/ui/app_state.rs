use crate::sys::pty::PtyBridge;
use async_channel::Sender;
use iced::stream;
use iced::widget::{column, container, scrollable, text, text_input};
use iced::{Element, Length, Subscription, Theme};

pub struct Nova {
  pty_tx: Option<Sender<String>>,
  history: Vec<String>,
  input: String,
}

#[derive(Debug, Clone)]
pub enum Message {
  InputChanged(String),
  CommandSubmitted,
  PtyReady(Sender<String>),
  PtyOutputReceived(Vec<u8>),
}

impl Default for Nova {
  fn default() -> Self {
    Self {
      history: Vec::new(),
      input: String::new(),
      pty_tx: None,
    }
  }
}

impl Nova {
  pub fn update(&mut self, message: Message) {
    match message {
      Message::PtyReady(tx) => {
        self.pty_tx = Some(tx);
      }
      Message::InputChanged(new_input) => {
        self.input = new_input;
      }
      Message::CommandSubmitted => {
        if !self.input.trim().is_empty() {
          let cmd = self.input.clone();
          if let Some(tx) = &self.pty_tx {
            let _ = tx.send_blocking(cmd);
          }

          self.input.clear();
        }
      }
      Message::PtyOutputReceived(output) => {
        let text = String::from_utf8_lossy(&output).to_string();
        self.history.push(text);
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

  pub fn subscription(&self) -> Subscription<Message> {
    Subscription::run(pty_worker)
  }
}

fn pty_worker() -> impl iced::futures::Stream<Item = Message> {
  stream::channel(
    100,
    |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
      use iced::futures::SinkExt;

      let (tx_out, rx_out) = async_channel::unbounded::<Vec<u8>>();
      let (tx_in, rx_in) = async_channel::unbounded::<String>();

      std::thread::spawn(move || {
        let mut pty = PtyBridge::new(tx_out).expect("Failed to create PTY bridge");

        while let Ok(input) = rx_in.recv_blocking() {
          pty.write_to_pty(&input);
        }
      });

      let _ = output.send(Message::PtyReady(tx_in)).await;

      while let Ok(bytes) = rx_out.recv().await {
        let _ = output.send(Message::PtyOutputReceived(bytes)).await;
      }
    },
  )
}
