use async_channel::Sender;
use iced::keyboard::Key;
use iced::keyboard::key::Named;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Length, Subscription, Theme};
use iced::{Event, event, keyboard, stream};
use vte::Parser;

use crate::core::grid::Grid;
use crate::sys::parser::AnsiExecutor;
use crate::sys::pty::PtyBridge;
use crate::ui::components;
use crate::ui::typography::Typography;

pub struct Tab {
  pub id: usize,
  pub grid: Grid,
  pub pty_tx: Option<Sender<Vec<u8>>>,
  pub ansi_parser: Parser,
}

impl Tab {
  pub fn new(id: usize) -> Self {
    Self {
      id,
      grid: Grid::new(80, 24),
      pty_tx: None,
      ansi_parser: Parser::new(),
    }
  }
}

pub struct Nova {
  tabs: Vec<Tab>,
  active_index: usize,
  next_tab_id: usize,
}

#[derive(Debug, Clone)]
pub enum Message {
  Type(Vec<u8>),
  NewTab,
  SwitchTab(usize),
  CloseTab(usize),
  PtyReady(usize, Sender<Vec<u8>>),
  PtyOutputReceived(usize, Vec<u8>),
}

impl Default for Nova {
  fn default() -> Self {
    Self {
      tabs: vec![Tab::new(0)],
      active_index: 0,
      next_tab_id: 1,
    }
  }
}

impl Nova {
  pub fn update(&mut self, message: Message) {
    match message {
      Message::Type(bytes) => {
        if let Some(active_tab) = self.tabs.get(self.active_index) {
          if let Some(tx) = &active_tab.pty_tx {
            let _ = tx.send_blocking(bytes);
          }
        }
      }
      Message::NewTab => {
        let new_id = self.next_tab_id;
        self.next_tab_id += 1;
        self.tabs.push(Tab::new(new_id));
        self.active_index = self.tabs.len() - 1;
      }
      Message::SwitchTab(index) => {
        if index < self.tabs.len() {
          self.active_index = index;
        }
      }
      Message::CloseTab(index) => {
        self.tabs.remove(index);
        if self.tabs.is_empty() {
          self.tabs.push(Tab::new(self.next_tab_id));
          self.next_tab_id += 1;
          self.active_index = 0;
        } else if self.active_index >= self.tabs.len() {
          self.active_index = self.tabs.len() - 1;
        }
      }
      Message::PtyReady(tab_id, tx) => {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) {
          tab.pty_tx = Some(tx);
        }
      }
      Message::PtyOutputReceived(tab_id, bytes) => {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) {
          let mut executor = AnsiExecutor {
            grid: &mut tab.grid,
          };
          for byte in bytes {
            tab.ansi_parser.advance(&mut executor, &[byte]);
          }
        }
      }
    }
  }

  pub fn view(&self) -> Element<'_, Message> {
    let mut tab_bar = row![].spacing(5).padding(5);

    for (i, tab) in self.tabs.iter().enumerate() {
      // let is_active = i == self.active_index;
      let tab_btn = button(text(format!("Terminal {}", tab.id)))
        .on_press(Message::SwitchTab(i))
        .padding(8);

      let close_btn = button(text("x")).on_press(Message::CloseTab(i)).padding(8);

      tab_bar = tab_bar.push(row![tab_btn, close_btn].spacing(2));
    }

    tab_bar = tab_bar.push(button(text("+")).on_press(Message::NewTab).padding(8));

    let active_tab = &self.tabs[self.active_index];
    let mut grid_ui = column![].spacing(0);

    for (y, row_cells) in active_tab.grid.cells.iter().enumerate() {
      let mut ui_row = row![].spacing(0);
      let mut current_text = String::new();
      let mut current_color = iced::Color::WHITE;

      for (x, cell) in row_cells.iter().enumerate() {
        let is_cursor = x == active_tab.grid.cursor_x && y == active_tab.grid.cursor_y;

        if is_cursor || cell.fg != current_color {
          if !current_text.is_empty() {
            ui_row = ui_row.push(
              text(current_text.clone())
                .font(iced::Font::MONOSPACE)
                .color(current_color)
                .size(16),
            );
            current_text.clear();
          }
          current_color = cell.fg;
        }

        if is_cursor {
          ui_row = ui_row.push(components::cursor());
        } else {
          current_text.push(cell.c);
        }
      }

      if !current_text.is_empty() {
        ui_row = ui_row.push(
          Typography {
            color: current_color,
            ..Default::default()
          }
          .as_text(&current_text),
        );
      }

      grid_ui = grid_ui.push(ui_row);
    }

    let output_area = scrollable(grid_ui).height(Length::Fill).width(Length::Fill);

    container(column![tab_bar, output_area, components::status_bar()])
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
    let mut subs = Vec::new();

    let keyboard_sub = event::listen_with(|event, _s, _w| {
      if let Event::Keyboard(keyboard::Event::KeyPressed {
        key,
        modifiers,
        modified_key,
        ..
      }) = event
      {
        match key {
          Key::Named(Named::Enter) => return Some(Message::Type(b"\r".to_vec())),
          Key::Named(Named::Backspace) => {
            #[cfg(target_os = "windows")]
            {
              return Some(Message::Type(b"\x08".to_vec()));
            }

            #[cfg(not(target_os = "windows"))]
            {
              return Some(Message::Type(b"\x7F".to_vec()));
            }
          }
          Key::Named(Named::Space) => return Some(Message::Type(b" ".to_vec())),
          _ => {}
        }

        if let Key::Character(c) = modified_key {
          return Some(Message::Type(c.as_str().as_bytes().to_vec()));
        }

        if let Key::Character(c) = key {
          let mut char_str = c.as_str().to_string();

          if char_str == "'" && modifiers.shift() {
            char_str = "\"".to_string();
          }

          return Some(Message::Type(char_str.as_bytes().to_vec()));
        }

        None
      } else {
        None
      }
    });

    subs.push(keyboard_sub);

    for tab in &self.tabs {
      let tab_id = tab.id;
      let pty_sub = Subscription::run_with(tab_id, |tab_id| pty_worker(*tab_id));
      subs.push(pty_sub);
    }

    Subscription::batch(subs)
  }
}

fn pty_worker(tab_id: usize) -> impl iced::futures::Stream<Item = Message> {
  stream::channel(
    100,
    move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
      use iced::futures::SinkExt;

      let (tx_out, rx_out) = async_channel::unbounded::<Vec<u8>>();
      let (tx_in, rx_in) = async_channel::unbounded::<Vec<u8>>();

      std::thread::spawn(move || {
        let mut pty = PtyBridge::new(tx_out).expect("Failed to create PTY bridge");

        while let Ok(bytes) = rx_in.recv_blocking() {
          pty.write_to_pty(&bytes);
        }
      });

      let _ = output.send(Message::PtyReady(tab_id, tx_in)).await;

      while let Ok(bytes) = rx_out.recv().await {
        let _ = output.send(Message::PtyOutputReceived(tab_id, bytes)).await;
      }
    },
  )
}
