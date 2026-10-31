use std::time::Instant;

use iced::widget::{button, container, row, text, text_input};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding, Shadow};

use crate::ui::app_state::Message;
use crate::ui::components::ds::button::icon_button_style;
use crate::ui::components::ds::focus_within::focus_within;
use crate::ui::components::ds::kbd::{kbd, mod_label};
use crate::ui::components::ds::transform::{Transform, press};
use crate::ui::components::session::input_id;
use crate::ui::helpers::til_home;
use crate::ui::icons::{Icon, icon};
use crate::ui::theme::motion::{self, lerp};
use crate::ui::theme::tokens::{self, duration, height, radius, space};
use crate::ui::theme::{color, font};

pub fn input_bar_style(focused: bool) -> container::Style {
  let accent = color::runtime().accent;
  container::Style {
    background: Some(Background::Color(if focused {
      tokens::INPUT_FOCUS
    } else {
      tokens::SURFACE_2
    })),
    border: Border {
      radius: radius::R12.into(),
      width: 1.0,
      color: if focused {
        tokens::with_alpha(accent, 0.48)
      } else {
        tokens::LINE
      },
    },
    shadow: if focused {
      Shadow {
        color: tokens::with_alpha(accent, 0.12),
        offset: iced::Vector::ZERO,
        blur_radius: space::S8,
      }
    } else {
      Shadow::default()
    },
    ..container::Style::default()
  }
}

pub fn input_bar_input_style(status: text_input::Status) -> text_input::Style {
  let accent = color::runtime().accent;
  let disabled = matches!(status, text_input::Status::Disabled);
  text_input::Style {
    background: Background::Color(Color::TRANSPARENT),
    border: Border::default(),
    icon: tokens::FG_3,
    placeholder: tokens::FG_3,
    value: if disabled { tokens::FG_3 } else { tokens::FG_1 },
    selection: tokens::with_alpha(accent, 0.32),
  }
}

pub fn pulse_scale(pulsed_at: Option<Instant>, now: Instant) -> f32 {
  let Some(at) = pulsed_at else {
    return 1.0;
  };
  let t = now.saturating_duration_since(at).as_secs_f32() / duration::D2.as_secs_f32();
  if t >= 1.0 {
    return 1.0;
  }
  lerp(0.8, 1.0, motion::spring(t))
}

pub fn ask_ai_button<'a>(compact: bool) -> Element<'a, Message> {
  let fg = tokens::FG_2;
  let mut content = row![icon(Icon::Sparkle, 16.0, fg)]
    .spacing(space::S8)
    .align_y(Alignment::Center);
  if !compact {
    content = content
      .push(
        text("Ask AI")
          .size(tokens::text::T14)
          .font(font::semibold()),
      )
      .push(kbd(format!("{}K", mod_label())));
  }
  press(
    button(container(content).center_y(Length::Fill))
      .height(height::H40)
      .padding(Padding::from([0.0, space::S16]))
      .style(|_t, s| {
        crate::ui::components::ds::button::style(
          crate::ui::components::ds::button::Variant::Ghost,
          s,
        )
      })
      .on_press(Message::OpenAskAi),
  )
  .into()
}

pub struct InputBarView<'a> {
  pub pane_id: usize,
  pub cwd: &'a str,
  pub value: &'a str,
  pub running: bool,
  pub pulsed_at: Option<Instant>,
  pub now: Instant,
  pub font_size: f32,
  pub compact: bool,
}

pub fn prompt<'a>(cwd: &str, size: f32) -> Element<'a, Message> {
  let accent = color::runtime().accent;
  row![
    text(til_home(cwd)).size(size).color(tokens::FG_3),
    text("$").size(size).font(font::semibold()).color(accent),
  ]
  .spacing(space::S8)
  .align_y(Alignment::Center)
  .into()
}

pub fn input_bar<'a>(v: InputBarView<'a>) -> Element<'a, Message> {
  let input = text_input("", v.value)
    .id(input_id(v.pane_id))
    .on_input_maybe((!v.running).then_some(Message::InputChanged))
    .on_submit(Message::InputSubmit)
    .font(font::regular())
    .size(v.font_size)
    .padding(0)
    .width(Length::Fill)
    .style(|_t, s| input_bar_input_style(s));

  let enter = Transform::new(
    button(
      container(icon(Icon::Enter, 20.0, tokens::FG_2))
        .center_x(Length::Fill)
        .center_y(Length::Fill),
    )
    .width(height::H40)
    .height(height::H40)
    .padding(0)
    .style(|_t, s| icon_button_style(s))
    .on_press(Message::InputSubmit),
  )
  .scale(pulse_scale(v.pulsed_at, v.now))
  .press();

  let bar = container(
    row![
      prompt(v.cwd, v.font_size),
      input,
      ask_ai_button(v.compact),
      enter,
    ]
    .spacing(space::S12)
    .align_y(Alignment::Center),
  )
  .height(height::H56)
  .padding(Padding {
    top: 0.0,
    right: space::S8,
    bottom: 0.0,
    left: space::S16,
  })
  .center_y(height::H56)
  .width(Length::Fill);

  container(focus_within(bar, input_bar_style))
    .padding(Padding {
      top: 0.0,
      right: space::S16,
      bottom: space::S16,
      left: space::S16,
    })
    .width(Length::Fill)
    .into()
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::time::Duration;

  #[test]
  fn pulse_starts_small_and_settles() {
    let t0 = Instant::now();
    assert_eq!(pulse_scale(None, t0), 1.0);
    assert!((pulse_scale(Some(t0), t0) - 0.8).abs() < 1e-3);
    assert_eq!(pulse_scale(Some(t0), t0 + Duration::from_millis(250)), 1.0);
  }

  #[test]
  fn focused_bar_lifts_background() {
    assert_eq!(
      input_bar_style(true).background,
      Some(Background::Color(tokens::INPUT_FOCUS))
    );
    assert_eq!(input_bar_style(false).border.color, tokens::LINE);
  }
}
