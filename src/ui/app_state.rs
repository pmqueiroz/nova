use async_channel::Sender;
use iced::keyboard::Key;
use iced::keyboard::key::Named;
use iced::widget::column;
use iced::{Element, Subscription, Theme, time, window};
use iced::{Event, event, keyboard, stream};

use crate::sys::parser::AnsiExecutor;
use crate::sys::pty::PtyBridge;
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
  PtyReady(usize, Sender<Vec<u8>>),
  PtyOutputReceived(usize, Vec<u8>),
  CloseWindow,
  MinimizeWindow,
  MaximizeWindow,
  DragWindow,
  WindowOpened(window::Id),
  WindowFocused,
  WindowUnfocused,
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

          while !tab.grid.output_queue.is_empty() {
            let response = tab.grid.output_queue.remove(0);

            if let Some(tx) = &tab.pty_tx {
              let _ = tx.send_blocking(response);
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

          // TODO: This is a bit of a hack to handle shifted characters. Ideally, we'd want to get the actual character that would be typed with the modifiers applied, but iced doesn't provide that directly.
          if char_str == "'" && modifiers.shift() {
            char_str = "\"".to_string();
          }
          if char_str == "`" && modifiers.shift() {
            char_str = "~".to_string();
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
      _ => None,
    });

    subs.push(global_sub);

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
