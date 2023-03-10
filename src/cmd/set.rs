use std::time::Duration;

use bytes::Bytes;

use crate::Connection;
use crate::Db;
use crate::Frame;
use crate::{Parse, ParseError};

#[derive(Debug, Default)]
pub struct Set {
    key: String,
    value: Bytes,
    expire: Option<Duration>,
}

impl Set {
    pub(crate) fn parse_frames(parse: &mut Parse) -> anyhow::Result<Set> {
        let key = parse.next_string()?;
        let value = parse.next_bytes()?;

        let mut expire = None;

        match parse.next_string() {
            Ok(s) if s.to_lowercase() == "px" => {
                let miliseconds = parse.next_int()?;
                expire = Some(Duration::from_millis(miliseconds));
            }
            Ok(_) => return Err(anyhow::anyhow!("invalid syntax")),
            Err(ParseError::End) => {}
            Err(e) => return Err(e.into()),
        }

        Ok(Set { key, value, expire })
    }

    pub async fn apply(self, db: &Db, dst: &mut Connection) -> anyhow::Result<()> {
        db.set(self.key, self.value, self.expire);

        let response = Frame::Simple("OK".to_string());

        dst.write_frame(&response).await?;

        Ok(())
    }
}
