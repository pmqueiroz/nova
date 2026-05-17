use std::sync::OnceLock;

const APP_ID: &str = "com.pmqueiroz.nova";

static CLICK_CHANNEL: OnceLock<(async_channel::Sender<()>, async_channel::Receiver<()>)> =
  OnceLock::new();

fn click_channel() -> &'static (async_channel::Sender<()>, async_channel::Receiver<()>) {
  CLICK_CHANNEL.get_or_init(async_channel::unbounded)
}

pub fn click_receiver() -> async_channel::Receiver<()> {
  click_channel().1.clone()
}

#[cfg(target_os = "windows")]
pub fn register() {
  use winreg::RegKey;
  use winreg::enums::{HKEY_CURRENT_USER, KEY_WRITE};

  let path = format!("Software\\Classes\\AppUserModelIds\\{}", APP_ID);
  if let Ok(key) = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(&path, KEY_WRITE) {
    let _ = key.set_value("DisplayName", &"Nova");
    return;
  }
  if let Ok((key, _)) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(&path) {
    let _ = key.set_value("DisplayName", &"Nova");
  }
}

#[cfg(not(target_os = "windows"))]
pub fn register() {}

pub fn send(title: &str, body: &str) {
  let title = title.to_string();
  let body = body.to_string();
  let tx = click_channel().0.clone();

  #[cfg(target_os = "windows")]
  std::thread::spawn(move || {
    let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
    let done_tx_dismiss = done_tx.clone();

    let _ = tauri_winrt_notification::Toast::new(APP_ID)
      .title(&title)
      .text1(&body)
      .on_activated(move |_| {
        let _ = tx.send_blocking(());
        let _ = done_tx.send(());
        Ok(())
      })
      .on_dismissed(move |_| {
        let _ = done_tx_dismiss.send(());
        Ok(())
      })
      .show();

    let _ = done_rx.recv();
  });

  #[cfg(not(target_os = "windows"))]
  std::thread::spawn(move || {
    if let Ok(handle) = notify_rust::Notification::new()
      .summary(&title)
      .body(&body)
      .show()
    {
      handle.wait_for_action(|_| {
        let _ = tx.send_blocking(());
      });
    }
  });
}
