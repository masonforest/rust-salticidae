#![feature(arbitrary_enum_discriminant)]
extern crate byteorder;
extern crate hex;
use async_trait::async_trait;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const HEADER_LEN: usize = 13;

#[derive(PartialEq, Debug)]
pub struct Header {
    pub magic: u32,
    pub opcode: u8,
    pub length: u32,
    pub check_sum: [u8; 4],
}

impl From<Header> for Vec<u8> {
    fn from(header: Header) -> Self {
        let mut wtr = vec![];
        wtr.write_u32::<LittleEndian>(header.magic).unwrap();
        wtr.write_u8(header.opcode).unwrap();
        wtr.write_u32::<LittleEndian>(header.length).unwrap();
        wtr.extend(&header.check_sum);
        wtr
    }
}

impl From<Vec<u8>> for Header {
    fn from(vec: Vec<u8>) -> Self {
        let mut rdr = Cursor::new(vec);
        let magic = rdr.read_u32::<LittleEndian>().unwrap();
        let opcode = rdr.read_u8().unwrap();
        let length = rdr.read_u32::<LittleEndian>().unwrap();
        let mut check_sum: [u8; 4] = Default::default();
        Read::read_exact(&mut rdr, &mut check_sum).unwrap();

        Header {
            length,
            opcode,
            magic,
            check_sum,
        }
    }
}

#[async_trait]
pub trait Stream {
    async fn write_message<E: Encodable + std::marker::Sync>(
        &mut self,
        message: &E,
        message_type: u8,
    );
    async fn read_message(&mut self) -> (Header, Vec<u8>);
}

pub trait Encodable {
    fn encode(&self) -> Vec<u8>;
    fn decode(vec: &[u8], message_type: u8) -> Self;
}

#[async_trait]
impl Stream for TcpStream {
    async fn write_message<E: Encodable + std::marker::Sync>(
        &mut self,
        message: &E,
        message_type: u8,
    ) {
        let payload_bytes = message.encode();
        let header_bytes: Vec<u8> = Header {
            magic: 0,
            opcode: message_type,
            length: payload_bytes.len() as u32,
            check_sum: [1 as u8, 2, 3, 4],
        }
        .into();
        self.write_all(&header_bytes).await.unwrap();
        self.write_all(&payload_bytes).await.unwrap();
    }

    async fn read_message(&mut self) -> (Header, Vec<u8>) {
        let mut buf: [u8; HEADER_LEN] = [0; HEADER_LEN];
        let n = match self.read_exact(&mut buf).await {
            Ok(n) if n == 0 => panic!("Got 0 bytes from socket"),
            Ok(n) => n,
            Err(e) => {
                panic!("failed to read from socket; err = {:?}", e);
            }
        };
        let header: Header = buf[0..n].to_vec().into();
        let mut payload_bytes = vec![0u8; header.length as usize];
        self.read_exact(&mut payload_bytes).await.unwrap();
        (header, payload_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::{Encodable, Stream};
    use crate::byteorder::{ReadBytesExt, WriteBytesExt};
    use byteorder::LittleEndian;
    use std::io::{Cursor, Read};
    use tokio::net::TcpListener;
    use tokio::net::TcpStream;
    use tokio::runtime::Runtime;
    use tokio::sync::oneshot;

    #[derive(Debug, PartialEq, Clone)]
    enum Message {
        Hello { name: String, text: String },
        Ack {},
    }

    impl Encodable for Message {
        fn encode(&self) -> Vec<u8> {
            match self {
                Message::Hello { name, text } => {
                    let mut wtr = vec![];
                    wtr.write_u32::<LittleEndian>(name.len() as u32).unwrap();
                    wtr.extend(name.as_bytes().to_vec());
                    wtr.extend(text.as_bytes().to_vec());
                    wtr
                }
                Message::Ack {} => vec![],
            }
        }

        fn decode(bytes: &[u8], message_type: u8) -> Self {
            match message_type {
                0 => {
                    let mut rdr = Cursor::new(bytes);
                    let name_len = rdr.read_u32::<LittleEndian>().unwrap();
                    let mut name_bytes = vec![0u8; name_len as usize];
                    let mut text_bytes = Default::default();
                    Read::read_exact(&mut rdr, &mut name_bytes).unwrap();
                    Read::read_to_end(&mut rdr, &mut text_bytes).unwrap();
                    Message::Hello {
                        name: std::str::from_utf8(&name_bytes.clone())
                            .unwrap()
                            .to_string(),
                        text: std::str::from_utf8(&text_bytes.clone())
                            .unwrap()
                            .to_string(),
                    }
                }
                1 => Message::Ack {},
                _ => panic!("unkown type"),
            }
        }
    }

    #[test]
    fn test_listen() {
        let rt = Runtime::new().unwrap();
        let (tx, rx) = oneshot::channel();
        rt.block_on(async move {
            let listener = TcpListener::bind("127.0.0.1:8081").await.unwrap();
            tx.send(listener).unwrap();
        });
        rt.spawn(async move {
            let mut listener = rx.await.unwrap();
            loop {
                let (mut socket, _) = listener.accept().await.unwrap();
                tokio::spawn(async move {
                    let (header, received_bytes) = socket.read_message().await;
                    if let Message::Hello { .. } = Message::decode(&received_bytes, header.opcode) {
                        socket.write_message(&Message::Ack {}, 1).await;
                    }
                });
            }
        });

        rt.block_on(async move {
            let mut stream = TcpStream::connect("127.0.0.1:8081").await.unwrap();
            let original = Message::Hello {
                name: "alice".to_string(),
                text: "Hello there!".to_string(),
            };
            stream.write_message(&original, 0).await;
            let (header, received_bytes) = stream.read_message().await;
            assert_eq!(
                Message::Ack {},
                Message::decode(&received_bytes, header.opcode)
            );
        })
    }
}
