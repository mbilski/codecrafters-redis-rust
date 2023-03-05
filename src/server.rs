use tokio::net::TcpStream;

use crate::cmd::Command;
use crate::Connection;
use crate::Db;

pub async fn run(db: Db, stream: TcpStream) -> anyhow::Result<()> {
    let mut connection = Connection::new(stream);

    loop {
        let frame = connection.read_frame().await?;

        let cmd = Command::from_frame(frame)?;

        dbg!(&cmd);

        cmd.apply(&db, &mut connection).await?;
    }
}
