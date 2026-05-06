use async_channel::Sender;
use iced::keyboard::Key;
use iced::keyboard::key::Named;
use iced::widget::column;
use iced::{Element, Subscription, Theme, time, window};
use iced::{Event, event, keyboard, stream};

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
}

#[derive(Debug, Clone)]
pub enum Message {
  Type(Vec<u8>),
  NewTab,
  SwitchTab(usize),
  CloseTab(usize),
  PtyReady(usize, Sender<PtyCommand>),
  PtyOutputReceived(usize, Vec<u8>),
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
  Tick,
}

impl Default for Nova {
  fn default() -> Self {
    Self {
      tabs: vec![Tab::new(0)],
      active_index: 0,
      next_tab_id: 1,
      window_id: None,
      window_focused: false,
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
        let font_size = 16.0_f32;
        let char_width = font_size * 0.62;  // slight safety margin over 0.6
        let char_height = font_size * 1.35; // iced line height is larger than 1.2x font size

        let padding_x = 40.0; // term.rs: left(20) + right(20)
        let padding_y = 118.0; // title(40) + tab(36) + status(22) + term vertical pad(20)

        let new_cols = ((width - padding_x) / char_width).floor() as usize;
        let new_rows = ((height - padding_y) / char_height).floor() as usize;

        let cols = new_cols.max(10);
        let rows = new_rows.max(5);

        for tab in self.tabs.iter_mut() {
          tab.grid.resize(cols, rows);
          if let Some(tx) = &tab.pty_tx {
            let _ = tx.send_blocking(PtyCommand::Resize {
              cols: cols as u16,
              rows: rows as u16,
            });
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
        match key {
          Key::Named(Named::Enter) => return Some(Message::Type(b"\r".to_vec())),
          Key::Named(Named::Backspace) => return Some(Message::Type(b"\x7F".to_vec())),
          Key::Named(Named::Space) => return Some(Message::Type(b" ".to_vec())),
          Key::Named(Named::ArrowUp) => return Some(Message::Type(b"\x1b[A".to_vec())),
          Key::Named(Named::ArrowDown) => return Some(Message::Type(b"\x1b[B".to_vec())),
          Key::Named(Named::ArrowRight) => return Some(Message::Type(b"\x1b[C".to_vec())),
          Key::Named(Named::ArrowLeft) => return Some(Message::Type(b"\x1b[D".to_vec())),
          Key::Named(Named::Delete) => return Some(Message::Type(b"\x1b[3~".to_vec())),
          _ => {}
        }

        if let Key::Character(c) = modified_key {
          if !modifiers.control() {
            return Some(Message::Type(c.as_str().as_bytes().to_vec()));
          }
        }

        if let Key::Character(c) = key {
          let mut char_str = c.as_str().to_string();

          // TODO: This is a bit of a hack to handle shifted characters. Ideally, we'd want to get the actual character that would be typed with the modifiers applied, but iced doesn't provide that directly.
          if char_str == "'" && modifiers.shift() {
            char_str = "\"".to_string();
          }
          if char_str == "`" && modifiers.shift() {
            char_str = "~".to_string();
          }
          if char_str == "v" && modifiers.control() {
            return Some(Message::PasteRequested);
          }

          return Some(Message::Type(char_str.as_bytes().to_vec()));
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
