use async_channel::Sender;
use iced::keyboard::Key;
use iced::keyboard::key::Named;
use iced::widget::column;
use iced::{Element, Point, Size, Subscription, Theme, time, window};
use iced::{Event, event, keyboard, mouse, stream};

use crate::sys::parser::AnsiExecutor;
use crate::sys::pty::{PtyBridge, PtyCommand};
use crate::ui::components;
use crate::ui::tab::Tab;

pub struct Nova {
  tabs: Vec<Tab>,
  active_index: usize,
  next_tab_id: usize,
  window_id: Option<window::Id>,
  window_focused: bool,
  window_size: Size,
  cursor_position: Point,
}

#[derive(Debug, Clone)]
pub enum Message {
  Type(Vec<u8>),
  NewTab,
  SwitchTab(usize),
  CloseTab(usize),
  PtyReady(usize, Sender<PtyCommand>),
  PtyOutputReceived(usize, Vec<u8>),
  CloseActiveTab,
  NextTab,
  PrevTab,
  CloseWindow,
  MinimizeWindow,
  MaximizeWindow,
  DragWindow,
  WindowOpened(window::Id),
  WindowFocused,
  WindowUnfocused,
  WindowResized(f32, f32),
  PasteRequested,
  ClipboardReceived(Option<String>),
  CursorMoved(Point),
  MousePressed,
  Tick,
}

fn calc_grid(width: f32, height: f32) -> (usize, usize) {
  let font_size = 16.0_f32;
  let char_width = font_size * 0.62;
  let char_height = font_size * 1.35;
  let cols = ((width - 40.0) / char_width).floor() as usize;
  let rows = ((height - 118.0) / char_height).floor() as usize;
  (cols.max(10), rows.max(5))
}

impl Default for Nova {
  fn default() -> Self {
    let (cols, rows) = calc_grid(1024.0, 768.0);
    Self {
      tabs: vec![Tab::new(0, cols, rows)],
      active_index: 0,
      next_tab_id: 1,
      window_id: None,
      window_focused: false,
      window_size: Size::new(1024.0, 768.0),
      cursor_position: Point::ORIGIN,
    }
  }
}

impl Nova {
  pub fn update(&mut self, message: Message) -> iced::Task<Message> {
    match message {
      Message::Type(bytes) => {
        if let Some(active_tab) = self.tabs.get(self.active_index) {
          if let Some(tx) = &active_tab.pty_tx {
            let _ = tx.send_blocking(PtyCommand::Input(bytes));
          }
        }
      }
      Message::NewTab => {
        let new_id = self.next_tab_id;
        self.next_tab_id += 1;
        let (cols, rows) = calc_grid(self.window_size.width, self.window_size.height);
        self.tabs.push(Tab::new(new_id, cols, rows));
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
          let (cols, rows) = calc_grid(self.window_size.width, self.window_size.height);
          self.tabs.push(Tab::new(self.next_tab_id, cols, rows));
          self.next_tab_id += 1;
          self.active_index = 0;
        } else if self.active_index >= self.tabs.len() {
          self.active_index = self.tabs.len() - 1;
        }
      }
      Message::CloseActiveTab => {
        return self.update(Message::CloseTab(self.active_index));
      }
      Message::NextTab => {
        if !self.tabs.is_empty() {
          self.active_index = (self.active_index + 1) % self.tabs.len();
        }
      }
      Message::PrevTab => {
        if !self.tabs.is_empty() {
          self.active_index = if self.active_index == 0 {
            self.tabs.len() - 1
          } else {
            self.active_index - 1
          };
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

          while !tab.grid.output_queue.is_empty() {
            let response = tab.grid.output_queue.remove(0);

            if let Some(tx) = &tab.pty_tx {
              let _ = tx.send_blocking(PtyCommand::Input(response));
            }
          }

          let new_pwd = tab.grid.pwd.clone();
          if new_pwd != tab.pwd {
            tab.pwd = new_pwd;
          }
          tab.update_git_status();
        }
      }
      Message::WindowOpened(id) => {
        self.window_id = Some(id);
      }
      Message::MinimizeWindow => {
        if let Some(window_id) = self.window_id {
          return window::minimize(window_id, true);
        }
      }
      Message::MaximizeWindow => {
        if let Some(window_id) = self.window_id {
          return window::toggle_maximize(window_id);
        }
      }
      Message::CloseWindow => {
        std::process::exit(0);
      }
      Message::DragWindow => {
        if let Some(window_id) = self.window_id {
          return window::drag(window_id);
        }
      }
      Message::WindowFocused => {
        self.window_focused = true;
      }
      Message::WindowUnfocused => {
        self.window_focused = false;
      }
      Message::WindowResized(width, height) => {
        self.window_size = Size::new(width, height);
        let (cols, rows) = calc_grid(width, height);

        for tab in self.tabs.iter_mut() {
          if tab.grid.cols == cols && tab.grid.rows == rows {
            continue;
          }
          tab.grid.resize(cols, rows);
          if let Some(tx) = &tab.pty_tx {
            let _ = tx.send_blocking(PtyCommand::Resize {
              cols: cols as u16,
              rows: rows as u16,
            });
          }
        }
      }
      Message::CursorMoved(position) => {
        self.cursor_position = position;
      }
      Message::MousePressed => {
        if let Some(window_id) = self.window_id {
          if let Some(direction) = resize_direction(self.cursor_position, self.window_size) {
            return window::drag_resize(window_id, direction);
          }
        }
      }
      Message::PasteRequested => {
        return iced::clipboard::read().map(Message::ClipboardReceived);
      }
      Message::ClipboardReceived(text) => {
        if let Some(text) = text {
          if let Some(tab) = self.tabs.get(self.active_index) {
            if let Some(tx) = &tab.pty_tx {
              let _ = tx.try_send(PtyCommand::Input(text.into_bytes()));
            }
          }
        }
      }
      Message::Tick => {}
    }

    iced::Task::none()
  }

  pub fn view(&self) -> Element<'_, Message> {
    let active_tab = &self.tabs[self.active_index];

    components::app(column![
      components::title_bar(self.window_focused, &active_tab.pwd),
      components::tab_bar(&self.tabs, self.active_index),
      components::term(active_tab),
      components::status_bar(active_tab)
    ])
  }

  pub fn theme(&self) -> Theme {
    Theme::Dracula
  }

  pub fn subscription(&self) -> Subscription<Message> {
    let mut subs = Vec::new();

    let time_sub = time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick);

    subs.push(time_sub);

    let global_sub = event::listen_with(|event, _s, window_id| match event {
      Event::Keyboard(keyboard::Event::KeyPressed {
        key,
        modifiers,
        modified_key,
        ..
      }) => {
        match &key {
          Key::Named(Named::Enter) => return Some(Message::Type(b"\r".to_vec())),
          Key::Named(Named::Backspace) => return Some(Message::Type(b"\x7F".to_vec())),
          Key::Named(Named::Tab) if modifiers.control() && modifiers.shift() => {
            return Some(Message::PrevTab);
          }
          Key::Named(Named::Tab) if modifiers.control() => {
            return Some(Message::NextTab);
          }
          Key::Named(Named::Tab) => return Some(Message::Type(b"\t".to_vec())),
          Key::Named(Named::Space) => return Some(Message::Type(b" ".to_vec())),
          Key::Named(Named::Escape) => return Some(Message::Type(b"\x1b".to_vec())),
          Key::Named(Named::ArrowUp) => return Some(Message::Type(b"\x1b[A".to_vec())),
          Key::Named(Named::ArrowDown) => return Some(Message::Type(b"\x1b[B".to_vec())),
          Key::Named(Named::ArrowRight) => return Some(Message::Type(b"\x1b[C".to_vec())),
          Key::Named(Named::ArrowLeft) => return Some(Message::Type(b"\x1b[D".to_vec())),
          Key::Named(Named::Delete) => return Some(Message::Type(b"\x1b[3~".to_vec())),
          Key::Named(Named::Home) => return Some(Message::Type(b"\x1b[H".to_vec())),
          Key::Named(Named::End) => return Some(Message::Type(b"\x1b[F".to_vec())),
          Key::Named(Named::PageUp) => return Some(Message::Type(b"\x1b[5~".to_vec())),
          Key::Named(Named::PageDown) => return Some(Message::Type(b"\x1b[6~".to_vec())),
          _ => {}
        }

        if modifiers.control() {
          if let Key::Character(c) = &key {
            if let Some(ch) = c.as_str().chars().next() {
              if ch.is_ascii_alphabetic() {
                let lower = ch.to_ascii_lowercase();
                match lower {
                  'v' => return Some(Message::PasteRequested),
                  'w' => return Some(Message::CloseActiveTab),
                  't' => return Some(Message::NewTab),
                  _ => return Some(Message::Type(vec![(lower as u8) & 0x1f])),
                }
              }
            }
          }
          return None;
        }

        let char_source = match &modified_key {
          Key::Character(_) => &modified_key,
          _ => &key,
        };

        if let Key::Character(c) = char_source {
          let mut s = c.as_str().to_string();
          if modifiers.shift() {
            if s == "'" {
              s = "\"".to_string();
            }
            if s == "`" {
              s = "~".to_string();
            }
          }
          return Some(Message::Type(s.into_bytes()));
        }

        None
      }
      Event::Window(window::Event::Opened { .. }) => {
        return Some(Message::WindowOpened(window_id));
      }
      Event::Window(window::Event::Focused) => {
        return Some(Message::WindowFocused);
      }
      Event::Window(window::Event::Unfocused) => {
        return Some(Message::WindowUnfocused);
      }
      Event::Window(window::Event::Resized(size)) => {
        return Some(Message::WindowResized(size.width, size.height));
      }
      Event::Mouse(mouse::Event::CursorMoved { position }) => {
        return Some(Message::CursorMoved(position));
      }
      Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
        return Some(Message::MousePressed);
      }
      _ => None,
    });

    subs.push(global_sub);

    for tab in &self.tabs {
      let tab_id = tab.id;
      let cols = tab.grid.cols as u16;
      let rows = tab.grid.rows as u16;
      let pty_sub = Subscription::run_with((tab_id, cols, rows), |(tab_id, cols, rows)| {
        pty_worker(*tab_id, *cols, *rows)
      });
      subs.push(pty_sub);
    }

    Subscription::batch(subs)
  }
}

const RESIZE_EDGE: f32 = 8.0;

fn resize_direction(pos: Point, size: Size) -> Option<window::Direction> {
  let left = pos.x < RESIZE_EDGE;
  let right = pos.x > size.width - RESIZE_EDGE;
  let top = pos.y < RESIZE_EDGE;
  let bottom = pos.y > size.height - RESIZE_EDGE;

  match (top, bottom, left, right) {
    (true, _, true, _) => Some(window::Direction::NorthWest),
    (true, _, _, true) => Some(window::Direction::NorthEast),
    (_, true, true, _) => Some(window::Direction::SouthWest),
    (_, true, _, true) => Some(window::Direction::SouthEast),
    (true, _, false, false) => Some(window::Direction::North),
    (_, true, false, false) => Some(window::Direction::South),
    (false, false, true, _) => Some(window::Direction::West),
    (false, false, _, true) => Some(window::Direction::East),
    _ => None,
  }
}

fn pty_worker(tab_id: usize, cols: u16, rows: u16) -> impl iced::futures::Stream<Item = Message> {
  stream::channel(
    100,
    move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
      use iced::futures::SinkExt;

      let (tx_out, rx_out) = async_channel::unbounded::<Vec<u8>>();
      let (tx_in, rx_in) = async_channel::unbounded::<PtyCommand>();

      std::thread::spawn(move || {
        let mut pty = PtyBridge::new(tx_out, cols, rows).expect("failed to create PTY bridge");

        while let Ok(command) = rx_in.recv_blocking() {
          match command {
            PtyCommand::Input(bytes) => pty.write_to_pty(&bytes),
            PtyCommand::Resize { cols, rows } => pty.resize_pty(cols, rows),
          }
        }
      });

      let _ = output.send(Message::PtyReady(tab_id, tx_in)).await;

      while let Ok(bytes) = rx_out.recv().await {
        let _ = output.send(Message::PtyOutputReceived(tab_id, bytes)).await;
      }
    },
  )
}
