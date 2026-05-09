use crate::core::grid::Cell;

fn is_url_terminator(c: char) -> bool {
  c.is_whitespace() || matches!(c, '"' | '\'' | '<' | '>' | ')' | ']')
}

pub fn url_continuation_len(cells: &[Cell]) -> usize {
  let mut end = 0;
  while end < cells.len() && !is_url_terminator(cells[end].c) {
    end += 1;
  }
  end
}

pub fn detect_urls(cells: &[Cell]) -> Vec<(usize, usize, String)> {
  let mut results = Vec::new();
  let n = cells.len();
  let mut i = 0;

  while i < n {
    if let Some(uri) = &cells[i].uri {
      let start = i;
      let ptr = std::sync::Arc::as_ptr(uri);
      while i < n && cells[i].uri.as_ref().map(std::sync::Arc::as_ptr) == Some(ptr) {
        i += 1;
      }
      results.push((start, i - 1, uri.to_string()));
      continue;
    }

    let prefix_len = if cells[i].c == 'h' {
      if i + 8 <= n && cells[i..i + 8].iter().map(|c| c.c).eq("https://".chars()) {
        8
      } else if i + 7 <= n && cells[i..i + 7].iter().map(|c| c.c).eq("http://".chars()) {
        7
      } else {
        0
      }
    } else {
      0
    };

    if prefix_len == 0 {
      i += 1;
      continue;
    }

    let mut end = i + prefix_len;
    while end < n && !is_url_terminator(cells[end].c) {
      end += 1;
    }

    if end > i + prefix_len {
      let url: String = cells[i..end].iter().map(|c| c.c).collect();
      results.push((i, end - 1, url));
      i = end;
    } else {
      i += 1;
    }
  }

  results
}
