use std::time::{Duration, Instant};

use iced::animation::{Animation, Easing, Float};

use super::tokens::duration;

fn bezier_axis(t: f32, p1: f32, p2: f32) -> f32 {
  let u = 1.0 - t;
  3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t
}

fn bezier_axis_slope(t: f32, p1: f32, p2: f32) -> f32 {
  let u = 1.0 - t;
  3.0 * u * u * p1 + 6.0 * u * t * (p2 - p1) + 3.0 * t * t * (1.0 - p2)
}

fn cubic_bezier(x: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
  if x <= 0.0 {
    return 0.0;
  }
  if x >= 1.0 {
    return 1.0;
  }
  let mut t = x;
  for _ in 0..8 {
    let err = bezier_axis(t, x1, x2) - x;
    if err.abs() < 1e-5 {
      return bezier_axis(t, y1, y2);
    }
    let slope = bezier_axis_slope(t, x1, x2);
    if slope.abs() < 1e-6 {
      break;
    }
    t -= err / slope;
  }
  let (mut lo, mut hi) = (0.0_f32, 1.0_f32);
  t = x;
  for _ in 0..32 {
    let v = bezier_axis(t, x1, x2);
    if (v - x).abs() < 1e-5 {
      break;
    }
    if v < x {
      lo = t;
    } else {
      hi = t;
    }
    t = (lo + hi) / 2.0;
  }
  bezier_axis(t, y1, y2)
}

pub fn ease(x: f32) -> f32 {
  cubic_bezier(x, 0.2, 0.8, 0.2, 1.0)
}

pub fn spring(x: f32) -> f32 {
  cubic_bezier(x, 0.34, 1.56, 0.64, 1.0)
}

pub const EASE: Easing = Easing::Custom(ease);
pub const SPRING: Easing = Easing::Custom(spring);

pub fn anim<T: Clone + Copy + PartialEq + Float>(
  state: T,
  dur: Duration,
  easing: Easing,
) -> Animation<T> {
  Animation::new(state).duration(dur).easing(easing)
}

pub fn fast<T: Clone + Copy + PartialEq + Float>(state: T) -> Animation<T> {
  anim(state, duration::D1, EASE)
}

pub fn base<T: Clone + Copy + PartialEq + Float>(state: T) -> Animation<T> {
  anim(state, duration::D2, EASE)
}

pub fn slow<T: Clone + Copy + PartialEq + Float>(state: T) -> Animation<T> {
  anim(state, duration::D4, EASE)
}

pub fn entrance(now: Instant) -> Animation<bool> {
  base(false).go(true, now)
}

pub fn progress(a: &Animation<bool>, now: Instant) -> f32 {
  a.interpolate(0.0, 1.0, now)
}

pub fn lerp(from: f32, to: f32, t: f32) -> f32 {
  from + (to - from) * t
}

pub fn blink_on(started: Instant, now: Instant) -> bool {
  (now.duration_since(started).as_millis() / 500).is_multiple_of(2)
}

pub fn dot_phase(started: Instant, now: Instant, delay_ms: u64) -> (f32, f32) {
  let period = 1000.0;
  let elapsed = now.duration_since(started).as_millis() as f32 - delay_ms as f32;
  if elapsed < 0.0 {
    return (0.24, 0.0);
  }
  let p = (elapsed % period) / period;
  let k = if p < 0.4 {
    ease(p / 0.4)
  } else if p < 0.8 {
    1.0 - ease((p - 0.4) / 0.4)
  } else {
    0.0
  };
  (lerp(0.24, 1.0, k), lerp(0.0, -2.0, k))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn curves_hit_endpoints() {
    for f in [ease, spring] {
      assert_eq!(f(0.0), 0.0);
      assert_eq!(f(1.0), 1.0);
    }
  }

  #[test]
  fn ease_is_monotonic() {
    let mut prev = 0.0;
    for i in 1..=100 {
      let v = ease(i as f32 / 100.0);
      assert!(v >= prev - 1e-4);
      prev = v;
    }
  }

  #[test]
  fn spring_overshoots() {
    let peak = (1..100)
      .map(|i| spring(i as f32 / 100.0))
      .fold(0.0, f32::max);
    assert!(peak > 1.0);
  }

  #[test]
  fn ease_front_loaded() {
    assert!(ease(0.25) > 0.6);
  }

  #[test]
  fn dot_phase_ranges() {
    let t0 = Instant::now();
    for ms in (0..2000).step_by(37) {
      let (o, y) = dot_phase(t0, t0 + Duration::from_millis(ms), 120);
      assert!((0.24..=1.0).contains(&o));
      assert!((-2.0..=0.0).contains(&y));
    }
  }
}
