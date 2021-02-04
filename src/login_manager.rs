use std::boxed::Box;
use std::env;
use std::error::Error;
use std::os::unix::net::UnixStream;
use std::string::String;

use greetd_ipc::{codec::SyncCodec, AuthMessageType, ErrorType, Request, Response};

pub struct LoginManager {
  stream: Option<UnixStream>,
}

impl Clone for LoginManager {
  fn clone(&self) -> Self {
    match &self.stream {
      Some(x) => match Some(x.try_clone()) {
        Some(y) => match y {
          Ok(z) => LoginManager { stream: Some(z) },
          _ => LoginManager { stream: None },
        },
        _ => LoginManager { stream: None },
      },
      None => LoginManager { stream: None },
    }
  }
}

impl LoginManager {
  pub fn new() -> Self {
    LoginManager { stream: None }
  }
  pub fn submit(&mut self, username: String, password: String) -> Result<String, Box<dyn Error>> {
    let req = Request::CreateSession {
      username: username.clone(),
    };
    let req2 = Request::PostAuthMessageResponse {
      response: Some(password.clone()),
    };
    let mut stream = match self.get_stream() {
      Ok(s) => s,
      Err(n) => return Err(n),
    };
    req.write_to(&mut stream)?;
    match Response::read_from(&mut stream)? {
      Response::AuthMessage {
        auth_message,
        auth_message_type,
      } => match auth_message_type {
        AuthMessageType::Error | AuthMessageType::Info => {
          let _ = self.cancel();
          return Err(auth_message.into());
        }
        _ => (),
      },
      Response::Error {
        error_type,
        description,
      } => {
        let _ = self.cancel();
        return match error_type {
          ErrorType::AuthError => Err("Login failed".into()),
          ErrorType::Error => Err(format!("err: {}", description).into()),
        };
      }
      _ => (),
    }

    req2.write_to(&mut stream)?;
    match Response::read_from(&mut stream)? {
      Response::AuthMessage {
        auth_message,
        auth_message_type,
      } => {
        return Ok(format!("2authmsg1: {} , {:?}", auth_message, auth_message_type).into());
      }
      Response::Success => {
        Request::StartSession {
          cmd: vec![String::from("sway")],
        }
        .write_to(&mut stream)?;
        match Response::read_from(&mut stream)? {
          Response::AuthMessage {
            auth_message,
            auth_message_type,
          } => {
            return Ok(format!("2authmsg2: {} , {:?}", auth_message, auth_message_type).into());
          }
          Response::Success => std::process::exit(0),
          Response::Error {
            error_type,
            description: _,
          } => {
            let _ = self.cancel();
            return match error_type {
              ErrorType::AuthError => Err("Login failed".into()),
              ErrorType::Error => Err("Login failed".into()),
            };
          }
        }
      }
      Response::Error {
        error_type,
        description,
      } => {
        let _ = self.cancel();
        return match error_type {
          ErrorType::AuthError => Err("Login failed".into()),
          ErrorType::Error => Err(format!("err: {}", description).into()),
        };
      }
    }
  }
  fn cancel(&mut self) -> Result<String, Box<dyn Error>> {
    let mut stream = match self.get_stream() {
      Ok(s) => s,
      Err(n) => return Err(n),
    };
    Request::CancelSession.write_to(&mut stream)?;
    match Response::read_from(&mut stream)? {
      Response::AuthMessage { .. } => Err(format!("Unexecpted auth response").into()),
      Response::Success => Ok(String::from("cancelled successfully")),
      Response::Error {
        error_type,
        description,
      } => {
        return Err(format!("err: {:?}: {}", error_type, description).into());
      }
    }
  }
  fn get_stream(&mut self) -> Result<&UnixStream, Box<dyn Error>> {
    match self.stream {
      Some(ref s) => Ok(s),
      None => {
        self.stream = Some(UnixStream::connect(
          env::var("GREETD_SOCK").expect("GREETD_SOCK not set"),
        )?);
        Ok(self.stream.as_ref().unwrap())
      }
    }
  }
}
