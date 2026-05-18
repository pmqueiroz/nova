use iced::{
  Border, Color, Element, Padding,
  border::Radius,
  widget::{button, column, container, pick_list, row, text, text_input, toggler},
};

use super::{btn_subtle_style, color_row, input_style, section_label, setting_row};
use crate::core::config::{self, CursorStyle, WindowControls};
use crate::ui::{
  app_state::{ColorField, Message},
  theme,
};

pub fn theme_tab<'a>(settings: &'a config::Config) -> Element<'a, Message> {
  let mut col = column![].spacing(24);

  let font_size = settings.theme.font.size;
  let fg = theme::color::runtime().foreground;

  let family_input: Element<'a, Message> = column![
    text_input("font family", &settings.theme.font.family)
      .on_input(Message::SettingsFontFamilyChanged)
      .font(theme::font::REGULAR)
      .size(12)
      .style(input_style)
      .padding(Padding {
        top: 7.0,
        bottom: 5.0,
        left: 10.0,
        right: 10.0
      }),
    text("Requires restart to take effect")
      .font(theme::font::REGULAR)
      .size(10)
      .color(theme::color::runtime().foreground_muted),
  ]
  .spacing(4)
  .into();

  let size_control: Element<'a, Message> = row![
    button(text("−").size(14).color(fg))
      .style(btn_subtle_style)
      .on_press(Message::SettingsFontSizeChanged(font_size - 1.0))
      .padding(Padding {
        top: 5.0,
        bottom: 3.0,
        left: 10.0,
        right: 10.0
      }),
    container(
      text(format!("{}", font_size as u32))
        .font(theme::font::REGULAR)
        .size(12)
        .color(fg),
    )
    .padding(Padding {
      top: 5.0,
      bottom: 3.0,
      left: 12.0,
      right: 12.0
    })
    .style(|_| {
      let border_c = theme::color::runtime().border;
      container::Style {
        background: Some(theme::color::BG_HIGH.as_color().into()),
        border: Border {
          color: border_c,
          radius: Radius::new(4.0),
          width: 1.0,
        },
        ..Default::default()
      }
    }),
    button(text("+").size(14).color(fg))
      .style(btn_subtle_style)
      .on_press(Message::SettingsFontSizeChanged(font_size + 1.0))
      .padding(Padding {
        top: 5.0,
        bottom: 3.0,
        left: 10.0,
        right: 10.0
      }),
  ]
  .spacing(4)
  .align_y(iced::alignment::Vertical::Center)
  .into();

  col = col.push(
    column![
      section_label("FONT"),
      column![
        setting_row("Family", "Monospace font family", family_input),
        setting_row("Size", "Font size in points", size_control),
      ]
      .spacing(16),
    ]
    .spacing(8),
  );

  let controls_list: Element<'a, Message> = pick_list(
    [WindowControls::TrafficLights, WindowControls::System].as_slice(),
    Some(settings.general.window_controls.clone()),
    Message::SettingsWindowControlsChanged,
  )
  .font(theme::font::REGULAR)
  .text_size(12)
  .padding(Padding {
    top: 7.0,
    bottom: 5.0,
    left: 10.0,
    right: 10.0,
  })
  .into();

  col = col.push(
    column![
      section_label("WINDOW"),
      setting_row(
        "Controls",
        "Style of the window control buttons",
        controls_list
      ),
    ]
    .spacing(8),
  );

  let c = &settings.theme.colors;
  col = col.push(
    column![
      section_label("COLORS"),
      column![
        color_row("Background", &c.background, ColorField::Background),
        color_row("Foreground", &c.foreground, ColorField::Foreground),
        color_row("Accent", &c.accent, ColorField::Accent),
        color_row(
          "Foreground muted",
          &c.foreground_muted,
          ColorField::ForegroundMuted
        ),
        color_row("Border", &c.border, ColorField::Border),
        color_row("Cursor", &c.cursor, ColorField::Cursor),
      ]
      .spacing(12),
    ]
    .spacing(8),
  );

  let cursor = &settings.theme.cursor;
  let cursor_style_list: Element<'a, Message> = pick_list(
    [
      CursorStyle::Block,
      CursorStyle::Beam,
      CursorStyle::Underline,
    ]
    .as_slice(),
    Some(cursor.style),
    Message::SettingsCursorStyleChanged,
  )
  .font(theme::font::REGULAR)
  .text_size(12)
  .into();

  let blink_toggle: Element<'a, Message> = toggler(cursor.blink)
    .on_toggle(Message::SettingsCursorBlinkToggled)
    .size(20)
    .style(move |_t, status| {
      let is_toggled = match status {
        toggler::Status::Active { is_toggled }
        | toggler::Status::Hovered { is_toggled }
        | toggler::Status::Disabled { is_toggled } => is_toggled,
      };
      let rt = theme::color::runtime();
      let (accent, border_c) = (rt.accent, rt.border);
      drop(rt);
      toggler::Style {
        background: if is_toggled {
          accent.into()
        } else {
          theme::color::BG_HIGH.as_color().into()
        },
        background_border_width: 1.0,
        background_border_color: if is_toggled { accent } else { border_c },
        foreground: Color::WHITE.into(),
        foreground_border_width: 0.0,
        foreground_border_color: Color::TRANSPARENT,
        text_color: None,
        border_radius: None,
        padding_ratio: 0.15,
      }
    })
    .into();

  col = col.push(
    column![
      section_label("CURSOR"),
      column![
        setting_row("Style", "Cursor shape", cursor_style_list),
        setting_row("Blink", "Animate cursor blinking", blink_toggle),
      ]
      .spacing(12),
    ]
    .spacing(8),
  );

  col.into()
}
