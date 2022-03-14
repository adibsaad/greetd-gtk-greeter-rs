#![feature(box_syntax)]

mod app_container;
mod login_manager;
mod main_window;
mod prompt;

fn main() {
  app_container::start_app();
}
