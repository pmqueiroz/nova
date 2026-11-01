use std::sync::atomic::Ordering;

use crate::core::config;

use super::super::helpers::{keybinding_to_string, rebuild_runtime_theme};
use super::super::nova::{ColorField, KB_RECORDING, Nova};

impl Nova {
  pub(super) fn handle_settings_color_changed(&mut self, field: ColorField, hex: String) {
    if config::parse_hex_color(&hex).is_ok() {
      match field {
        ColorField::Accent => {
          self.settings.theme.colors.cursor = hex.clone();
          self.settings.theme.colors.accent = hex;
        }
      }
      let _ = config::save(&self.settings);
      rebuild_runtime_theme(&self.settings.theme.colors);
    }
  }

  pub(super) fn handle_settings_record_kb(
    &mut self,
    key: iced::keyboard::Key,
    modifiers: iced::keyboard::Modifiers,
  ) {
    if let Some(idx) = self.settings_recording_index
      && let Some(s) = keybinding_to_string(&key, modifiers)
    {
      match idx {
        0 => self.settings.keybindings.new_tab = s,
        1 => self.settings.keybindings.close_tab = s,
        2 => self.settings.keybindings.next_tab = s,
        3 => self.settings.keybindings.prev_tab = s,
        4 => self.settings.keybindings.paste = s,
        5 => self.settings.keybindings.copy = s,
        6 => self.settings.keybindings.open_palette = s,
        7 => self.settings.keybindings.ask_ai = s,
        8 => self.settings.keybindings.settings = s,
        _ => {}
      }
      let _ = config::save(&self.settings);
      let _ = config::reload_parsed_keybindings(&self.settings);
      self.settings_recording_index = None;
      KB_RECORDING.store(false, Ordering::SeqCst);
    }
  }
}
