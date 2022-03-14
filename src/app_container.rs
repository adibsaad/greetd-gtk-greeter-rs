use gtk::{prelude::*, Align, CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};
use relm::Widget;
use relm_derive::{widget, Msg};

use crate::main_window::MainWin;
use crate::prompt::Prompt;

pub struct Model {}

#[derive(Msg)]
pub enum Msg {}

#[widget]
impl Widget for AppContainer {
  fn model() -> Model {
    Model {}
  }
  fn update(&mut self, _: Msg) {}
  fn init_view(&mut self) {
    let style_context = self.widgets.main_win.style_context();
    let style = include_bytes!("./css/app.css");
    let provider = CssProvider::new();
    provider.load_from_data(style).unwrap();
    style_context.add_provider(&provider, STYLE_PROVIDER_PRIORITY_APPLICATION);
  }
  view! {
    #[name="main_win"]
    MainWin {
      widget_name: "main_win",
      gtk::Box {
        valign: Align::Center,
        halign: Align::Center,
        Prompt {},
      },
    }
  }
}

pub fn start_app() {
  AppContainer::run(()).unwrap();
}
