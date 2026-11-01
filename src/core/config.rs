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
  pub theme: ThemeConfig,
  pub keybindings: KeybindingsConfig,
  #[serde(default)]
  pub ai: AiConfig,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AiProvider {
  #[default]
  Anthropic,
  OpenAi,
}

impl std::fmt::Display for AiProvider {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      AiProvider::Anthropic => write!(f, "Anthropic"),
      AiProvider::OpenAi => write!(f, "OpenAI"),
    }
  }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AiConfig {
  pub provider: AiProvider,
  pub model: String,
  pub api_key: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub base_url: Option<String>,
  #[serde(default = "default_true")]
  pub diagnostic_banner: bool,
}

fn default_true() -> bool {
  true
}

impl Default for AiConfig {
  fn default() -> Self {
    Self {
      provider: AiProvider::Anthropic,
      model: "claude-haiku-4-5-20251001".into(),
      api_key: String::new(),
      base_url: None,
      diagnostic_banner: true,
    }
  }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum WindowControls {
  TrafficLights,
  System,
}

impl std::fmt::Display for WindowControls {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      WindowControls::TrafficLights => write!(f, "Traffic Lights"),
      WindowControls::System => write!(f, "System"),
    }
  }
}

fn default_window_controls() -> WindowControls {
  #[cfg(target_os = "macos")]
  return WindowControls::TrafficLights;
  #[cfg(not(target_os = "macos"))]
  return WindowControls::System;
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GeneralConfig {
  pub editor: String,
  pub bell: BellType,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub shells: Option<Vec<String>>,
  #[serde(rename = "window-controls", default = "default_window_controls")]
  pub window_controls: WindowControls,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub scrollback: Option<usize>,
  #[serde(
    rename = "initial-command",
    default,
    skip_serializing_if = "Option::is_none"
  )]
  pub initial_command: Option<String>,
  #[serde(rename = "wait-after-command", default)]
  pub wait_after_command: bool,
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

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CursorStyle {
  Block,
  Beam,
  Underline,
}

impl std::fmt::Display for CursorStyle {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      CursorStyle::Block => write!(f, "Block"),
      CursorStyle::Beam => write!(f, "Beam"),
      CursorStyle::Underline => write!(f, "Underline"),
    }
  }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CursorConfig {
  pub style: CursorStyle,
  pub blink: bool,
}

impl Default for CursorConfig {
  fn default() -> Self {
    Self {
      style: CursorStyle::Underline,
      blink: true,
    }
  }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ThemeConfig {
  pub font: FontConfig,
  pub colors: ThemeColorsConfig,
  #[serde(default)]
  pub cursor: CursorConfig,
  #[serde(default)]
  pub timestamps: bool,
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
  #[serde(rename = "open-palette", default = "default_open_palette")]
  pub open_palette: String,
  #[serde(rename = "ask-ai", default = "default_ask_ai")]
  pub ask_ai: String,
  #[serde(default = "default_settings_key")]
  pub settings: String,
}

pub const MOD: &str = if cfg!(target_os = "macos") {
  "cmd"
} else {
  "ctrl"
};

pub const LEGACY_ACCENT: &str = "#3ECF8E";
pub const DEFAULT_ACCENT: &str = "#82BAFF";

fn default_open_palette() -> String {
  format!("{MOD}+p")
}

fn default_ask_ai() -> String {
  format!("{MOD}+k")
}

fn default_settings_key() -> String {
  format!("{MOD}+,")
}

pub fn migrate(config: &mut Config) -> bool {
  let mut changed = false;
  if config.keybindings.open_palette == format!("{MOD}+k")
    && config.keybindings.ask_ai == default_ask_ai()
  {
    config.keybindings.open_palette = default_open_palette();
    changed = true;
  }
  if config
    .theme
    .colors
    .accent
    .eq_ignore_ascii_case(LEGACY_ACCENT)
  {
    config.theme.colors.accent = DEFAULT_ACCENT.to_string();
    if config
      .theme
      .colors
      .cursor
      .eq_ignore_ascii_case(LEGACY_ACCENT)
    {
      config.theme.colors.cursor = DEFAULT_ACCENT.to_string();
    }
    changed = true;
  }
  changed
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
  pub open_palette: ParsedKeybinding,
  pub ask_ai: ParsedKeybinding,
  pub settings: ParsedKeybinding,
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
  Ok(ParsedKeybinding {
    ctrl,
    shift,
    alt,
    meta,
    key,
  })
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

  let mut config = config;
  if migrate(&mut config) {
    let _ = std::fs::write(&path, toml::to_string_pretty(&config).unwrap_or_default());
  }

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
    open_palette: parse_keybinding(&config.keybindings.open_palette)
      .unwrap_or_else(|_| parse_keybinding(&default_open_palette()).unwrap()),
    ask_ai: parse_keybinding(&config.keybindings.ask_ai)
      .unwrap_or_else(|_| parse_keybinding(&default_ask_ai()).unwrap()),
    settings: parse_keybinding(&config.keybindings.settings)
      .unwrap_or_else(|_| parse_keybinding(&default_settings_key()).unwrap()),
  })
}

pub fn get() -> &'static Config {
  CONFIG.get().expect("config not initialized")
}

pub fn keybindings() -> std::sync::MutexGuard<'static, ParsedKeybindings> {
  PARSED_KB
    .get()
    .expect("keybindings not initialized")
    .lock()
    .unwrap()
}

pub fn update_keybindings(new: ParsedKeybindings) {
  if let Some(m) = PARSED_KB.get() {
    *m.lock().unwrap() = new;
  }
}

pub fn available_shells() -> Vec<String> {
  if let Some(shells) = &get().general.shells
    && !shells.is_empty()
  {
    return shells.clone();
  }
  detect_shells()
}

pub fn detect_shells() -> Vec<String> {
  #[cfg(target_os = "windows")]
  {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let mut shells = vec!["powershell".to_string()];

    let has_pwsh = std::process::Command::new("where")
      .arg("pwsh")
      .creation_flags(CREATE_NO_WINDOW)
      .output()
      .map(|o| o.status.success())
      .unwrap_or(false);
    if has_pwsh {
      shells.push("pwsh".to_string());
    }

    shells.push("cmd".to_string());

    let has_wsl = std::process::Command::new("wsl")
      .args(["--list", "--quiet"])
      .creation_flags(CREATE_NO_WINDOW)
      .output()
      .map(|o| o.status.success())
      .unwrap_or(false);
    if has_wsl {
      shells.push("wsl".to_string());
    }

    let git_bash_paths = [
      r"C:\Program Files\Git\bin\bash.exe",
      r"C:\Program Files (x86)\Git\bin\bash.exe",
      r"C:\Git\bin\bash.exe",
    ];
    if git_bash_paths
      .iter()
      .any(|p| std::path::Path::new(p).exists())
    {
      shells.push("git-bash".to_string());
    }

    shells
  }
  #[cfg(not(target_os = "windows"))]
  {
    let mut shells = Vec::new();
    let env_shell = std::env::var("SHELL").unwrap_or_default();
    if !env_shell.trim().is_empty() {
      shells.push(env_shell.trim().to_string());
    }

    if let Ok(content) = std::fs::read_to_string("/etc/shells") {
      let etc_shells: Vec<String> = content
        .lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .map(String::from)
        .collect();

      for shell in etc_shells {
        if !shells.contains(&shell) {
          shells.push(shell);
        }
      }
    }

    if shells.is_empty() {
      shells.push("bash".to_string());
    }
    shells
  }
}

pub fn save(config: &Config) -> anyhow::Result<()> {
  let path = config_path().ok_or_else(|| anyhow::anyhow!("cannot determine config path"))?;
  let content = toml::to_string_pretty(config)?;
  std::fs::write(path, content)?;
  Ok(())
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default_config_parses() {
    toml::from_str::<Config>(DEFAULT_CONFIG).unwrap();
  }

  #[test]
  fn defaults_use_design_accent_and_bindings() {
    let c: Config = toml::from_str(DEFAULT_CONFIG).unwrap();
    assert_eq!(c.theme.colors.accent, DEFAULT_ACCENT);
    assert_eq!(c.keybindings.open_palette, default_open_palette());
    assert_eq!(c.keybindings.ask_ai, default_ask_ai());
    assert_eq!(c.keybindings.settings, default_settings_key());
  }

  #[test]
  fn migrates_legacy_palette_binding_and_accent() {
    let legacy = DEFAULT_CONFIG
      .replace(DEFAULT_ACCENT, LEGACY_ACCENT)
      .replace(
        &format!("open-palette = \"{MOD}+p\""),
        &format!("open-palette = \"{MOD}+k\""),
      )
      .replace(&format!("ask-ai = \"{MOD}+k\"\n"), "");
    let mut c: Config = toml::from_str(&legacy).unwrap();
    assert!(migrate(&mut c));
    assert_eq!(c.keybindings.open_palette, default_open_palette());
    assert_eq!(c.theme.colors.accent, DEFAULT_ACCENT);
    assert_eq!(c.theme.colors.cursor, DEFAULT_ACCENT);
    assert!(!migrate(&mut c));
  }

  #[test]
  fn custom_accent_is_kept() {
    let mut c: Config = toml::from_str(DEFAULT_CONFIG).unwrap();
    c.theme.colors.accent = "#FF00FF".into();
    migrate(&mut c);
    assert_eq!(c.theme.colors.accent, "#FF00FF");
  }

  #[test]
  fn timestamps_default_off_for_old_configs() {
    let c: Config = toml::from_str(DEFAULT_CONFIG).unwrap();
    assert!(!c.theme.timestamps);
  }

  #[test]
  fn legacy_status_bar_section_is_ignored() {
    let legacy = format!(
      "{DEFAULT_CONFIG}\n[status-bar]\nvisible = true\ndate-format = \"%b %d\"\ntime-format = \"%H:%M:%S\"\n"
    );
    toml::from_str::<Config>(&legacy).unwrap();
  }
}
