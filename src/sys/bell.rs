// Notification Sound by thehorriblejoke -- https://freesound.org/s/583554/ -- License: Creative Commons 0
const BELL_FLAC: &[u8] = include_bytes!("../../assets/sounds/bell.flac");

pub fn play() {
  std::thread::spawn(|| {
    if let Ok((_stream, handle)) = rodio::OutputStream::try_default()
      && let Ok(source) = rodio::Decoder::new(std::io::Cursor::new(BELL_FLAC))
      && let Ok(sink) = rodio::Sink::try_new(&handle)
    {
      sink.append(source);
      sink.sleep_until_end();
    }
  });
}
