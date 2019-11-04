use crypto::digest::Digest;
use crypto::sha1::Sha1;
use crate::header::{self, Header};
use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const MAGIC: u32 = 0;
const CHECKSUM_SIZE: usize = 4;

pub trait Encodable {
    fn encode(&self) -> Vec<u8>;
    fn decode(vec: &[u8], message_type: u8) -> Self;
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

#[async_trait]
impl Stream for TcpStream {
    async fn write_message<E: Encodable + std::marker::Sync>(
        &mut self,
        message: &E,
        message_type: u8,
    ) {
        let payload_bytes = message.encode();
        let header_bytes: Vec<u8> = Header {
            magic: MAGIC,
            opcode: message_type,
            length: payload_bytes.len() as u32,
            check_sum: check_sum(&payload_bytes),
        }
        .into();
        self.write_all(&header_bytes).await.unwrap();
        self.write_all(&payload_bytes).await.unwrap();
    }

    async fn read_message(&mut self) -> (Header, Vec<u8>) {
        let mut buf: [u8; header::LEN] = [0; header::LEN];
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
        if check_sum(&payload_bytes) != header.check_sum {
            panic!("invalid check sum");
        }
        (header, payload_bytes)
    }
}

fn check_sum(bytes: &[u8]) -> [u8; CHECKSUM_SIZE] {
    let mut hash = [0; 20];
    let mut output_bytes = [0; CHECKSUM_SIZE];
    let mut hasher = Sha1::new();
    hasher.input(bytes);
    hasher.result(&mut hash);
    output_bytes.copy_from_slice(&hash[0..CHECKSUM_SIZE]);
    output_bytes
}
