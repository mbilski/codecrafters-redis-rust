pub mod server;

pub mod cmd;
pub use cmd::Command;

mod connection;
pub use connection::Connection;

mod frame;
pub use frame::{Frame, FrameError};

mod parse;
use parse::Parse;

pub type Db = std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, String>>>;
