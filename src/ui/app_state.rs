use async_channel::Sender;
use iced::keyboard::Key;
use iced::keyboard::key::Named;
use iced::widget::{column, container, row, scrollable, text};
use iced::{Element, Length, Subscription, Theme};
use iced::{Event, event, keyboard, stream};
use vte::Parser;

use crate::core::grid::Grid;
use crate::sys::parser::AnsiExecutor;
use crate::sys::pty::PtyBridge;

pub struct Nova {
  pty_tx: Option<Sender<String>>,
  grid: Grid,
  ansi_parser: Parser,
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
      pty_tx: None,
      grid: Grid::new(80, 24),
      ansi_parser: Parser::new(),
      input: String::new(),
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
        let mut executor = AnsiExecutor {
          grid: &mut self.grid,
        };
        for byte in output {
          self.ansi_parser.advance(&mut executor, &[byte]);
        }
      }
    }
  }

  pub fn view(&self) -> Element<'_, Message> {
    let mut ui_column = column![].spacing(2);
    for row_cells in &self.grid.cells {
      let mut ui_row = row![].spacing(0);

      for cell in row_cells {
        ui_row = ui_row.push(
          text(cell.c.to_string())
            .font(iced::Font::MONOSPACE)
            .color(cell.fg)
            .size(16),
        );
      }
      ui_column = ui_column.push(ui_row);
    }

    let prompt = format!("$ {}{}", self.input, "|");

    ui_column = ui_column.push(
      text(prompt)
        .size(16)
        .font(iced::Font::MONOSPACE)
        .color(iced::Color::from_rgb(0.2, 0.8, 0.2)),
    );

    let output_area = scrollable(ui_column)
      .height(Length::Fill)
      .width(Length::Fill);

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
