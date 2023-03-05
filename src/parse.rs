use std::{str, vec};

use crate::frame::Frame;

pub(crate) struct Parse {
    parts: vec::IntoIter<Frame>,
}

impl Parse {
    pub fn new(frame: Frame) -> anyhow::Result<Parse> {
        let array = match frame {
            Frame::Array(array) => array,
            frame => return Err(anyhow::anyhow!("expected array, got {:?}", frame)),
        };

        Ok(Parse {
            parts: array.into_iter(),
        })
    }

    fn next(&mut self) -> anyhow::Result<Frame> {
        self.parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing frame"))
    }

    pub fn next_string(&mut self) -> anyhow::Result<String> {
        match self.next()? {
            Frame::Bulk(data) => Ok(str::from_utf8(&data)?.to_string()),
            frame => Err(anyhow::anyhow!("expected bulk string, got {:?}", frame)),
        }
    }
}
