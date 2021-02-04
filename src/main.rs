#![feature(box_syntax)]
extern crate gtk_layer_shell_rs as gtk_layer_shell;

mod app_container;
mod login_manager;
mod main_window;
mod prompt;

fn main() {
  app_container::start_app();
}
