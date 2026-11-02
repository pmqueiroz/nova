use std::sync::atomic::{AtomicBool, Ordering};

const BELL_FLAC: &[u8] = include_bytes!("../../assets/sounds/bell.flac");

static PLAYING: AtomicBool = AtomicBool::new(false);

pub fn play() {
  if PLAYING.swap(true, Ordering::AcqRel) {
    return;
  }
  std::thread::spawn(|| {
    if let Ok((_stream, handle)) = rodio::OutputStream::try_default()
      && let Ok(source) = rodio::Decoder::new(std::io::Cursor::new(BELL_FLAC))
      && let Ok(sink) = rodio::Sink::try_new(&handle)
    {
      sink.append(source);
      sink.sleep_until_end();
    }
    PLAYING.store(false, Ordering::Release);
  });
}
