use std::sync::{OnceLock, RwLock};

static FONT_FAMILY: OnceLock<RwLock<&'static str>> = OnceLock::new();
static INSTALLED_FONTS: OnceLock<Vec<String>> = OnceLock::new();

pub fn installed_fonts() -> &'static [String] {
  INSTALLED_FONTS.get_or_init(|| {
    use font_kit::source::SystemSource;
    let mut fonts = SystemSource::new().all_families().unwrap_or_default();
    fonts.sort_unstable();
    fonts.dedup();
    fonts
  })
}

fn get_family() -> &'static str {
  *FONT_FAMILY
    .get_or_init(|| RwLock::new("FiraCode Nerd Font"))
    .read()
    .unwrap()
}

pub fn set_family(name: &str) {
  let leaked: &'static str = Box::leak(name.to_string().into_boxed_str());
  *FONT_FAMILY
    .get_or_init(|| RwLock::new("FiraCode Nerd Font"))
    .write()
    .unwrap() = leaked;
}

pub fn regular() -> iced::Font {
  iced::Font {
    family: iced::font::Family::Name(get_family()),
    weight: iced::font::Weight::Normal,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
  }
}

pub fn bold() -> iced::Font {
  iced::Font {
    family: iced::font::Family::Name(get_family()),
    weight: iced::font::Weight::Bold,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
  }
}

pub fn italic() -> iced::Font {
  iced::Font {
    family: iced::font::Family::Name(get_family()),
    weight: iced::font::Weight::Normal,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Italic,
  }
}

pub fn bold_italic() -> iced::Font {
  iced::Font {
    family: iced::font::Family::Name(get_family()),
    weight: iced::font::Weight::Bold,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Italic,
  }
}
