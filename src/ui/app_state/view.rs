use super::helpers::{dir_to_cursor, normalize_sel, resize_direction, strip_markdown};
use super::message::Message;
use super::nova::Nova;

use iced::alignment::{Horizontal, Vertical};
use iced::mouse;
use iced::widget::{button, column, container, mouse_area, row, stack, text};
use iced::{Border, Color, Element, Length, Padding, Theme, border::Radius};

use crate::core::router::Mode;
use crate::ui::components;

pub const COMPACT_WIDTH: f32 = 640.0;
use crate::ui::theme;

impl Nova {
  pub fn view(&self) -> Element<'_, Message> {
    let active_tab = &self.tabs[self.active_index];
    let pop = self.window_pop.interpolate(0.0, 1.0, self.now);

    let selection = match (self.selection_start, self.selection_end) {
      (Some(start), Some(end)) if start != end => {
        let ((sc, sr), (ec, er)) = normalize_sel(start, end);
        Some((sc, sr, ec, er))
      }
      _ => None,
    };

    let font_size = self.settings.theme.font.size;
    let resize_cursor = resize_direction(self.cursor_position, self.window_size).map(dir_to_cursor);

    let term_interaction = if self.dragging_split {
      mouse::Interaction::ResizingColumn
    } else {
      resize_cursor.unwrap_or_else(|| {
        if self.hovered_url.is_some() {
          mouse::Interaction::Pointer
        } else {
          mouse::Interaction::Text
        }
      })
    };

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

      let ratio = active_tab.split_ratio;
      let left_portion = (ratio * 100.0).round() as u16;
      let right_portion = 100u16.saturating_sub(left_portion);

      let search_matches_ref = if self.search_active {
        self.search_matches.as_slice()
      } else {
        &[]
      };
      let search_current = if self.search_active {
        Some(self.search_match_index)
      } else {
        None
      };

      let cursor_style = self.settings.theme.cursor.style;
      let cursor_blink = self.cursor_blink_visible;

      let left = mouse_area(
        container(stack![
          self
            .pane_body(active_tab, false)
            .unwrap_or_else(|| components::term(
              &active_tab.grid,
              primary_selection,
              font_size,
              active_tab.scroll_offset,
              primary_url,
              primary_span,
              search_matches_ref,
              search_current,
              cursor_style,
              cursor_blink,
            )),
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
        .width(Length::FillPortion(left_portion))
        .height(Length::Fill),
      )
      .interaction(term_interaction);

      let vis_bar = container(iced::widget::Space::new())
        .width(1)
        .height(Length::Fill)
        .style(move |_| container::Style {
          background: Some(border_color.into()),
          ..Default::default()
        });

      let divider = mouse_area(
        container(vis_bar)
          .width(8)
          .height(Length::Fill)
          .align_x(Horizontal::Center),
      )
      .interaction(mouse::Interaction::ResizingColumn);

      let right = mouse_area(
        container(stack![
          self
            .pane_body(active_tab, true)
            .unwrap_or_else(|| components::term(
              &split.grid,
              None,
              font_size,
              split.scroll_offset,
              None,
              None,
              &[],
              None,
              cursor_style,
              cursor_blink,
            )),
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
        .width(Length::FillPortion(right_portion))
        .height(Length::Fill),
      )
      .interaction(term_interaction);

      let pane_row: Element<'_, Message> = row![left, divider, right].height(Length::Fill).into();
      if self.search_active {
        use iced::widget::stack;
        stack![
          pane_row,
          components::search_bar(
            &self.search_query,
            self.search_match_index,
            self.search_match_count(),
          ),
        ]
        .into()
      } else {
        pane_row
      }
    } else {
      let search_matches_ref = if self.search_active {
        self.search_matches.as_slice()
      } else {
        &[]
      };
      let search_current = if self.search_active {
        Some(self.search_match_index)
      } else {
        None
      };
      let cursor_style = self.settings.theme.cursor.style;
      let cursor_blink = self.cursor_blink_visible;
      let term_el: Element<'_, Message> = match self.pane_body(active_tab, false) {
        Some(body) => body,
        None => mouse_area(components::term(
          &active_tab.grid,
          selection,
          font_size,
          active_tab.scroll_offset,
          self.hovered_url.as_deref(),
          self.hovered_link_span,
          search_matches_ref,
          search_current,
          cursor_style,
          cursor_blink,
        ))
        .interaction(term_interaction)
        .into(),
      };
      let waiting = if active_tab.active_pane_is_split {
        active_tab
          .split
          .as_ref()
          .map(|s| s.waiting_after_exit)
          .unwrap_or(false)
      } else {
        active_tab.waiting_after_exit
      };

      if self.search_active || waiting {
        let mut layers: Vec<Element<'_, Message>> = vec![term_el];
        if self.search_active {
          layers.push(components::search_bar(
            &self.search_query,
            self.search_match_index,
            self.search_match_count(),
          ));
        }
        if waiting {
          layers.push(
            container(
              container(
                text("Process exited — press any key to close")
                  .size(12)
                  .color(theme::color::runtime().foreground),
              )
              .padding(Padding {
                top: 6.0,
                bottom: 4.0,
                left: 14.0,
                right: 14.0,
              })
              .style(|_| {
                let rt = theme::color::runtime();
                container::Style {
                  background: Some(rt.background.into()),
                  border: Border {
                    color: rt.border,
                    width: 1.0,
                    radius: Radius::new(4.0),
                  },
                  ..Default::default()
                }
              }),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Bottom)
            .padding(Padding {
              top: 0.0,
              bottom: 12.0,
              left: 0.0,
              right: 0.0,
            })
            .into(),
          );
        }
        stack(layers).into()
      } else {
        term_el
      }
    };

    let tb_interaction = resize_cursor.unwrap_or(mouse::Interaction::Idle);

    let mut col = column![
      components::title_bar(
        if self.tabs.len() > 1 {
          components::tab_bar(components::TabView {
            tabs: &self.tabs,
            active: self.active_index,
            hovered: self.hovered_tab,
            dragging: self.dragging_tab_index,
            now: self.now,
            window_width: self.window_size.width,
          })
        } else {
          components::title(&active_tab.pwd, !self.bell_blink_visible)
        },
        tb_interaction,
        &self.settings.general.window_controls,
      ),
      term_area,
    ];

    if let Some((_code, ref message, ref command)) = self.diagnostic_banner {
      col = col.push(self.diagnostic_banner_widget(message, command.as_deref()));
    }

    let outer_interaction = if self.dragging_split {
      mouse::Interaction::ResizingColumn
    } else {
      resize_cursor.unwrap_or(mouse::Interaction::Idle)
    };

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
      components::app(stack![col, modal], self.window_maximized, pop)
    } else if self.command_palette_open {
      let palette = components::command_palette(&self.palette_query, self.palette_selected);
      components::app(stack![col, palette], self.window_maximized, pop)
    } else if self.ai_overlay_open || self.ai_loading {
      let overlay = components::ai_overlay(
        &self.ai_input,
        self.ai_response.as_deref(),
        self.ai_loading,
        self.ai_is_error,
      );
      components::app(stack![col, overlay], self.window_maximized, pop)
    } else if self.shell_picker_open {
      let picker = components::shell_picker(
        &self.available_shells,
        self.shell_picker_anchor,
        self.window_size.width,
      );
      components::app(stack![col, picker], self.window_maximized, pop)
    } else if let Some(ref pending) = self.paste_warning {
      let overlay = components::paste_warning_overlay(pending);
      components::app(stack![col, overlay], self.window_maximized, pop)
    } else {
      components::app(col, self.window_maximized, pop)
    };

    mouse_area(inner).interaction(outer_interaction).into()
  }

  fn pane_body<'a>(
    &'a self,
    tab: &'a crate::ui::tab::Tab,
    split: bool,
  ) -> Option<Element<'a, Message>> {
    let pane = tab.pane(split);
    if pane.integration.mode == Mode::Classic {
      return None;
    }
    let font_size = self.settings.theme.font.size;
    if let Some(grid) = pane.raw_grid() {
      return Some(
        container(components::term(
          grid,
          None,
          crate::ui::theme::tokens::block::font_size(font_size),
          0,
          None,
          None,
          &[],
          None,
          self.settings.theme.cursor.style,
          self.cursor_blink_visible,
        ))
        .padding(crate::ui::components::session::scroll_padding(
          self.window_size.width <= COMPACT_WIDTH,
        ))
        .into(),
      );
    }
    let compact = self.window_size.width <= COMPACT_WIDTH;
    let view = components::SessionView {
      pane_id: pane.id,
      session: pane.session,
      font_size,
      timestamps: false,
      now: self.now,
      shown_at: pane.integration.spawned_at,
      compact,
      selection: self.block_selection,
      hover_url: self.block_url.as_ref(),
      matches: if self.search_active && tab.active_pane().id == pane.id {
        &self.block_matches
      } else {
        &[]
      },
      current_match: (self.search_active && !self.block_matches.is_empty())
        .then_some(self.search_match_index),
    };
    Some(
      column![
        components::blocks_area(&view),
        components::input_bar(components::InputBarView {
          pane_id: pane.id,
          cwd: &pane.session.cwd,
          value: &pane.input.text,
          running: pane.session.is_running(),
          pulsed_at: pane.input.pulsed_at,
          now: self.now,
          font_size,
          compact,
        }),
      ]
      .into(),
    )
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
        .font(theme::font::bold())
        .size(12)
        .color(accent),
    );
    inner = inner.push(
      text(format!(" {}", strip_markdown(message)))
        .font(theme::font::regular())
        .size(12)
        .color(fg),
    );
    if let Some(cmd) = command {
      let cmd_text = cmd.to_string();
      inner = inner.push(
        button(
          text(format!(" {} ", cmd_text))
            .font(theme::font::regular())
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
