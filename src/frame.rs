use std::io::Cursor;

use bytes::{Buf, Bytes};
use thiserror::Error;

#[derive(Clone, Debug)]
pub enum Frame {
    Simple(String),
    Bulk(Bytes),
    Array(Vec<Frame>),
    Null,
}

#[derive(Error, Debug)]
pub enum FrameError {
    #[error("incomplete")]
    Incomplete,
    #[error("invalid")]
    Invalid,
    #[error(transparent)]
    InvalidDecimal(#[from] std::num::TryFromIntError),
}

impl Frame {
    pub fn parse(buf: &mut Cursor<&[u8]>) -> Result<Frame, FrameError> {
        match get_u8(buf)? {
            b'$' => {
                let len: usize = get_decimal(buf)?.try_into()?;
                let n = len + 2;

                if buf.remaining() < n {
                    return Err(FrameError::Incomplete);
                }

                let data = Bytes::copy_from_slice(&buf.chunk()[..len]);

                skip(buf, n)?;

                Ok(Frame::Bulk(data))
            }
            b'*' => {
                let len: usize = get_decimal(buf)?.try_into()?;
                let mut out = Vec::with_capacity(len);

                for _ in 0..len {
                    out.push(Frame::parse(buf)?);
                }

                Ok(Frame::Array(out))
            }
            _ => Err(FrameError::Invalid),
        }
    }
}

fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8, FrameError> {
    if !src.has_remaining() {
        return Err(FrameError::Incomplete);
    }

    Ok(src.get_u8())
}

fn get_decimal(src: &mut Cursor<&[u8]>) -> Result<u64, FrameError> {
    let line = get_line(src)?;

    let mut out = 0;

    for b in line {
        out *= 10;
        out += (b - b'0') as u64;
    }

    Ok(out)
}

fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], FrameError> {
    let start = src.position() as usize;
    let end = src.get_ref().len() - 1;

    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            src.set_position((i + 2) as u64);
            return Ok(&src.get_ref()[start..i]);
        }
    }

    Err(FrameError::Incomplete)
}

fn skip(src: &mut Cursor<&[u8]>, n: usize) -> Result<(), FrameError> {
    if src.remaining() < n {
        return Err(FrameError::Incomplete);
    }

    src.advance(n);
    Ok(())
}
