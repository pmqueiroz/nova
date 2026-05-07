use serde::Deserialize;
use std::sync::OnceLock;

static CONFIG: OnceLock<Config> = OnceLock::new();
static PARSED_KB: OnceLock<ParsedKeybindings> = OnceLock::new();

const DEFAULT_CONFIG: &str = include_str!("../../assets/default_settings.toml");

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
  #[serde(rename = "status-bar")]
  pub status_bar: StatusBarConfig,
  #[allow(dead_code)]
  pub bell: BellConfig,
  pub theme: ThemeConfig,
  pub keybindings: KeybindingsConfig,
}

#[derive(Deserialize, Debug, Clone)]
pub struct StatusBarConfig {
  pub visible: bool,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct BellConfig {
  #[serde(rename = "type")]
  pub bell_type: BellType,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum BellType {
  Audio,
  Blink,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ThemeConfig {
  pub font: FontConfig,
  pub colors: ThemeColorsConfig,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FontConfig {
  pub size: f32,
  #[allow(dead_code)]
  pub family: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ThemeColorsConfig {
  pub background: String,
  pub foreground: String,
  pub accent: String,
  #[serde(rename = "foreground-muted")]
  pub foreground_muted: String,
  pub border: String,
  pub cursor: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct KeybindingsConfig {
  #[serde(rename = "new-tab")]
  pub new_tab: String,
  #[serde(rename = "close-tab")]
  pub close_tab: String,
  #[serde(rename = "next-tab")]
  pub next_tab: String,
  #[serde(rename = "prev-tab")]
  pub prev_tab: String,
  pub paste: String,
}

pub struct ParsedKeybinding {
  pub ctrl: bool,
  pub shift: bool,
  pub alt: bool,
  pub key: KeyId,
}

pub enum KeyId {
  Char(char),
  Tab,
}

pub struct ParsedKeybindings {
  pub new_tab: ParsedKeybinding,
  pub close_tab: ParsedKeybinding,
  pub next_tab: ParsedKeybinding,
  pub prev_tab: ParsedKeybinding,
  pub paste: ParsedKeybinding,
}

fn parse_keybinding(s: &str) -> anyhow::Result<ParsedKeybinding> {
  let parts: Vec<&str> = s.split('+').collect();
  let (mut ctrl, mut shift, mut alt) = (false, false, false);
  let mut key_part = "";
  for p in &parts {
    match p.to_ascii_lowercase().as_str() {
      "ctrl" => ctrl = true,
      "shift" => shift = true,
      "alt" => alt = true,
      _ => key_part = p,
    }
  }
  let key = match key_part.to_ascii_lowercase().as_str() {
    "tab" => KeyId::Tab,
    c if c.len() == 1 => KeyId::Char(c.chars().next().unwrap()),
    _ => return Err(anyhow::anyhow!("unknown key in keybinding: '{}'", key_part)),
  };
  Ok(ParsedKeybinding { ctrl, shift, alt, key })
}

pub fn parse_hex_color(hex: &str) -> anyhow::Result<iced::Color> {
  let h = hex.trim_start_matches('#');
  match h.len() {
    6 => {
      let r = u8::from_str_radix(&h[0..2], 16)?;
      let g = u8::from_str_radix(&h[2..4], 16)?;
      let b = u8::from_str_radix(&h[4..6], 16)?;
      Ok(iced::Color::from_rgb8(r, g, b))
    }
    8 => {
      let r = u8::from_str_radix(&h[0..2], 16)?;
      let g = u8::from_str_radix(&h[2..4], 16)?;
      let b = u8::from_str_radix(&h[4..6], 16)?;
      let a = u8::from_str_radix(&h[6..8], 16)?;
      Ok(iced::Color {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: a as f32 / 255.0,
      })
    }
    _ => Err(anyhow::anyhow!("invalid hex color: '{}'", hex)),
  }
}

pub fn init() -> anyhow::Result<()> {
  let config_dir = dirs::config_dir()
    .ok_or_else(|| anyhow::anyhow!("cannot determine config directory"))?
    .join("nova");

  std::fs::create_dir_all(&config_dir)?;
  let path = config_dir.join("settings.toml");

  if !path.exists() {
    std::fs::write(&path, DEFAULT_CONFIG)?;
  }

  let content = std::fs::read_to_string(&path)?;
  let config: Config = toml::from_str(&content)
    .map_err(|e| anyhow::anyhow!("config error in {}: {}", path.display(), e))?;

  let parsed = ParsedKeybindings {
    new_tab: parse_keybinding(&config.keybindings.new_tab)
      .map_err(|e| anyhow::anyhow!("keybinding 'new-tab': {}", e))?,
    close_tab: parse_keybinding(&config.keybindings.close_tab)
      .map_err(|e| anyhow::anyhow!("keybinding 'close-tab': {}", e))?,
    next_tab: parse_keybinding(&config.keybindings.next_tab)
      .map_err(|e| anyhow::anyhow!("keybinding 'next-tab': {}", e))?,
    prev_tab: parse_keybinding(&config.keybindings.prev_tab)
      .map_err(|e| anyhow::anyhow!("keybinding 'prev-tab': {}", e))?,
    paste: parse_keybinding(&config.keybindings.paste)
      .map_err(|e| anyhow::anyhow!("keybinding 'paste': {}", e))?,
  };

  CONFIG.set(config).ok();
  PARSED_KB.set(parsed).ok();

  Ok(())
}

pub fn get() -> &'static Config {
  CONFIG.get().expect("config not initialized")
}

pub fn keybindings() -> &'static ParsedKeybindings {
  PARSED_KB.get().expect("keybindings not initialized")
}
