use crate::sys::pty::PtyBridge;
use async_channel::Sender;
use iced::keyboard::Key;
use iced::keyboard::key::Named;
use iced::widget::{column, container, scrollable, text};
use iced::{Element, Length, Subscription, Theme};
use iced::{Event, event, keyboard, stream};

pub struct Nova {
  pty_tx: Option<Sender<String>>,
  history: Vec<String>,
  input: String,
}

#[derive(Debug, Clone)]
pub enum Message {
  Type(String),
  Backspace,
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
      Message::Type(c) => {
        if !c.chars().any(|ch| ch.is_control()) {
          self.input.push_str(&c);
        }
      }
      Message::Backspace => {
        self.input.pop();
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
    let mut col = column![].spacing(4);
    for line in &self.history {
      col = col.push(text(line).size(16).font(iced::Font::MONOSPACE));
    }

    let prompt = format!("$ {}{}", self.input, "|");

    col = col.push(
      text(prompt)
        .size(16)
        .font(iced::Font::MONOSPACE)
        .color(iced::Color::from_rgb(0.2, 0.8, 0.2)),
    );

    let output_area = scrollable(col).height(Length::Fill).width(Length::Fill);

    container(output_area)
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
    let pty_sub = Subscription::run(pty_worker);

    let keyboard_sub = event::listen_with(|event, _s, _w| {
      if let Event::Keyboard(keyboard::Event::KeyPressed { key, text, .. }) = event {
        match key {
          Key::Named(Named::Enter) => Some(Message::CommandSubmitted),
          Key::Named(Named::Backspace) => Some(Message::Backspace),
          Key::Named(Named::Space) => Some(Message::Type(" ".to_string())),
          _ => {
            if let Some(t) = text {
              Some(Message::Type(t.to_string()))
            } else {
              None
            }
          }
        }
      } else {
        None
      }
    });

    Subscription::batch([pty_sub, keyboard_sub])
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
