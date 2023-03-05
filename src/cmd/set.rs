use crate::Connection;
use crate::Db;
use crate::Frame;
use crate::Parse;

#[derive(Debug, Default)]
pub struct Set {
    key: String,
    value: String,
}

impl Set {
    pub(crate) fn parse_frames(parse: &mut Parse) -> anyhow::Result<Set> {
        let key = parse.next_string()?;
        let value = parse.next_string()?;
        Ok(Set { key, value })
    }

    pub async fn apply(self, db: &Db, dst: &mut Connection) -> anyhow::Result<()> {
        let response = Frame::Simple("OK".to_string());

        db.lock().unwrap().insert(self.key, self.value);

        dst.write_frame(&response).await?;

        Ok(())
    }
}
