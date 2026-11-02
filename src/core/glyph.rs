#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect8 {
  pub x0: u8,
  pub y0: u8,
  pub x1: u8,
  pub y1: u8,
}

const fn r(x0: u8, y0: u8, x1: u8, y1: u8) -> Rect8 {
  Rect8 { x0, y0, x1, y1 }
}

const FULL: Rect8 = r(0, 0, 8, 8);
const UL: Rect8 = r(0, 0, 4, 4);
const UR: Rect8 = r(4, 0, 8, 4);
const LL: Rect8 = r(0, 4, 4, 8);
const LR: Rect8 = r(4, 4, 8, 8);

const NONE: Rect8 = r(0, 0, 0, 0);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockGlyph {
  rects: [Rect8; 3],
  len: u8,
  pub alpha: f32,
}

impl BlockGlyph {
  pub fn rects(&self) -> &[Rect8] {
    &self.rects[..self.len as usize]
  }
}

fn solid(rects: &[Rect8]) -> BlockGlyph {
  let mut out = [NONE; 3];
  out[..rects.len()].copy_from_slice(rects);
  BlockGlyph {
    rects: out,
    len: rects.len() as u8,
    alpha: 1.0,
  }
}

fn shade(alpha: f32) -> BlockGlyph {
  BlockGlyph {
    alpha,
    ..solid(&[FULL])
  }
}

pub fn is_block_glyph(c: char) -> bool {
  matches!(c as u32, 0x2580..=0x259F)
}

pub fn block_glyph(c: char) -> Option<BlockGlyph> {
  if !is_block_glyph(c) {
    return None;
  }
  let code = c as u32;
  Some(match code {
    0x2580 => solid(&[r(0, 0, 8, 4)]),
    0x2581..=0x2587 => {
      let n = (code - 0x2580) as u8;
      solid(&[r(0, 8 - n, 8, 8)])
    }
    0x2588 => solid(&[FULL]),
    0x2589..=0x258F => {
      let n = (0x2590 - code) as u8;
      solid(&[r(0, 0, n, 8)])
    }
    0x2590 => solid(&[r(4, 0, 8, 8)]),
    0x2591 => shade(0.25),
    0x2592 => shade(0.5),
    0x2593 => shade(0.75),
    0x2594 => solid(&[r(0, 0, 8, 1)]),
    0x2595 => solid(&[r(7, 0, 8, 8)]),
    0x2596 => solid(&[LL]),
    0x2597 => solid(&[LR]),
    0x2598 => solid(&[UL]),
    0x2599 => solid(&[UL, LL, LR]),
    0x259A => solid(&[UL, LR]),
    0x259B => solid(&[UL, UR, LL]),
    0x259C => solid(&[UL, UR, LR]),
    0x259D => solid(&[UR]),
    0x259E => solid(&[UR, LL]),
    0x259F => solid(&[UR, LL, LR]),
    _ => return None,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn eighth_blocks_grow_from_their_edge() {
    assert_eq!(block_glyph('▁').unwrap().rects(), [r(0, 7, 8, 8)]);
    assert_eq!(block_glyph('▇').unwrap().rects(), [r(0, 1, 8, 8)]);
    assert_eq!(block_glyph('▉').unwrap().rects(), [r(0, 0, 7, 8)]);
    assert_eq!(block_glyph('▏').unwrap().rects(), [r(0, 0, 1, 8)]);
  }

  #[test]
  fn halves_quadrants_and_shades() {
    assert_eq!(block_glyph('▀').unwrap().rects(), [r(0, 0, 8, 4)]);
    assert_eq!(block_glyph('▐').unwrap().rects(), [r(4, 0, 8, 8)]);
    assert_eq!(block_glyph('▟').unwrap().rects(), [UR, LL, LR]);
    assert_eq!(block_glyph('▒').unwrap().alpha, 0.5);
    assert_eq!(block_glyph('a'), None);
    assert_eq!(block_glyph('─'), None);
  }
}
