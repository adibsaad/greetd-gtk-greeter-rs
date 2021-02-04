use gtk::{prelude::*, Window, WindowType};
use relm::{connect, Container, Relm, Update, Widget};
use relm_derive::Msg;

#[derive(Msg)]
pub enum Msg {
  Quit,
}

pub struct Model {}

#[allow(dead_code)]
pub struct MainWin {
  model: Model,
  window: Window,
}

impl Update for MainWin {
  type Model = Model;
  type ModelParam = ();
  type Msg = Msg;

  fn model(_: &Relm<Self>, _: ()) -> Model {
    Model {}
  }

  fn update(&mut self, event: Msg) {
    match event {
      Msg::Quit => gtk::main_quit(),
    }
  }
}

impl Widget for MainWin {
  type Root = Window;

  fn root(&self) -> Self::Root {
    self.window.clone()
  }

  fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
    let window = gtk::Window::new(WindowType::Toplevel);
    init_layer_shell(&window);
    connect!(
      relm,
      window,
      connect_delete_event(_, _),
      return (Some(Msg::Quit), Inhibit(false))
    );

    window.show_all();
    MainWin { model, window }
  }
}

impl Container for MainWin {
  type Container = gtk::Window;
  type Containers = ();

  fn container(&self) -> &Self::Container {
    &self.window
  }

  fn other_containers(&self) -> () {}
}

fn init_layer_shell(window: &gtk::Window) {
  gtk_layer_shell::init_for_window(window);
  gtk_layer_shell::set_keyboard_interactivity(window, true);
  gtk_layer_shell::auto_exclusive_zone_enable(window);
  gtk_layer_shell::set_layer(window, gtk_layer_shell::Layer::Top);
  gtk_layer_shell::auto_exclusive_zone_enable(window);

  gtk_layer_shell::set_margin(window, gtk_layer_shell::Edge::Top, 0);
  gtk_layer_shell::set_margin(window, gtk_layer_shell::Edge::Bottom, 0);
  gtk_layer_shell::set_margin(window, gtk_layer_shell::Edge::Left, 0);
  gtk_layer_shell::set_margin(window, gtk_layer_shell::Edge::Right, 0);

  gtk_layer_shell::set_anchor(window, gtk_layer_shell::Edge::Top, true);
  gtk_layer_shell::set_anchor(window, gtk_layer_shell::Edge::Bottom, true);
  gtk_layer_shell::set_anchor(window, gtk_layer_shell::Edge::Left, true);
  gtk_layer_shell::set_anchor(window, gtk_layer_shell::Edge::Right, true);
}
