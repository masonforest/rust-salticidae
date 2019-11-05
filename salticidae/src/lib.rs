extern crate byteorder;
extern crate crypto;
extern crate hex;

pub use byteorder::{LittleEndian, WriteBytesExt};
pub use stream::{Encodable, Stream};
pub mod error;
pub mod header;
pub mod stream;

#[cfg(feature = "salticidae_derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate salticidae_derive;
#[cfg(feature = "salticidae_derive")]
#[doc(hidden)]
pub use salticidae_derive::Serialize;

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
                    let (header, received_bytes) = socket.read_message().await.unwrap();
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
            let (header, received_bytes) = stream.read_message().await.unwrap();
            assert_eq!(
                Message::Ack {},
                Message::decode(&received_bytes, header.opcode)
            );
        })
    }
}
