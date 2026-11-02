use super::helpers::{dir_to_cursor, normalize_sel, resize_direction};
use super::message::Message;
use super::nova::{Confirm, Nova};

use iced::alignment::{Horizontal, Vertical};
use iced::mouse;
use iced::widget::{column, container, mouse_area, row, stack, text};
use iced::{Border, Element, Length, Padding, Theme, border::Radius};

use crate::core::router::Mode;
use crate::ui::components;

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
      let border_color = crate::ui::theme::tokens::LINE;

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
        container(crate::ui::components::ds::button::icon_button(
          crate::ui::icons::Icon::Close,
          crate::ui::components::ds::button::IconSize::Sm,
          Some(msg),
        ))
        .align_x(Horizontal::Right)
        .align_y(Vertical::Top)
        .padding(crate::ui::theme::tokens::space::S8)
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
              components::CLASSIC_PADDING,
            )),
          make_close_btn(Message::CloseLeftPane),
        ])
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
              components::CLASSIC_PADDING,
            )),
          make_close_btn(Message::CloseSplitPane),
        ])
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
          components::CLASSIC_PADDING,
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
          components::title(
            &active_tab.pwd,
            !self.bell_blink_visible,
            crate::ui::theme::tokens::block::is_compact(self.window_size.width),
          )
        },
        tb_interaction,
        &self.settings.general.window_controls,
        components::title_update_button(&self.update_status),
      ),
      term_area,
    ];

    if let Some((_code, ref message, ref command)) = self.diagnostic_banner
      && active_tab.active_pane().integration.mode == Mode::Classic
    {
      col = col.push(
        container(components::inline_suggestion(message, command.as_deref()))
          .padding(Padding::from([0.0, crate::ui::theme::tokens::space::S16])),
      );
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
      let modal = components::settings_modal(components::SettingsView {
        settings: &self.settings,
        tab: &self.settings_tab,
        shell_input: &self.settings_shell_input,
        recording: self.settings_recording_index,
        raw_content: &self.raw_config_content,
        config_path: config_path_str,
        timestamps: self.timestamps,
        progress: self.sheet_progress(),
        window_height: self.window_size.height,
        update: &self.update_status,
      });
      components::app(stack![col, modal], self.window_maximized, pop)
    } else if self.command_palette_open {
      let palette = components::command_palette(
        &self.palette_query,
        self.palette_selected,
        self.sheet_progress(),
        self.window_size.height,
      );
      components::app(stack![col, palette], self.window_maximized, pop)
    } else if self.ai_overlay_open {
      let overlay = components::ai_sheet(components::AiSheetView {
        query: &self.ai_input,
        items: self.ai_items(),
        selected: self.ai_selected,
        answer: self.ai_answer.as_ref(),
        now: self.now,
        progress: self.sheet_progress(),
        window_height: self.window_size.height,
      });
      components::app(stack![col, overlay], self.window_maximized, pop)
    } else if self.shell_picker_open {
      let picker = components::shell_picker(
        &self.available_shells,
        self.shell_picker_selected,
        self.sheet_progress(),
        self.window_size.height,
      );
      components::app(stack![col, picker], self.window_maximized, pop)
    } else if let Some(ref confirm) = self.confirm {
      let overlay = match confirm {
        Confirm::Paste(pending) => {
          components::paste_warning_overlay(pending, self.sheet_progress(), self.window_size.height)
        }
        Confirm::Restart => {
          components::restart_confirm(self.sheet_progress(), self.window_size.height)
        }
      };
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
      return Some(components::term(
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
        crate::ui::components::session::raw_padding(crate::ui::theme::tokens::block::is_compact(
          self.window_size.width,
        )),
      ));
    }
    let compact = crate::ui::theme::tokens::block::is_compact(self.window_size.width);
    let view = components::SessionView {
      pane_id: pane.id,
      session: pane.session,
      font_size,
      timestamps: self.timestamps,
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
      suggestion: (tab.active_pane().id == pane.id)
        .then_some(self.diagnostic_banner.as_ref())
        .flatten()
        .map(|(_, m, c)| (m.as_str(), c.as_deref())),
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
}
