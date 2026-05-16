use super::helpers::{dir_to_cursor, normalize_sel, resize_direction, strip_markdown};
use super::message::Message;
use super::nova::Nova;

use iced::alignment::{Horizontal, Vertical};
use iced::mouse;
use iced::widget::{button, column, container, mouse_area, row, stack, text};
use iced::{Border, Color, Element, Length, Padding, Theme, border::Radius};

use crate::ui::components;
use crate::ui::theme;

impl Nova {
  pub fn view(&self) -> Element<'_, Message> {
    let active_tab = &self.tabs[self.active_index];

    let selection = match (self.selection_start, self.selection_end) {
      (Some(start), Some(end)) if start != end => {
        let ((sc, sr), (ec, er)) = normalize_sel(start, end);
        Some((sc, sr, ec, er))
      }
      _ => None,
    };

    let font_size = self.settings.theme.font.size;
    let resize_cursor = resize_direction(self.cursor_position, self.window_size).map(dir_to_cursor);

    let term_interaction = resize_cursor.unwrap_or_else(|| {
      if self.hovered_url.is_some() {
        mouse::Interaction::Pointer
      } else {
        mouse::Interaction::Text
      }
    });

    let term_area: Element<'_, Message> = if let Some(split) = &active_tab.split {
      let rt = theme::color::runtime();
      let border_color = rt.border;
      let accent = rt.accent;
      let fg_muted = rt.foreground_muted;
      drop(rt);

      let primary_selection = if !active_tab.active_pane_is_split {
        selection
      } else {
        None
      };
      let primary_url = if !active_tab.active_pane_is_split {
        self.hovered_url.as_deref()
      } else {
        None
      };
      let primary_span = if !active_tab.active_pane_is_split {
        self.hovered_link_span
      } else {
        None
      };

      let make_close_btn = move |msg: Message| {
        container(
          button(text("\u{2715}").size(10).color(fg_muted))
            .on_press(msg)
            .padding(Padding::from([2, 6]))
            .style(move |_t, status| button::Style {
              text_color: match status {
                button::Status::Hovered | button::Status::Pressed => theme::color::RED.as_color(),
                _ => fg_muted,
              },
              background: None,
              ..Default::default()
            }),
        )
        .align_x(Horizontal::Right)
        .align_y(Vertical::Top)
        .padding(Padding::from([4, 4]))
        .width(Length::Fill)
        .height(Length::Fill)
      };

      let make_active_triangle = move |is_active: bool| {
        const SIZE: u32 = 16;
        let fill = if is_active {
          format!(
            "#{:02x}{:02x}{:02x}",
            (accent.r * 255.0) as u8,
            (accent.g * 255.0) as u8,
            (accent.b * 255.0) as u8,
          )
        } else {
          "none".to_string()
        };
        let svg_data = format!(
          r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {s} {s}"><polygon points="0,0 {s},0 0,{s}" fill="{fill}"/></svg>"#,
          s = SIZE,
          fill = fill,
        );
        let handle = iced::widget::svg::Handle::from_memory(svg_data.into_bytes());
        container(iced::widget::svg(handle).width(SIZE).height(SIZE))
          .align_x(Horizontal::Left)
          .align_y(Vertical::Top)
          .width(Length::Fill)
          .height(Length::Fill)
      };

      let left = mouse_area(
        container(stack![
          components::term(
            &active_tab.grid,
            primary_selection,
            font_size,
            active_tab.scroll_offset,
            primary_url,
            primary_span,
          ),
          make_active_triangle(!active_tab.active_pane_is_split),
          make_close_btn(Message::CloseLeftPane),
        ])
        .style(move |_| {
          let border_width = if !active_tab.active_pane_is_split {
            1.0
          } else {
            0.0
          };
          container::Style {
            border: Border {
              color: accent,
              width: border_width,
              radius: Radius::new(0.0),
            },
            ..Default::default()
          }
        })
        .width(Length::Fill)
        .height(Length::Fill),
      )
      .interaction(term_interaction);

      let divider = container(iced::widget::Space::new())
        .width(1)
        .height(Length::Fill)
        .style(move |_| container::Style {
          background: Some(border_color.into()),
          ..Default::default()
        });

      let right = mouse_area(
        container(stack![
          components::term(
            &split.grid,
            None,
            font_size,
            split.scroll_offset,
            None,
            None,
          ),
          make_active_triangle(active_tab.active_pane_is_split),
          make_close_btn(Message::CloseSplitPane),
        ])
        .style(move |_| {
          let border_width = if active_tab.active_pane_is_split {
            1.0
          } else {
            0.0
          };
          container::Style {
            border: Border {
              color: accent,
              width: border_width,
              radius: Radius::new(0.0),
            },
            ..Default::default()
          }
        })
        .width(Length::Fill)
        .height(Length::Fill),
      )
      .interaction(term_interaction);

      row![left, divider, right].height(Length::Fill).into()
    } else {
      mouse_area(components::term(
        &active_tab.grid,
        selection,
        font_size,
        active_tab.scroll_offset,
        self.hovered_url.as_deref(),
        self.hovered_link_span,
      ))
      .interaction(term_interaction)
      .into()
    };

    let tb_interaction = resize_cursor.unwrap_or(mouse::Interaction::Idle);

    let mut col = column![
      components::title_bar(
        self.window_focused,
        &active_tab.pwd,
        self.window_maximized,
        tb_interaction,
        &self.settings.general.window_controls,
        self.bell_blink_visible,
      ),
      components::tab_bar(
        &self.tabs,
        self.active_index,
        self.editing_tab_index,
        &self.editing_tab_title,
      ),
      term_area,
    ];

    if let Some((_code, ref message, ref command)) = self.diagnostic_banner {
      col = col.push(self.diagnostic_banner_widget(message, command.as_deref()));
    }

    if self.settings.status_bar.visible {
      col = col.push(components::status_bar(
        active_tab,
        &self.settings.status_bar.date_format,
        &self.settings.status_bar.time_format,
        self.window_maximized,
      ));
    }

    let outer_interaction = resize_cursor.unwrap_or(mouse::Interaction::Idle);

    let inner = if self.settings_open {
      let config_path_str = crate::core::config::config_path()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
      let modal = components::settings_modal(
        &self.settings,
        &self.settings_tab,
        &self.settings_shell_input,
        self.settings_recording_index,
        &self.raw_config_content,
        config_path_str,
      );
      components::app(stack![col, modal], self.window_maximized)
    } else if self.command_palette_open {
      let palette = components::command_palette(&self.palette_query, self.palette_selected);
      components::app(stack![col, palette], self.window_maximized)
    } else if self.ai_overlay_open || self.ai_loading {
      let overlay = components::ai_overlay(
        &self.ai_input,
        self.ai_response.as_deref(),
        self.ai_loading,
        self.ai_is_error,
      );
      components::app(stack![col, overlay], self.window_maximized)
    } else if self.shell_picker_open {
      let picker = components::shell_picker(
        &self.available_shells,
        self.shell_picker_anchor,
        self.window_size.width,
      );
      components::app(stack![col, picker], self.window_maximized)
    } else {
      components::app(col, self.window_maximized)
    };

    mouse_area(inner).interaction(outer_interaction).into()
  }

  pub fn theme(&self) -> Theme {
    let rt = crate::ui::theme::color::runtime();
    Theme::custom(
      "Nova",
      iced::theme::Palette {
        background: rt.background,
        text: rt.foreground,
        primary: rt.accent,
        success: rt.accent,
        warning: iced::Color::from_rgb(1.0, 0.75, 0.0),
        danger: iced::Color::from_rgb(0.9, 0.3, 0.3),
      },
    )
  }

  fn diagnostic_banner_widget<'a>(
    &self,
    message: &'a str,
    command: Option<&'a str>,
  ) -> iced::Element<'a, Message> {
    let rt = theme::color::runtime();
    let bg = rt.background;
    let accent = rt.accent;
    let fg = rt.foreground;
    drop(rt);

    let mut inner = column![].spacing(6);
    inner = inner.push(
      text(" \u{2726} NOVA \u{00B7} AI ")
        .font(theme::font::BOLD)
        .size(12)
        .color(accent),
    );
    inner = inner.push(
      text(format!(" {}", strip_markdown(message)))
        .font(theme::font::REGULAR)
        .size(12)
        .color(fg),
    );
    if let Some(cmd) = command {
      let cmd_text = cmd.to_string();
      inner = inner.push(
        button(
          text(format!(" {} ", cmd_text))
            .font(theme::font::REGULAR)
            .size(12)
            .color(accent),
        )
        .on_press(Message::DiagnosticBannerCommand(cmd_text))
        .padding(Padding::from([4, 10]))
        .style(move |_t, _s| button::Style {
          background: Some(Color { a: 0.08, ..accent }.into()),
          border: Border {
            color: accent,
            radius: Radius::new(4.0),
            width: 0.0,
          },
          text_color: accent,
          ..Default::default()
        }),
      );
    }

    container(
      container(inner)
        .padding(Padding::from([8, 12]))
        .style(move |_| container::Style {
          background: Some(Color { a: 0.08, ..accent }.into()),
          border: Border {
            color: accent,
            radius: Radius::new(8.0),
            width: 1.0,
          },
          ..Default::default()
        })
        .width(Length::Fill),
    )
    .padding(Padding::from([8, 8]))
    .style(move |_| container::Style {
      background: Some(bg.into()),
      ..Default::default()
    })
    .width(Length::Fill)
    .into()
  }
}
