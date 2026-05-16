pub fn send(title: &str, body: &str) {
  let title = title.to_string();
  let body = body.to_string();
  std::thread::spawn(move || {
    #[cfg(target_os = "windows")]
    {
      let script = "Add-Type -AssemblyName System.Windows.Forms; \
        $i = New-Object System.Windows.Forms.NotifyIcon; \
        $i.Icon = [System.Drawing.SystemIcons]::Application; \
        $i.Visible = $true; \
        $i.ShowBalloonTip(5000, $env:NOVA_NOTIF_TITLE, $env:NOVA_NOTIF_BODY, \
          [System.Windows.Forms.ToolTipIcon]::None); \
        Start-Sleep 6; \
        $i.Visible = $false";
      let _ = std::process::Command::new("powershell")
        .env("NOVA_NOTIF_TITLE", &title)
        .env("NOVA_NOTIF_BODY", &body)
        .args([
          "-NoProfile",
          "-NonInteractive",
          "-WindowStyle",
          "Hidden",
          "-Command",
          script,
        ])
        .spawn();
    }
    #[cfg(target_os = "macos")]
    {
      let _ = std::process::Command::new("osascript")
        .args([
          "-e",
          &format!("display notification {:?} with title {:?}", body, title),
        ])
        .spawn();
    }
    #[cfg(target_os = "linux")]
    {
      let _ = std::process::Command::new("notify-send")
        .args([&title, &body])
        .spawn();
    }
  });
}
