use std::{str, vec};

use bytes::Bytes;
use thiserror::Error;

use crate::frame::Frame;

pub(crate) struct Parse {
    parts: vec::IntoIter<Frame>,
}

#[derive(Error, Debug)]
pub(crate) enum ParseError {
    #[error("end")]
    End,

    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl Parse {
    pub fn new(frame: Frame) -> Result<Parse, ParseError> {
        let array = match frame {
            Frame::Array(array) => array,
            frame => return Err(anyhow::anyhow!("expected array, got {:?}", frame).into()),
        };

        Ok(Parse {
            parts: array.into_iter(),
        })
    }

    fn next(&mut self) -> Result<Frame, ParseError> {
        self.parts.next().ok_or(ParseError::End)
    }

    pub fn next_string(&mut self) -> Result<String, ParseError> {
        match self.next()? {
            Frame::Bulk(data) => Ok(str::from_utf8(&data)?.to_string()),
            frame => Err(anyhow::anyhow!("expected bulk string, got {:?}", frame).into()),
        }
    }

    pub fn next_bytes(&mut self) -> Result<Bytes, ParseError> {
        match self.next()? {
            Frame::Bulk(data) => Ok(data),
            frame => Err(ParseError::Other(anyhow::anyhow!(
                "expected bulk string, got {:?}",
                frame
            ))),
        }
    }

    pub(crate) fn next_int(&mut self) -> Result<u64, ParseError> {
        match self.next()? {
            Frame::Bulk(data) => {
                let mut out = 0;

                for b in data {
                    out *= 10;
                    out += (b - b'0') as u64;
                }

                Ok(out)
            }
            frame => Err(
                anyhow::anyhow!("protocol error; expected int frame but got {:?}", frame).into(),
            ),
        }
    }
}
