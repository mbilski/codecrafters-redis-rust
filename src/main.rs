use std::{io::Cursor, num::TryFromIntError};

use tokio::{
    io::BufWriter,
    net::{TcpListener, TcpStream},
};

use bytes::{Buf, Bytes, BytesMut};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    println!("Listening on {}", listener.local_addr().unwrap());

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                println!("New client connected");

                tokio::spawn(async move {
                    match process(stream).await {
                        Ok(_) => println!("Client disconnected"),
                        Err(e) => eprintln!("error handling connection: {}", e),
                    }
                });
            }
            Err(e) => {
                eprintln!("error accepting connection: {}", e);
            }
        }
    }
}

async fn process(stream: TcpStream) -> anyhow::Result<()> {
    let mut connection = Connection::new(stream);

    loop {
        let request = connection.read_frame().await?;

        dbg!(request);

        connection
            .write_frame(&Frame::Simple("PONG".into()))
            .await?;
    }
}
struct Connection {
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
            Frame::Bulk(_) => todo!(),
            Frame::Array(_) => todo!(),
        }

        self.stream.flush().await?;

        Ok(())
    }

    pub fn parse_frame(&mut self) -> Result<Frame, FrameError> {
        let mut buf = Cursor::new(&self.buffer[..]);

        let len = buf.position() as usize;

        buf.set_position(0);

        let frame = Frame::parse(&mut buf)?;

        self.buffer.advance(len);

        Ok(frame)
    }
}

#[derive(Clone, Debug)]
pub enum Frame {
    Simple(String),
    Bulk(Bytes),
    Array(Vec<Frame>),
}

#[derive(Error, Debug)]
pub enum FrameError {
    #[error("incomplete")]
    Incomplete,
    #[error("invalid")]
    Invalid,
    #[error(transparent)]
    InvalidDecimal(#[from] TryFromIntError),
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
