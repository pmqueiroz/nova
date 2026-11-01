use crate::core::grid::Grid;
use crate::core::search::{relative_position, search_session};

use super::super::nova::Nova;

pub(super) fn compute_search_matches(grid: &Grid, query: &str) -> Vec<(bool, usize, usize, usize)> {
  let query_lower: Vec<char> = query.chars().map(|c| c.to_ascii_lowercase()).collect();
  let qlen = query_lower.len();
  if qlen == 0 {
    return vec![];
  }

  let mut matches = Vec::new();

  let mut search_row = |row_cells: &[crate::core::grid::Cell], is_sb: bool, row_idx: usize| {
    let n = row_cells.len();
    if n < qlen {
      return;
    }
    'outer: for start in 0..=(n - qlen) {
      for i in 0..qlen {
        let gc = row_cells[start + i]
          .c
          .chars()
          .next()
          .unwrap_or('\0')
          .to_ascii_lowercase();
        if gc != query_lower[i] {
          continue 'outer;
        }
      }
      matches.push((is_sb, row_idx, start, start + qlen));
    }
  };

  for (row_idx, (row_cells, _)) in grid.scrollback.iter().enumerate() {
    search_row(row_cells, true, row_idx);
  }
  for y in 0..grid.rows {
    search_row(grid.row(y), false, y);
  }

  matches
}

impl Nova {
  pub(super) fn recompute_search(&mut self) {
    let Some(tab) = self.tabs.get(self.active_index) else {
      self.search_matches.clear();
      self.block_matches.clear();
      self.search_match_index = 0;
      return;
    };
    let pane = tab.active_pane();
    if pane.integration.mode != crate::core::router::Mode::Classic {
      self.block_matches = search_session(pane.session, &self.search_query);
      self.search_matches.clear();
    } else {
      self.block_matches.clear();
      self.search_matches = compute_search_matches(&tab.grid, &self.search_query);
    }
    self.search_match_index = 0;
    self.scroll_to_search_match();
  }

  pub(in crate::ui::app_state) fn search_match_count(&self) -> usize {
    if self.block_matches.is_empty() {
      self.search_matches.len()
    } else {
      self.block_matches.len()
    }
  }

  pub(super) fn scroll_to_block_match(&self) -> iced::Task<super::super::message::Message> {
    let Some(m) = self.block_matches.get(self.search_match_index) else {
      return iced::Task::none();
    };
    let Some(tab) = self.tabs.get(self.active_index) else {
      return iced::Task::none();
    };
    let pane = tab.active_pane();
    let y = relative_position(pane.session, m);
    iced::widget::operation::snap_to(
      crate::ui::components::session::scroll_id(pane.id),
      iced::widget::scrollable::RelativeOffset { x: 0.0, y },
    )
  }

  pub(super) fn scroll_to_search_match(&mut self) {
    let Some(&(is_scrollback, row, _, _)) = self.search_matches.get(self.search_match_index) else {
      return;
    };
    let Some(tab) = self.tabs.get_mut(self.active_index) else {
      return;
    };
    if is_scrollback {
      let sb_len = tab.grid.scrollback.len();
      tab.scroll_offset = sb_len.saturating_sub(row);
    } else {
      tab.scroll_offset = 0;
    }
  }
}
