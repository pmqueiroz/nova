use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

static CONFIG: OnceLock<Config> = OnceLock::new();
static PARSED_KB: OnceLock<std::sync::Mutex<ParsedKeybindings>> = OnceLock::new();

#[cfg(target_os = "macos")]
const DEFAULT_CONFIG: &str = include_str!("../../assets/default_settings_macos.toml");
#[cfg(not(target_os = "macos"))]
const DEFAULT_CONFIG: &str = include_str!("../../assets/default_settings.toml");

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
  pub general: GeneralConfig,
  #[serde(rename = "status-bar")]
  pub status_bar: StatusBarConfig,
  pub theme: ThemeConfig,
  pub keybindings: KeybindingsConfig,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GeneralConfig {
  pub editor: String,
  pub bell: BellType,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub shells: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StatusBarConfig {
  pub visible: bool,
  #[serde(rename = "date-format")]
  pub date_format: String,
  #[serde(rename = "time-format")]
  pub time_format: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BellType {
  None,
  Audio,
  Blink,
}

impl std::fmt::Display for BellType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      BellType::None => write!(f, "None"),
      BellType::Audio => write!(f, "Audio"),
      BellType::Blink => write!(f, "Blink"),
    }
  }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ThemeConfig {
  pub font: FontConfig,
  pub colors: ThemeColorsConfig,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FontConfig {
  pub size: f32,
  pub family: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ThemeColorsConfig {
  pub background: String,
  pub foreground: String,
  pub accent: String,
  #[serde(rename = "foreground-muted")]
  pub foreground_muted: String,
  pub border: String,
  pub cursor: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
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
  pub copy: String,
}

pub struct ParsedKeybinding {
  pub ctrl: bool,
  pub shift: bool,
  pub alt: bool,
  pub meta: bool,
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
  pub copy: ParsedKeybinding,
}

pub fn parse_keybinding(s: &str) -> anyhow::Result<ParsedKeybinding> {
  let parts: Vec<&str> = s.split('+').collect();
  let (mut ctrl, mut shift, mut alt, mut meta) = (false, false, false, false);
  let mut key_part = "";
  for p in &parts {
    match p.to_ascii_lowercase().as_str() {
      "ctrl" => ctrl = true,
      "shift" => shift = true,
      "alt" | "option" => alt = true,
      "cmd" | "command" | "meta" | "super" => meta = true,
      _ => key_part = p,
    }
  }
  let key = match key_part.to_ascii_lowercase().as_str() {
    "tab" => KeyId::Tab,
    c if c.len() == 1 => KeyId::Char(c.chars().next().unwrap()),
    _ => return Err(anyhow::anyhow!("unknown key in keybinding: '{}'", key_part)),
  };
  Ok(ParsedKeybinding { ctrl, shift, alt, meta, key })
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

pub fn config_path() -> Option<std::path::PathBuf> {
  dirs::config_dir().map(|d| d.join("nova").join("settings.toml"))
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
  let config: Config = match toml::from_str(&content) {
    Ok(c) => c,
    Err(_) => {
      std::fs::write(&path, DEFAULT_CONFIG)?;
      toml::from_str(DEFAULT_CONFIG).expect("embedded default config is invalid")
    }
  };

  let parsed = build_parsed_keybindings(&config)?;

  CONFIG.set(config).ok();
  PARSED_KB.set(std::sync::Mutex::new(parsed)).ok();

  Ok(())
}

fn build_parsed_keybindings(config: &Config) -> anyhow::Result<ParsedKeybindings> {
  Ok(ParsedKeybindings {
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
    copy: parse_keybinding(&config.keybindings.copy)
      .map_err(|e| anyhow::anyhow!("keybinding 'copy': {}", e))?,
  })
}

pub fn get() -> &'static Config {
  CONFIG.get().expect("config not initialized")
}

pub fn keybindings() -> std::sync::MutexGuard<'static, ParsedKeybindings> {
  PARSED_KB.get().expect("keybindings not initialized").lock().unwrap()
}

pub fn update_keybindings(new: ParsedKeybindings) {
  if let Some(m) = PARSED_KB.get() {
    *m.lock().unwrap() = new;
  }
}

pub fn available_shells() -> Vec<String> {
  if let Some(shells) = &get().general.shells {
    if !shells.is_empty() {
      return shells.clone();
    }
  }
  detect_shells()
}

pub fn detect_shells() -> Vec<String> {
  #[cfg(target_os = "windows")]
  {
    let mut shells = vec!["powershell".to_string()];
    let has_pwsh = std::process::Command::new("where")
      .arg("pwsh")
      .output()
      .map(|o| o.status.success())
      .unwrap_or(false);
    if has_pwsh {
      shells.push("pwsh".to_string());
    }
    shells.push("cmd".to_string());
    shells
  }
  #[cfg(not(target_os = "windows"))]
  {
    if let Ok(content) = std::fs::read_to_string("/etc/shells") {
      let shells: Vec<String> = content
        .lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .map(String::from)
        .collect();
      if !shells.is_empty() {
        return shells;
      }
    }
    vec![std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string())]
  }
}

pub fn save(config: &Config) -> anyhow::Result<()> {
  let path = config_path().ok_or_else(|| anyhow::anyhow!("cannot determine config path"))?;
  let content = toml::to_string_pretty(config)?;
  std::fs::write(path, content)?;
  Ok(())
}

pub fn default_config_str() -> &'static str {
  DEFAULT_CONFIG
}

pub fn reset_to_defaults() -> anyhow::Result<Config> {
  let path = config_path().ok_or_else(|| anyhow::anyhow!("cannot determine config path"))?;
  std::fs::write(&path, DEFAULT_CONFIG)?;
  let config: Config = toml::from_str(DEFAULT_CONFIG).expect("embedded default config is invalid");
  Ok(config)
}

pub fn reload_parsed_keybindings(config: &Config) -> anyhow::Result<()> {
  let parsed = build_parsed_keybindings(config)?;
  update_keybindings(parsed);
  Ok(())
}
