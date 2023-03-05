use crate::{Connection, Db, Frame, Parse};

mod echo;
mod get;
mod ping;
mod set;

pub use echo::Echo;
pub use get::Get;
pub use ping::Ping;
pub use set::Set;

#[derive(Debug)]
pub enum Command {
    Ping(Ping),
    Echo(Echo),
    Set(Set),
    Get(Get),
}

impl Command {
    pub fn from_frame(frame: Frame) -> anyhow::Result<Command> {
        let mut parse = Parse::new(frame)?;

        let name = parse.next_string()?;

        match name.to_lowercase().as_str() {
            "ping" => Ok(Command::Ping(Ping::parse_frames()?)),
            "echo" => Ok(Command::Echo(Echo::parse_frames(&mut parse)?)),
            "set" => Ok(Command::Set(Set::parse_frames(&mut parse)?)),
            "get" => Ok(Command::Get(Get::parse_frames(&mut parse)?)),
            _ => Err(anyhow::anyhow!("unknown command: {}", name)),
        }
    }

    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> anyhow::Result<()> {
        match self {
            Command::Ping(cmd) => cmd.apply(dst).await,
            Command::Echo(cmd) => cmd.apply(dst).await,
            Command::Set(cmd) => cmd.apply(db, dst).await,
            Command::Get(cmd) => cmd.apply(db, dst).await,
        }
    }
}
