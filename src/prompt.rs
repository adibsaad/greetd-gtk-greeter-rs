use std::boxed::Box;
use std::string::String;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use dbus::arg::{PropMap, RefArg};
use dbus::blocking::Connection;
use dbus::Path;
use gtk::{
  prelude::*, Align, CssProvider, Orientation, TreeIter, STYLE_PROVIDER_PRIORITY_APPLICATION,
};
use relm::{Channel, Relm, Sender, StreamHandle, Widget};
use relm_derive::{widget, Msg};

use crate::login_manager::LoginManager;

const OTHER_ID: &'static str = "*other";

pub struct Model {
  username: String,
  password: String,
  selected_user: String,
  cached_users: Vec<PropMap>,
  _channel: Channel<LoginResult>,
  _sender: Sender<LoginResult>,
  stream: StreamHandle<Msg>,
  is_submitting: bool,
  other_login: bool,
  info_label: gtk::Label,
  info_msg: String,
  login_manager: Arc<Mutex<LoginManager>>,
}

#[derive(Msg)]
pub enum Msg {
  OnUsernameChange(String),
  OnPasswordChange(String),
  OnLoginSuccess(String),
  OnLoginFail(String),
  OnSelectionChanged(Option<TreeIter>),
  OnSubmit,
  OnCancel,
}

pub enum LoginResult {
  Succuessful(String),
  Failure(String),
}

fn get_cached_users() -> Result<Vec<PropMap>, Box<dyn std::error::Error>> {
  let conn = Connection::new_system()?;

  let proxy = conn.with_proxy(
    "org.freedesktop.Accounts",
    "/org/freedesktop/Accounts",
    Duration::from_millis(5000),
  );

  let (names,): (Vec<Path>,) =
    proxy.method_call("org.freedesktop.Accounts", "ListCachedUsers", ())?;

  Ok(
    names
      .iter()
      .map(|pp| {
        let pr2 = conn.with_proxy(
          "org.freedesktop.Accounts",
          pp.to_string(),
          Duration::from_millis(5000),
        );

        let (all_info,): (PropMap,) = pr2
          .method_call(
            "org.freedesktop.DBus.Properties",
            "GetAll",
            ("org.freedesktop.Accounts.User",),
          )
          .unwrap();
        all_info
      })
      .collect::<Vec<_>>(),
  )
}

#[widget]
impl Widget for Prompt {
  fn model(relm: &Relm<Self>, _: ()) -> Model {
    let stream = relm.stream().clone();
    let (channel, sender) = Channel::new(move |res| match res {
      LoginResult::Succuessful(msg) => {
        stream.emit(Msg::OnLoginSuccess(msg));
      }
      LoginResult::Failure(msg) => {
        stream.emit(Msg::OnLoginFail(msg));
      }
    });

    let cached_users = match get_cached_users() {
      Ok(u) => u,
      _ => vec![],
    };

    let info_label = gtk::Label::new(Some(""));
    info_label.set_max_width_chars(10);
    info_label.set_visible(true);
    info_label.set_line_wrap(true);
    info_label.set_line_wrap_mode(pango::WrapMode::Word);

    Model {
      username: String::new(),
      password: String::new(),
      selected_user: String::new(),
      _channel: channel,
      _sender: sender,
      stream: relm.stream().clone(),
      is_submitting: false,
      other_login: false,
      cached_users,
      info_label,
      info_msg: String::new(),
      login_manager: Arc::new(Mutex::new(LoginManager::new())),
    }
  }
  fn init_view(&mut self) {
    let style_context = self.widgets.prompt.get_style_context();
    let style = include_bytes!("./css/prompt.css");
    let provider = CssProvider::new();
    provider.load_from_data(style).unwrap();
    style_context.add_provider(&provider, STYLE_PROVIDER_PRIORITY_APPLICATION);

    self
      .widgets
      .info_bar
      .get_content_area()
      .pack_start(&self.model.info_label, true, true, 0);

    let model = gtk::ListStore::new(&[str::static_type(), str::static_type()]);
    for (_, user_info) in self.model.cached_users.iter().enumerate() {
      let id = match (&user_info["UserName"]).as_str() {
        Some(s) => s,
        _ => continue,
      };
      let rn = &(user_info["RealName"]);
      let display = match rn.as_str() {
        Some(s) => {
          if s.is_empty() {
            id
          } else {
            s
          }
        }
        _ => continue,
      };

      model.insert_with_values(None, &[0, 1], &[&id, &display]);
    }

    model.insert_with_values(None, &[0, 1], &[&OTHER_ID, &"Other..."]);

    self.widgets.user_list.set_model(Some(&model));
    self.widgets.user_list.set_active(Some(0));
    let renderer_text = gtk::CellRendererText::new();
    self.widgets.user_list.pack_start(&renderer_text, true);
    self
      .widgets
      .user_list
      .add_attribute(&renderer_text, "text", 1);
  }

  fn update(&mut self, event: Msg) {
    match event {
      Msg::OnUsernameChange(text) => {
        let _lock = self.model.stream.lock();
        self.model.username = text;
      }
      Msg::OnPasswordChange(text) => {
        let _lock = self.model.stream.lock();
        self.model.password = text;
      }
      Msg::OnSubmit => {
        if self.model.is_submitting {
          return;
        }

        if self.model.other_login && self.model.username.is_empty() {
          self.model.info_msg = String::from("Enter your username");
          self.model.info_label.set_text(&self.model.info_msg);
          return;
        } else if self.model.password.is_empty() {
          self.model.info_msg = String::from("Enter your password");
          self.model.info_label.set_text(&self.model.info_msg);
          return;
        }

        self.model.info_msg = String::new();
        self.model.info_label.set_text(&self.model.info_msg);
        self.model.is_submitting = true;
        self.process_submit();
      }
      Msg::OnCancel => {
        if self.model.is_submitting {
          return;
        }

        self.clear_fields();
      }
      Msg::OnLoginSuccess(msg) => {
        self.model.is_submitting = false;
        self.model.info_msg = msg.clone();
        self.model.info_label.set_text(&self.model.info_msg);
      }
      Msg::OnLoginFail(msg) => {
        self.model.is_submitting = false;
        self.model.info_msg = msg.clone();
        self.model.info_label.set_text(&self.model.info_msg);
      }
      Msg::OnSelectionChanged(active_idx) => {
        self.clear_fields();

        self.model.selected_user = match active_idx {
          Some(idx) => {
            let model = match self.widgets.user_list.get_model() {
              Some(m) => m,
              _ => return,
            };
            let val = model.get_value(&idx, 0);
            let username = match val.get::<&str>() {
              Ok(s) => match s {
                Some(s) => s,
                _ => return,
              },
              _ => return,
            };
            self.model.other_login = username == OTHER_ID;
            username.to_string()
          }
          _ => "".to_string(),
        };
      }
    }
  }
  fn clear_fields(&mut self) {
    self.model.info_msg = String::new();
    self.model.username = String::new();
    self.model.password = String::new();
    self.model.info_label.set_text(&String::new());
  }
  view! {
    #[name="prompt"]
    gtk::EventBox {
      widget_name: "prompt",
      valign: Align::Start,
      halign: Align::Start,

      gtk::Box {
        orientation: Orientation::Vertical,

        #[name="content_frame"]
        gtk::Frame {
          gtk::Grid {
            orientation: Orientation::Vertical,
            margin_start: 24,
            margin_end: 24,
            margin_top: 24,
            margin_bottom: 24,
            row_spacing: 6,

            #[name="user_list"]
            gtk::ComboBox {
              margin_top: 10,
              sensitive: !self.model.is_submitting,
              changed(combobox) => Msg::OnSelectionChanged(combobox.get_active_iter())
            },
            #[name="username_entry"]
            gtk::Entry {
              widget_name: "username",
              placeholder_text: Some("Enter Username"),
              sensitive: !self.model.is_submitting,
              visible: self.model.other_login,
              property_width_request: 200,
              text: &self.model.username,

              changed(entry) => Msg::OnUsernameChange(entry.get_text().to_string()),
              activate => Msg::OnSubmit,
            },
            #[name="password_entry"]
            gtk::Entry {
              widget_name: "password",
              placeholder_text: Some("Enter Password"),
              margin_bottom: 10,
              sensitive: !self.model.is_submitting,
              visibility: false,
              property_width_request: 200,
              text: &self.model.password,

              changed(entry) => Msg::OnPasswordChange(entry.get_text().to_string()),
              activate => Msg::OnSubmit,
            },
          },
        },

        #[name="info_bar"]
        gtk::InfoBar {
          revealed: !self.model.info_msg.is_empty(),
        },

        gtk::Frame {
          gtk::Box {
            margin_start: 24,
            margin_end: 24,
            margin_top: 24,
            margin_bottom: 24,

            gtk::Button {
              label: &String::from("Cancel"),
              sensitive: !self.model.is_submitting,
              clicked => Msg::OnCancel
            },
            gtk::Button {
              label: &String::from("Log In"),
              sensitive: !self.model.is_submitting,
              clicked => Msg::OnSubmit,
              child: {
                pack_type: gtk::PackType::End,
              }
            },
          }
        }
      }
    }
  }
}

impl Prompt {
  fn process_submit(&mut self) {
    let sender = self.model._sender.clone();
    let username = self.model.username.clone();
    let password = self.model.password.clone();
    let selected_user = self.model.selected_user.clone();
    let lm = Arc::clone(&self.model.login_manager);
    thread::spawn(move || {
      let res = lm.lock().unwrap().submit(
        if selected_user == OTHER_ID {
          username
        } else {
          selected_user
        },
        password,
      );
      // Wait a bit, so user can't spam.
      thread::sleep(Duration::from_millis(1000));
      match res {
        Ok(msg) => sender.send(LoginResult::Succuessful(msg.into())),
        Err(msg) => sender.send(LoginResult::Failure(msg.to_string().into())),
      }
    });
  }
}
