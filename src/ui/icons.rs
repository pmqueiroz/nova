use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use iced::Color;
use iced::widget::svg::{self, Svg};

static HANDLES: LazyLock<Mutex<HashMap<Icon, svg::Handle>>> =
  LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Icon {
  Enter,
  Plus,
  Close,
  Sparkle,
  Sliders,
  Search,
  Clear,
  ZoomIn,
  ZoomOut,
  Clock,
  Alert,
  Back,
  Help,
  Terminal,
  Minimize,
  Maximize,
}

impl Icon {
  pub fn path(self) -> &'static str {
    match self {
      Icon::Enter => "M20 5v7a3 3 0 0 1-3 3H5M9 11l-4 4 4 4",
      Icon::Plus => "M12 5v14M5 12h14",
      Icon::Close => "M6 6l12 12M18 6L6 18",
      Icon::Sparkle => "M12 3l1.8 6.2L20 11l-6.2 1.8L12 19l-1.8-6.2L4 11l6.2-1.8z",
      Icon::Sliders => "M4 7h10M18 7h2M4 17h4M12 17h8M16 5v4M10 15v4",
      Icon::Search => "M11 18a7 7 0 1 0 0-14 7 7 0 0 0 0 14zM20 20l-4-4",
      Icon::Clear => "M4 6h16M4 12h10M4 18h6",
      Icon::ZoomIn => "M11 18a7 7 0 1 0 0-14 7 7 0 0 0 0 14zM20 20l-4-4M8 11h6M11 8v6",
      Icon::ZoomOut => "M11 18a7 7 0 1 0 0-14 7 7 0 0 0 0 14zM20 20l-4-4M8 11h6",
      Icon::Clock => "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM12 7v5l3 2",
      Icon::Alert => "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM12 8v5M12 16v.01",
      Icon::Back => "M15 6l-6 6 6 6",
      Icon::Help => {
        "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM9.5 9.5a2.5 2.5 0 1 1 3.5 2.3c-.6.3-1 .9-1 1.6V14M12 17v.01"
      }
      Icon::Terminal => "M4 6l6 6-6 6M12 18h8",
      Icon::Minimize => "M6 12h12",
      Icon::Maximize => "M6 6h12v12H6z",
    }
  }

  pub fn markup(self) -> String {
    format!(
      r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="{}"/></svg>"##,
      self.path()
    )
  }

  pub fn handle(self) -> svg::Handle {
    HANDLES
      .lock()
      .unwrap()
      .entry(self)
      .or_insert_with(|| svg::Handle::from_memory(self.markup().into_bytes()))
      .clone()
  }
}

pub fn icon<'a>(name: Icon, size: f32, color: Color) -> Svg<'a> {
  Svg::new(name.handle())
    .width(size)
    .height(size)
    .style(move |_theme, _status| svg::Style { color: Some(color) })
}

#[cfg(test)]
mod tests {
  use super::*;

  const ALL: [Icon; 16] = [
    Icon::Enter,
    Icon::Plus,
    Icon::Close,
    Icon::Sparkle,
    Icon::Sliders,
    Icon::Search,
    Icon::Clear,
    Icon::ZoomIn,
    Icon::ZoomOut,
    Icon::Clock,
    Icon::Alert,
    Icon::Back,
    Icon::Help,
    Icon::Terminal,
    Icon::Minimize,
    Icon::Maximize,
  ];

  #[test]
  fn markup_is_stroked_svg() {
    for i in ALL {
      let m = i.markup();
      assert!(m.starts_with("<svg"));
      assert!(m.contains("stroke-width=\"2\""));
      assert!(m.contains(i.path()));
    }
  }

  #[test]
  fn handles_are_cached() {
    assert_eq!(Icon::Plus.handle().id(), Icon::Plus.handle().id());
  }
}
