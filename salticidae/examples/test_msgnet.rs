use salticidae::{error::Error, Deserializable, Deserialize, Serializable, Serialize, Stream};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::sync::{
    oneshot,
    oneshot::{Receiver, Sender},
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
enum Message {
    Hello { name: String, text: String },
    Ack {},
}

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
            respond(&mut socket, server_name).await;
        });
    }
}

async fn respond(socket: &mut TcpStream, server_name: &'static str) {
    let (header, received_bytes) = socket.read_message().await.unwrap();
    if let Message::Hello { name, text } = Message::deserialize(&received_bytes, header.opcode) {
        println!("[{}] {} says {}", server_name, name, text);
        socket.write_message(&Message::Ack {}, 1).await;
    }
}

async fn say_hello(addr: &str, client_name: &str) {
    let mut socket = TcpStream::connect(addr).await.unwrap();

    println!("[{}] connected, sending hello.", client_name);
    let original = Message::Hello {
        name: client_name.to_string(),
        text: "Hello there!".to_string(),
    };
    socket.write_message(&original, 0).await;
    let (header, received_bytes) = socket.read_message().await.unwrap();
    if let Message::Ack {} = Message::deserialize(&received_bytes, header.opcode) {
        println!("[{}] the peer knows", client_name);
    };
}

fn main() -> Result<(), Error> {
    let alices_addr = "127.0.0.1:12345";
    let bobs_addr = "127.0.0.1:12346";
    let rt = Runtime::new().unwrap();
    let (tx1, rx1) = oneshot::channel();
    let (tx2, rx2) = oneshot::channel();
    rt.block_on(bind(alices_addr, tx1));
    rt.block_on(bind(bobs_addr, tx2));
    rt.spawn(listen(rx1, "alice"));
    rt.spawn(listen(rx2, "bob"));
    rt.spawn(say_hello(alices_addr, "alice"));
    rt.spawn(say_hello(alices_addr, "bob"));
    rt.block_on(tokio::future::pending::<()>());
    Ok(())
}
