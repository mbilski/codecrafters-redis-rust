use std::io::Cursor;

use bytes::{Buf, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::{io::BufWriter, net::TcpStream};

use crate::{Frame, FrameError};

pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    pub async fn read_frame(&mut self) -> anyhow::Result<Frame> {
        loop {
            if self.stream.read_buf(&mut self.buffer).await? == 0 {
                return Err(anyhow::anyhow!("connection closed"));
            };

            match self.parse_frame() {
                Ok(frame) => return Ok(frame),
                Err(FrameError::Incomplete) => continue,
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
    }

    pub async fn write_frame(&mut self, frame: &Frame) -> anyhow::Result<()> {
        match frame {
            Frame::Simple(s) => {
                self.stream.write_all(b"+").await?;
                self.stream.write_all(s.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Bulk(b) => {
                self.stream.write_all(b"$").await?;
                self.stream
                    .write_all(b.len().to_string().as_bytes())
                    .await?;
                self.stream.write_all(b"\r\n").await?;
                self.stream.write_all(b).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Null => {
                self.stream.write_all(b"$-1\r\n").await?;
            }
            Frame::Array(_) => todo!(),
        }

        self.stream.flush().await?;

        Ok(())
    }

    pub fn parse_frame(&mut self) -> Result<Frame, FrameError> {
        let mut buf = Cursor::new(&self.buffer[..]);

        let frame = Frame::parse(&mut buf)?;

        let len = buf.position() as usize;

        self.buffer.advance(len);

        Ok(frame)
    }
}
