use iced::Element;

use crate::core::install::UpdatePlan;
use crate::core::update::Status;
use crate::ui::app_state::Message;
use crate::ui::components::confirm::{ConfirmView, confirm_sheet};
use crate::ui::components::ds::button::{Variant, btn};

pub fn update_action(status: &Status) -> Option<(String, Option<Message>)> {
  match status {
    Status::Available(o) => Some(match o.plan {
      UpdatePlan::OpenPage(_) => (
        format!("Get {}", o.release.version),
        Some(Message::StartUpdate),
      ),
      _ => (
        format!("Update to {}", o.release.version),
        Some(Message::StartUpdate),
      ),
    }),
    Status::Downloading(_, p) => Some((format!("Updating {:.0}%", p * 100.0), None)),
    Status::Ready(_) => Some(("Restart to update".into(), Some(Message::ConfirmRestart))),
    _ => None,
  }
}

pub fn title_update_button<'a>(status: &Status) -> Option<Element<'a, Message>> {
  let (_, msg) = update_action(status)?;
  let b = btn::<Message>(Variant::Orange).label("Update");
  Some(
    match msg {
      Some(m) => b.on_press(m),
      None => b,
    }
    .into_element(),
  )
}

pub fn update_line(status: &Status) -> Option<String> {
  let via = |o: &crate::core::update::Offer| match o.plan.command_line() {
    Some(cmd) => format!("Installed via {} · updates with `{cmd}`", o.install),
    None => format!("Installed via {}", o.install),
  };
  match status {
    Status::Unknown => None,
    Status::Checking => Some("Checking for updates\u{2026}".into()),
    Status::UpToDate => Some("You're on the latest version".into()),
    Status::Available(o) => Some(format!(
      "Version {} is available. {}",
      o.release.version,
      via(o)
    )),
    Status::Downloading(o, _) => Some(format!("Downloading {}\u{2026}", o.release.version)),
    Status::Ready(o) => Some(format!(
      "Version {} is ready. It applies when Nova restarts.",
      o.release.version
    )),
    Status::Delegated(o) => Some(format!(
      "Updating to {} in a new tab. Restart Nova when it finishes.",
      o.release.version
    )),
    Status::Failed(e) => Some(format!("Update failed: {e}")),
  }
}

pub fn update_button<'a>(status: &Status) -> Element<'a, Message> {
  match update_action(status) {
    Some((label, msg)) => {
      let b = btn::<Message>(Variant::Primary).label(label);
      match msg {
        Some(m) => b.on_press(m),
        None => b,
      }
      .into_element()
    }
    None => {
      let b = btn::<Message>(Variant::Default).label("Check for updates");
      if status.is_busy() {
        b.into_element()
      } else {
        b.on_press(Message::CheckForUpdates).into_element()
      }
    }
  }
}

pub fn restart_confirm<'a>(progress: f32, window_height: f32) -> Element<'a, Message> {
  confirm_sheet(
    ConfirmView {
      title: "Restart Nova to update?".into(),
      reason: "Open tabs and running commands will be closed.".into(),
      detail: None,
      confirm_label: "Restart",
    },
    progress,
    window_height,
  )
}
