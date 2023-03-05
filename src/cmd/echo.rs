use bytes::Bytes;

use crate::connection::Connection;
use crate::frame::Frame;
use crate::parse::Parse;

#[derive(Debug, Default)]
pub struct Echo {
    message: String,
}

impl Echo {
    pub(crate) fn parse_frames(parse: &mut Parse) -> anyhow::Result<Echo> {
        let message = parse.next_string()?;
        Ok(Echo { message })
    }

    pub async fn apply(self, dst: &mut Connection) -> anyhow::Result<()> {
        let response = Frame::Bulk(Bytes::from(self.message));

        dst.write_frame(&response).await?;

        Ok(())
    }
}
