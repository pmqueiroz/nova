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
  pty_tx: Option<Sender<Vec<u8>>>,
  grid: Grid,
  ansi_parser: Parser,
}

#[derive(Debug, Clone)]
pub enum Message {
  Type(Vec<u8>),
  PtyReady(Sender<Vec<u8>>),
  PtyOutputReceived(Vec<u8>),
}

impl Default for Nova {
  fn default() -> Self {
    Self {
      pty_tx: None,
      grid: Grid::new(80, 24),
      ansi_parser: Parser::new(),
    }
  }
}

impl Nova {
  pub fn update(&mut self, message: Message) {
    match message {
      Message::PtyReady(tx) => {
        self.pty_tx = Some(tx);
      }
      Message::Type(bytes) => {
        if let Some(tx) = &self.pty_tx {
          let _ = tx.send_blocking(bytes);
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
    for (y, row_cells) in self.grid.cells.iter().enumerate() {
      let mut ui_row = row![].spacing(0);

      for (x, cell) in row_cells.iter().enumerate() {
        let mut text_char = cell.c.to_string();
        let mut color = cell.fg;

        if x == self.grid.cursor_x && y == self.grid.cursor_y {
          text_char = "_".to_string();
          color = iced::Color::from_rgb(0.2, 0.8, 0.2);
        }

        ui_row = ui_row.push(
          text(text_char)
            .font(iced::Font::MONOSPACE)
            .color(color)
            .size(16),
        );
      }
      ui_column = ui_column.push(ui_row);
    }

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
          Key::Named(Named::Enter) => Some(Message::Type(b"\r".to_vec())),
          Key::Named(Named::Backspace) => Some(Message::Type(b"\x08".to_vec())),
          Key::Named(Named::Space) => Some(Message::Type(b" ".to_vec())),
          _ => {
            if let Some(t) = text {
              Some(Message::Type(t.as_bytes().to_vec()))
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
      let (tx_in, rx_in) = async_channel::unbounded::<Vec<u8>>();

      std::thread::spawn(move || {
        let mut pty = PtyBridge::new(tx_out).expect("Failed to create PTY bridge");

        while let Ok(bytes) = rx_in.recv_blocking() {
          pty.write_to_pty(&bytes);
        }
      });

      let _ = output.send(Message::PtyReady(tx_in)).await;

      while let Ok(bytes) = rx_out.recv().await {
        let _ = output.send(Message::PtyOutputReceived(bytes)).await;
      }
    },
  )
}
