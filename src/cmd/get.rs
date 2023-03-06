use crate::Connection;
use crate::Db;
use crate::Frame;
use crate::Parse;

#[derive(Debug, Default)]
pub struct Get {
    key: String,
}

impl Get {
    pub(crate) fn parse_frames(parse: &mut Parse) -> anyhow::Result<Get> {
        let key = parse.next_string()?;
        Ok(Get { key })
    }

    pub async fn apply(self, db: &Db, dst: &mut Connection) -> anyhow::Result<()> {
        let value = db.get(&self.key);

        let response = match value {
            Some(value) => Frame::Bulk(value),
            None => Frame::Null,
        };

        dst.write_frame(&response).await?;

        Ok(())
    }
}
