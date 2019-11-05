use crate::error::Error;
use crate::header::{self, Header};
use async_trait::async_trait;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const MAGIC: u32 = 0;
const CHECKSUM_SIZE: usize = 4;

pub trait Serializable {
    fn serialize(&self) -> Vec<u8>;
}

pub trait Deserializable {
    fn deserialize(bytes: &[u8], message_type: u8) -> Self;
}

#[async_trait]
pub trait Stream {
    async fn write_message<S: Serializable + std::marker::Sync>(
        &mut self,
        message: &S,
        message_type: u8,
    );
    async fn read_message(&mut self) -> Result<(Header, Vec<u8>), Error>;
}

#[async_trait]
impl Stream for TcpStream {
    async fn write_message<S: Serializable + std::marker::Sync>(
        &mut self,
        message: &S,
        message_type: u8,
    ) {
        let payload_bytes = message.serialize();
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

    async fn read_message(&mut self) -> Result<(Header, Vec<u8>), Error> {
        let mut buf: [u8; header::LEN] = [0; header::LEN];
        let n = self.read_exact(&mut buf).await?;
        if n == 0 {
            return Err(Error::StreamClosed);
        }
        let header: Header = buf[0..n].to_vec().into();
        let mut payload = vec![0u8; header.length as usize];
        self.read_exact(&mut payload).await?;
        verify_checksum(&header, &payload)?;
        Ok((header, payload))
    }
}

fn verify_checksum(header: &Header, payload: &[u8]) -> Result<(), Error> {
    if check_sum(&payload) == header.check_sum {
        Ok(())
    } else {
        Err(Error::InvalidChecksum)
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
