use crate::connection::Connection;
use crate::frame::Frame;

#[derive(Debug, Default)]
pub struct Ping {}

impl Ping {
    pub fn parse_frames() -> anyhow::Result<Ping> {
        Ok(Ping::default())
    }

    pub async fn apply(self, dst: &mut Connection) -> anyhow::Result<()> {
        let response = Frame::Simple("PONG".to_string());

        dst.write_frame(&response).await?;

        Ok(())
    }
}
