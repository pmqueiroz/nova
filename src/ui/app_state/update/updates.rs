use std::path::Path;

use crate::core::install::{self, UpdatePlan};
use crate::core::update::{Offer, Status};
use crate::sys::update::{self as sys_update, Event, apply};
use crate::ui::tab::Tab;

use super::super::helpers::calc_grid;
use super::super::message::Message;
use super::super::nova::Nova;

const UPDATE_TAB_TITLE: &str = "update";

impl Nova {
  pub(super) fn check_for_updates(&mut self) -> iced::Task<Message> {
    if self.update_status.is_busy()
      || matches!(self.update_status, Status::Ready(_) | Status::Delegated(_))
    {
      return iced::Task::none();
    }
    self.update_status = Status::Checking;
    iced::Task::perform(sys_update::check(), Message::UpdateChecked)
  }

  pub(super) fn handle_update_checked(
    &mut self,
    result: Result<Option<Offer>, String>,
  ) -> iced::Task<Message> {
    self.update_status = crate::core::update::status_from(result);
    iced::Task::none()
  }

  pub(super) fn start_update(&mut self) -> iced::Task<Message> {
    let Status::Available(offer) = &self.update_status else {
      return iced::Task::none();
    };
    let offer = offer.clone();
    match &offer.plan {
      UpdatePlan::OpenPage(url) => {
        let _ = open::that_detached(url);
        iced::Task::none()
      }
      UpdatePlan::Delegate {
        argv,
        download: None,
      } => {
        self.open_update_tab(argv.clone());
        self.update_status = Status::Delegated(offer);
        iced::Task::none()
      }
      UpdatePlan::AfterExit { .. } => {
        self.update_status = Status::Ready(offer);
        iced::Task::none()
      }
      _ => {
        self.update_status = Status::Downloading(offer.clone(), 0.0);
        iced::Task::run(sys_update::prepare(offer), Message::UpdateProgress)
      }
    }
  }

  pub(super) fn handle_update_progress(&mut self, event: Event) -> iced::Task<Message> {
    let Status::Downloading(offer, progress) = &mut self.update_status else {
      return iced::Task::none();
    };
    match event {
      Event::Progress(p) => *progress = p,
      Event::Done(Ok(file)) => {
        let offer = offer.clone();
        self.update_status = match &offer.plan {
          UpdatePlan::Delegate { argv, .. } => {
            self.open_update_tab(install::fill_file(argv, Path::new(&file)));
            Status::Delegated(offer)
          }
          _ => Status::Ready(offer),
        };
      }
      Event::Done(Err(e)) => self.update_status = Status::Failed(e),
    }
    iced::Task::none()
  }

  pub(super) fn restart_for_update(&mut self) -> iced::Task<Message> {
    let Status::Ready(offer) = &self.update_status else {
      return iced::Task::none();
    };
    match apply::restart(offer) {
      Ok(()) => std::process::exit(0),
      Err(e) => {
        self.update_status = Status::Failed(e);
        iced::Task::none()
      }
    }
  }

  fn open_update_tab(&mut self, argv: Vec<String>) {
    let id = self.next_tab_id;
    self.next_tab_id += 1;
    let (cols, rows) = calc_grid(
      self.window_size.width,
      self.window_size.height,
      self.settings.theme.font.size,
      self.diagnostic_banner.is_some(),
    );
    let cwd = dirs::home_dir()
      .map(|h| h.display().to_string())
      .unwrap_or_default();
    self.tabs.push(Tab::for_command(
      id,
      cols,
      rows,
      UPDATE_TAB_TITLE,
      argv,
      cwd,
    ));
    self.active_index = self.tabs.len() - 1;
  }
}
