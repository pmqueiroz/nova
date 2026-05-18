pub fn run() -> i32 {
  eprint!("{}", render());
  0
}

fn render() -> String {
  const ROWS: usize = 21;
  const COLS: usize = 41;
  const CY: f64 = 10.0;
  const CX: f64 = 20.0;
  const RY: f64 = 9.0;
  const RX: f64 = 18.0;

  let mut grid = vec![vec![' '; COLS]; ROWS];

  let in_ellipse = |r: usize, c: usize| -> bool {
    let dy = (r as f64 - CY) / RY;
    let dx = (c as f64 - CX) / RX;
    dy * dy + dx * dx <= 1.0 + 1e-9
  };

  for r in 0..ROWS {
    let dy = r as f64 - CY;
    if dy.abs() > RY {
      continue;
    }
    let dx = RX * (1.0 - (dy / RY).powi(2)).sqrt();
    let left = (CX - dx).round() as usize;
    let right = (CX + dx).round() as usize;
    if left < COLS {
      grid[r][left] = '·';
    }
    if right < COLS {
      grid[r][right] = '·';
    }
  }

  for r in 0..ROWS {
    if in_ellipse(r, 20) {
      grid[r][20] = '│';
    }
  }

  for c in 0..COLS {
    if in_ellipse(10, c) {
      grid[10][c] = '─';
    }
  }

  for r in 0..ROWS {
    let cf = CX + (RX / RY) * (r as f64 - CY);
    let c = cf.round() as isize;
    if c >= 0 && c < COLS as isize && in_ellipse(r, c as usize) {
      grid[r][c as usize] = '\\';
    }
  }

  for r in 0..ROWS {
    let cf = CX - (RX / RY) * (r as f64 - CY);
    let c = cf.round() as isize;
    if c >= 0 && c < COLS as isize && in_ellipse(r, c as usize) {
      grid[r][c as usize] = '/';
    }
  }

  grid[10][20] = '┼';

  let mut out = String::with_capacity(ROWS * (COLS + 1));
  for row in &grid {
    let line: String = row.iter().collect();
    out.push_str(line.trim_end());
    out.push('\n');
  }
  out
}
