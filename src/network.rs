use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::io;
use bytes::BytesMut;

use crate::protocol::{encode_message, decode_message, matrixserver::MatrixServerMessage};

pub struct MatrixConnection {
    stream: TcpStream,
    buffer: BytesMut,
}

impl MatrixConnection {
    pub async fn connect(addr: &str) -> io::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self {
            stream,
            buffer: BytesMut::with_capacity(8192),
        })
    }

    pub async fn send_message(&mut self, msg: &MatrixServerMessage) -> io::Result<()> {
        let framed = encode_message(msg);
        self.stream.write_all(&framed).await?;
        self.stream.flush().await?;
        Ok(())
    }

    pub async fn read_message(&mut self) -> io::Result<Option<MatrixServerMessage>> {
        loop {
            // Check if we have a full frame in the buffer (terminated by 0x00)
            if let Some(pos) = self.buffer.iter().position(|&b| b == 0x00) {
                // Extract the frame including the 0x00 byte
                let frame = self.buffer.split_to(pos + 1);
                
                // Attempt to decode
                match decode_message(&frame) {
                    Ok(msg) => return Ok(Some(msg)),
                    Err(e) => {
                        eprintln!("Failed to decode message: {}", e);
                        // Skip malformed frame and continue loop
                    }
                }
            }

            // Read more data from socket
            let mut temp_buf = [0u8; 4096];
            let n = self.stream.read(&mut temp_buf).await?;
            if n == 0 {
                // Connection closed
                return Ok(None);
            }
            self.buffer.extend_from_slice(&temp_buf[..n]);
        }
    }
}
