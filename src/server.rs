use tokio::net::TcpStream;

use crate::cmd::Command;
use crate::connection::Connection;

pub async fn run(stream: TcpStream) -> anyhow::Result<()> {
    let mut connection = Connection::new(stream);

    loop {
        let frame = connection.read_frame().await?;

        let cmd = Command::from_frame(frame)?;

        dbg!(&cmd);

        cmd.apply(&mut connection).await?;
    }
}
