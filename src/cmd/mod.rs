use self::echo::Echo;
use self::ping::Ping;

mod echo;
mod ping;

use crate::connection::Connection;
use crate::frame::Frame;
use crate::parse::Parse;

#[derive(Debug)]
pub enum Command {
    Ping(Ping),
    Echo(Echo),
}

impl Command {
    pub fn from_frame(frame: Frame) -> anyhow::Result<Command> {
        let mut parse = Parse::new(frame)?;

        let name = parse.next_string()?;

        match name.to_lowercase().as_str() {
            "ping" => Ok(Command::Ping(Ping::parse_frames()?)),
            "echo" => Ok(Command::Echo(Echo::parse_frames(&mut parse)?)),
            _ => Err(anyhow::anyhow!("unknown command: {}", name)),
        }
    }

    pub(crate) async fn apply(self, dst: &mut Connection) -> anyhow::Result<()> {
        match self {
            Command::Ping(cmd) => cmd.apply(dst).await,
            Command::Echo(cmd) => cmd.apply(dst).await,
        }
    }
}
