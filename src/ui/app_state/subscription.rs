use super::helpers::matches_kb;
use super::message::Message;
use super::nova::{
  ACTIVE_KITTY_FLAGS, AI_OPEN, KB_RECORDING, Nova, PALETTE_OPEN, SEARCH_OPEN, SETTINGS_OPEN,
};

use iced::keyboard::Key;
use iced::keyboard::key::Named;
use iced::{Event, Subscription, event, keyboard, mouse, stream, time, window};
use std::sync::atomic::Ordering;

use crate::core::config;
use crate::sys::pty::{PtyBridge, PtyCommand};

struct PtyKey {
  tab_id: usize,
  shell_cmd: String,
  initial_cols: u16,
  initial_rows: u16,
  initial_cwd: String,
}

impl std::hash::Hash for PtyKey {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.tab_id.hash(state);
    self.shell_cmd.hash(state);
  }
}

impl PartialEq for PtyKey {
  fn eq(&self, other: &Self) -> bool {
    self.tab_id == other.tab_id && self.shell_cmd == other.shell_cmd
  }
}

impl Eq for PtyKey {}

fn notification_click_stream() -> impl iced::futures::Stream<Item = Message> {
  stream::channel(
    1,
    |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
      use iced::futures::SinkExt;
      let rx = crate::sys::notification::click_receiver();
      while rx.recv().await.is_ok() {
        let _ = output.send(Message::NotificationActivated).await;
      }
    },
  )
}

fn pty_worker(
  tab_id: usize,
  cols: u16,
  rows: u16,
  shell: String,
  initial_cwd: String,
) -> impl iced::futures::Stream<Item = Message> {
  stream::channel(
    100,
    move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
      use iced::futures::SinkExt;

      let (tx_out, rx_out) = async_channel::unbounded::<Vec<u8>>();
      let (tx_in, rx_in) = async_channel::unbounded::<PtyCommand>();

      std::thread::spawn(move || {
        let cwd = if initial_cwd.is_empty() || initial_cwd == "~" {
          None
        } else {
          Some(initial_cwd.as_str())
        };
        let mut pty =
          PtyBridge::new(tx_out, cols, rows, &shell, cwd).expect("failed to create PTY bridge");

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

      let _ = output.send(Message::PtyExited(tab_id)).await;
    },
  )
}

fn kitty_mod_bits(modifiers: keyboard::Modifiers) -> u8 {
  let mut m = 0u8;
  if modifiers.shift() {
    m |= 1;
  }
  if modifiers.alt() {
    m |= 2;
  }
  if modifiers.control() {
    m |= 4;
  }
  if modifiers.logo() {
    m |= 8;
  }
  m
}

fn kitty_csi_letter(letter: char, modifiers: keyboard::Modifiers) -> Vec<u8> {
  let m = kitty_mod_bits(modifiers);
  if m == 0 {
    format!("\x1b[{}", letter).into_bytes()
  } else {
    format!("\x1b[1;{}{}", m + 1, letter).into_bytes()
  }
}

fn kitty_csi_tilde(num: u8, modifiers: keyboard::Modifiers) -> Vec<u8> {
  let m = kitty_mod_bits(modifiers);
  if m == 0 {
    format!("\x1b[{}~", num).into_bytes()
  } else {
    format!("\x1b[{};{}~", num, m + 1).into_bytes()
  }
}

fn kitty_csi_u(codepoint: u32, modifiers: keyboard::Modifiers) -> Vec<u8> {
  let m = kitty_mod_bits(modifiers);
  if m == 0 {
    format!("\x1b[{}u", codepoint).into_bytes()
  } else {
    format!("\x1b[{};{}u", codepoint, m + 1).into_bytes()
  }
}

fn kitty_f1_f4(letter: char, modifiers: keyboard::Modifiers) -> Vec<u8> {
  let m = kitty_mod_bits(modifiers);
  if m == 0 {
    format!("\x1bO{}", letter).into_bytes()
  } else {
    format!("\x1b[1;{}{}", m + 1, letter).into_bytes()
  }
}

fn handle_key_pressed(
  key: Key,
  modifiers: keyboard::Modifiers,
  modified_key: Key,
) -> Option<Message> {
  let kitty_flags = ACTIVE_KITTY_FLAGS.load(Ordering::Relaxed);
  let use_kitty = kitty_flags != 0;
  if KB_RECORDING.load(Ordering::SeqCst) {
    return match &key {
      Key::Named(Named::Escape) => Some(Message::SettingsCancelRecordKb),
      _ => Some(Message::SettingsRecordKb {
        key: key.clone(),
        modifiers,
      }),
    };
  }
  if SETTINGS_OPEN.load(Ordering::SeqCst) {
    return match &key {
      Key::Named(Named::Escape) => Some(Message::CloseSettings),
      _ => None,
    };
  }
  if AI_OPEN.load(Ordering::SeqCst) {
    return match &key {
      Key::Named(Named::Escape) => Some(Message::CloseAiOverlay),
      _ => None,
    };
  }
  if PALETTE_OPEN.load(Ordering::SeqCst) {
    return match &key {
      Key::Named(Named::Escape) => Some(Message::CloseCommandPalette),
      Key::Named(Named::ArrowUp) => Some(Message::PaletteNavigate(-1)),
      Key::Named(Named::ArrowDown) => Some(Message::PaletteNavigate(1)),
      Key::Named(Named::Enter) => Some(Message::PaletteConfirm),
      _ => None,
    };
  }
  if SEARCH_OPEN.load(Ordering::SeqCst) {
    return match &key {
      Key::Named(Named::Escape) => Some(Message::SearchClose),
      Key::Named(Named::Enter) => {
        if modifiers.shift() {
          Some(Message::SearchPrev)
        } else {
          Some(Message::SearchNext)
        }
      }
      _ => None,
    };
  }

  #[cfg(target_os = "macos")]
  let split_mod = modifiers.logo() && modifiers.shift();
  #[cfg(not(target_os = "macos"))]
  let split_mod = modifiers.control() && modifiers.shift();

  if split_mod && let Key::Character(c) = &key {
    match c.as_str() {
      "t" | "T" => return Some(Message::SplitPane),
      "w" | "W" => return Some(Message::CloseSplitPane),
      _ => {}
    }
  }

  #[cfg(target_os = "macos")]
  let font_mod = modifiers.logo() && !modifiers.shift();
  #[cfg(not(target_os = "macos"))]
  let font_mod = modifiers.control() && !modifiers.shift();

  if font_mod && let Key::Character(c) = &key {
    match c.as_str() {
      "=" | "+" => return Some(Message::FontSizeUp),
      "-" => return Some(Message::FontSizeDown),
      "0" => return Some(Message::FontSizeReset),
      "f" | "F" => return Some(Message::SearchOpen),
      _ => {}
    }
  }

  let kb = config::keybindings();
  if matches_kb(&kb.prev_tab, &key, modifiers) {
    return Some(Message::PrevTab);
  }
  if matches_kb(&kb.next_tab, &key, modifiers) {
    return Some(Message::NextTab);
  }
  if matches_kb(&kb.new_tab, &key, modifiers) {
    return Some(Message::NewTab);
  }
  if matches_kb(&kb.close_tab, &key, modifiers) {
    return Some(Message::CloseActiveTab);
  }
  if matches_kb(&kb.paste, &key, modifiers) {
    return Some(Message::PasteRequested);
  }
  if matches_kb(&kb.copy, &key, modifiers) {
    return Some(Message::CopySelection);
  }
  if matches_kb(&kb.open_palette, &key, modifiers) {
    return Some(Message::OpenCommandPalette);
  }
  drop(kb);

  match &key {
    Key::Named(Named::Enter) => {
      return Some(Message::Type(
        if use_kitty && kitty_mod_bits(modifiers) != 0 {
          kitty_csi_u(13, modifiers)
        } else {
          b"\r".to_vec()
        },
      ));
    }
    Key::Named(Named::Backspace) => {
      return Some(Message::Type(
        if use_kitty && kitty_mod_bits(modifiers) != 0 {
          kitty_csi_u(127, modifiers)
        } else {
          b"\x7F".to_vec()
        },
      ));
    }
    Key::Named(Named::Tab) => {
      return Some(Message::Type(if use_kitty && modifiers.shift() {
        kitty_csi_u(9, modifiers)
      } else if !use_kitty && modifiers.shift() {
        b"\x1b[Z".to_vec()
      } else {
        b"\t".to_vec()
      }));
    }
    Key::Named(Named::Space) => {
      return Some(Message::Type(
        if use_kitty && kitty_mod_bits(modifiers) != 0 {
          kitty_csi_u(32, modifiers)
        } else {
          b" ".to_vec()
        },
      ));
    }
    Key::Named(Named::Escape) => {
      return Some(Message::Type(if use_kitty && kitty_flags & 1 != 0 {
        kitty_csi_u(27, keyboard::Modifiers::default())
      } else {
        b"\x1b".to_vec()
      }));
    }
    Key::Named(Named::ArrowUp) => {
      return Some(Message::Type(if use_kitty {
        kitty_csi_letter('A', modifiers)
      } else {
        b"\x1b[A".to_vec()
      }));
    }
    Key::Named(Named::ArrowDown) => {
      return Some(Message::Type(if use_kitty {
        kitty_csi_letter('B', modifiers)
      } else {
        b"\x1b[B".to_vec()
      }));
    }
    #[cfg(target_os = "macos")]
    Key::Named(Named::ArrowRight) => {
      return Some(Message::Type(if use_kitty {
        kitty_csi_letter('C', modifiers)
      } else if modifiers.logo() {
        b"\x05".to_vec()
      } else if modifiers.alt() || modifiers.control() {
        b"\x1bf".to_vec()
      } else {
        b"\x1b[C".to_vec()
      }));
    }
    #[cfg(not(target_os = "macos"))]
    Key::Named(Named::ArrowRight) => {
      return Some(Message::Type(if use_kitty {
        kitty_csi_letter('C', modifiers)
      } else if modifiers.alt() {
        b"\x05".to_vec()
      } else if modifiers.control() {
        b"\x1bf".to_vec()
      } else {
        b"\x1b[C".to_vec()
      }));
    }
    #[cfg(target_os = "macos")]
    Key::Named(Named::ArrowLeft) => {
      return Some(Message::Type(if use_kitty {
        kitty_csi_letter('D', modifiers)
      } else if modifiers.logo() {
        b"\x01".to_vec()
      } else if modifiers.alt() || modifiers.control() {
        b"\x1bb".to_vec()
      } else {
        b"\x1b[D".to_vec()
      }));
    }
    #[cfg(not(target_os = "macos"))]
    Key::Named(Named::ArrowLeft) => {
      return Some(Message::Type(if use_kitty {
        kitty_csi_letter('D', modifiers)
      } else if modifiers.alt() {
        b"\x01".to_vec()
      } else if modifiers.control() {
        b"\x1bb".to_vec()
      } else {
        b"\x1b[D".to_vec()
      }));
    }
    Key::Named(Named::Delete) => return Some(Message::Type(kitty_csi_tilde(3, modifiers))),
    Key::Named(Named::Insert) => return Some(Message::Type(kitty_csi_tilde(2, modifiers))),
    Key::Named(Named::Home) => {
      return Some(Message::Type(if use_kitty {
        kitty_csi_letter('H', modifiers)
      } else {
        b"\x1b[H".to_vec()
      }));
    }
    Key::Named(Named::End) => {
      return Some(Message::Type(if use_kitty {
        kitty_csi_letter('F', modifiers)
      } else {
        b"\x1b[F".to_vec()
      }));
    }
    Key::Named(Named::PageUp) => return Some(Message::Type(kitty_csi_tilde(5, modifiers))),
    Key::Named(Named::PageDown) => return Some(Message::Type(kitty_csi_tilde(6, modifiers))),
    Key::Named(Named::F1) => return Some(Message::Type(kitty_f1_f4('P', modifiers))),
    Key::Named(Named::F2) => return Some(Message::Type(kitty_f1_f4('Q', modifiers))),
    Key::Named(Named::F3) => return Some(Message::Type(kitty_f1_f4('R', modifiers))),
    Key::Named(Named::F4) => return Some(Message::Type(kitty_f1_f4('S', modifiers))),
    Key::Named(Named::F5) => return Some(Message::Type(kitty_csi_tilde(15, modifiers))),
    Key::Named(Named::F6) => return Some(Message::Type(kitty_csi_tilde(17, modifiers))),
    Key::Named(Named::F7) => return Some(Message::Type(kitty_csi_tilde(18, modifiers))),
    Key::Named(Named::F8) => return Some(Message::Type(kitty_csi_tilde(19, modifiers))),
    Key::Named(Named::F9) => return Some(Message::Type(kitty_csi_tilde(20, modifiers))),
    Key::Named(Named::F10) => return Some(Message::Type(kitty_csi_tilde(21, modifiers))),
    Key::Named(Named::F11) => return Some(Message::Type(kitty_csi_tilde(23, modifiers))),
    Key::Named(Named::F12) => return Some(Message::Type(kitty_csi_tilde(24, modifiers))),
    _ => {}
  }

  if modifiers.control() {
    if let Key::Character(c) = &key
      && let Some(ch) = c.as_str().chars().next()
      && ch.is_ascii_alphabetic()
    {
      if use_kitty && modifiers.shift() {
        let lower = ch.to_ascii_lowercase();
        return Some(Message::Type(kitty_csi_u(lower as u32, modifiers)));
      }
      let lower = ch.to_ascii_lowercase();
      return Some(Message::Type(vec![(lower as u8) & 0x1f]));
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

impl Nova {
  pub fn subscription(&self) -> Subscription<Message> {
    let mut subs = Vec::new();

    subs.push(time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick));
    subs.push(Subscription::run(notification_click_stream));
    subs.push(time::every(std::time::Duration::from_millis(500)).map(|_| Message::CursorBlinkTick));

    if self.bell_blink_remaining > 0 {
      subs.push(time::every(std::time::Duration::from_millis(200)).map(|_| Message::BellBlinkTick));
    }

    let kitty_flags = self
      .tabs
      .get(self.active_index)
      .map(|tab| {
        if tab.active_pane_is_split {
          tab
            .split
            .as_ref()
            .map(|s| s.grid.kitty_keyboard_flags())
            .unwrap_or(0)
        } else {
          tab.grid.kitty_keyboard_flags()
        }
      })
      .unwrap_or(0);
    ACTIVE_KITTY_FLAGS.store(kitty_flags, Ordering::Relaxed);

    subs.push(event::listen_with(|event, _s, window_id| match event {
      Event::Keyboard(keyboard::Event::KeyPressed {
        key,
        modifiers,
        modified_key,
        ..
      }) => handle_key_pressed(key, modifiers, modified_key),
      Event::Window(window::Event::Opened { .. }) => Some(Message::WindowOpened(window_id)),
      Event::Window(window::Event::Focused) => Some(Message::WindowFocused),
      Event::Window(window::Event::Unfocused) => Some(Message::WindowUnfocused),
      Event::Window(window::Event::Resized(size)) => {
        Some(Message::WindowResized(size.width, size.height))
      }
      Event::Mouse(mouse::Event::CursorMoved { position }) => Some(Message::CursorMoved(position)),
      Event::Mouse(mouse::Event::ButtonPressed(button)) => Some(Message::MousePressed(button)),
      Event::Mouse(mouse::Event::ButtonReleased(button)) => Some(Message::MouseReleased(button)),
      Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
        let lines = match delta {
          mouse::ScrollDelta::Lines { y, .. } => y,
          mouse::ScrollDelta::Pixels { y, .. } => y / 20.0,
        };
        if lines != 0.0 {
          Some(Message::Scroll(lines))
        } else {
          None
        }
      }
      Event::Keyboard(keyboard::Event::ModifiersChanged(mods)) => {
        Some(Message::ModifiersChanged(mods))
      }
      _ => None,
    }));

    for tab in &self.tabs {
      if tab.pty_alive {
        let key = PtyKey {
          tab_id: tab.id,
          shell_cmd: tab.shell_cmd.clone(),
          initial_cols: tab.grid.cols as u16,
          initial_rows: tab.grid.rows as u16,
          initial_cwd: tab.initial_cwd.clone(),
        };
        subs.push(Subscription::run_with(key, |k| {
          pty_worker(
            k.tab_id,
            k.initial_cols,
            k.initial_rows,
            k.shell_cmd.clone(),
            k.initial_cwd.clone(),
          )
        }));
      }

      if let Some(split) = &tab.split
        && split.pty_alive
      {
        let split_key = PtyKey {
          tab_id: split.id,
          shell_cmd: split.shell_cmd.clone(),
          initial_cols: split.grid.cols as u16,
          initial_rows: split.grid.rows as u16,
          initial_cwd: split.initial_cwd.clone(),
        };
        subs.push(Subscription::run_with(split_key, |k| {
          pty_worker(
            k.tab_id,
            k.initial_cols,
            k.initial_rows,
            k.shell_cmd.clone(),
            k.initial_cwd.clone(),
          )
        }));
      }
    }

    Subscription::batch(subs)
  }
}
