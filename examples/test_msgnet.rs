use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use salticidae::{Encodable, Stream};
use std::io::{Cursor, Read};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::sync::{
    oneshot,
    oneshot::{Receiver, Sender},
};

async fn bind(addr: &str, tx: Sender<TcpListener>) {
    let listener = TcpListener::bind(addr).await.unwrap();
    tx.send(listener).unwrap();
}

async fn listen(rx1: Receiver<TcpListener>, server_name: &'static str) {
    let mut listener = rx1.await.unwrap();
    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        println!("[{}] accepted, waiting for greetings.", server_name);
        tokio::spawn(async move {
            let (header, received_bytes) = socket.read_message().await.unwrap();
            if let Message::Hello { name, text } = Message::decode(&received_bytes, header.opcode) {
                println!("[{}] {} says {}", server_name, name, text);
                socket.write_message(&Message::Ack {}, 1).await;
            }
        });
    }
}

async fn connect(addr: &str, client_name: &str) {
    let mut socket = TcpStream::connect(addr).await.unwrap();

    println!("[{}] connected, sending hello.", client_name);
    let original = Message::Hello {
        name: client_name.to_string(),
        text: "Hello there!".to_string(),
    };
    socket.write_message(&original, 0).await;
    let (header, received_bytes) = socket.read_message().await.unwrap();
    if let Message::Ack {} = Message::decode(&received_bytes, header.opcode) {
        println!("[{}] the peer knows", client_name)
    }
}

fn main() {
    let alices_addr = "127.0.0.1:12345";
    let bobs_addr = "127.0.0.1:12346";
    let rt = Runtime::new().unwrap();
    let (tx1, rx1) = oneshot::channel();
    let (tx2, rx2) = oneshot::channel();
    rt.block_on(bind(alices_addr, tx1));
    rt.block_on(bind(bobs_addr, tx2));
    rt.spawn(listen(rx1, "alice"));
    rt.spawn(listen(rx2, "bob"));
    rt.spawn(connect(alices_addr, "bob"));
    rt.spawn(connect(bobs_addr, "alice"));
    rt.block_on(tokio::future::pending::<()>());
}

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
