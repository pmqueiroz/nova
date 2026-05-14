use std::sync::atomic::Ordering;

use crate::core::config;

use super::super::helpers::{keybinding_to_string, rebuild_runtime_theme};
use super::super::nova::{ColorField, KB_RECORDING, Nova};

impl Nova {
  pub(super) fn handle_settings_color_changed(&mut self, field: ColorField, hex: String) {
    if config::parse_hex_color(&hex).is_ok() {
      match field {
        ColorField::Background => self.settings.theme.colors.background = hex,
        ColorField::Foreground => self.settings.theme.colors.foreground = hex,
        ColorField::Accent => self.settings.theme.colors.accent = hex,
        ColorField::ForegroundMuted => self.settings.theme.colors.foreground_muted = hex,
        ColorField::Border => self.settings.theme.colors.border = hex,
        ColorField::Cursor => self.settings.theme.colors.cursor = hex,
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
        _ => {}
      }
      let _ = config::save(&self.settings);
      let _ = config::reload_parsed_keybindings(&self.settings);
      self.settings_recording_index = None;
      KB_RECORDING.store(false, Ordering::SeqCst);
    }
  }

  pub(super) fn handle_settings_reset_kb(&mut self, idx: usize) {
    let default_cfg: config::Config =
      toml::from_str(config::default_config_str()).expect("invalid default config");
    match idx {
      0 => self.settings.keybindings.new_tab = default_cfg.keybindings.new_tab,
      1 => self.settings.keybindings.close_tab = default_cfg.keybindings.close_tab,
      2 => self.settings.keybindings.next_tab = default_cfg.keybindings.next_tab,
      3 => self.settings.keybindings.prev_tab = default_cfg.keybindings.prev_tab,
      4 => self.settings.keybindings.paste = default_cfg.keybindings.paste,
      5 => self.settings.keybindings.copy = default_cfg.keybindings.copy,
      _ => {}
    }
    let _ = config::save(&self.settings);
    let _ = config::reload_parsed_keybindings(&self.settings);
  }
}
