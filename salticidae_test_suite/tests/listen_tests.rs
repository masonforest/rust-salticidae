extern crate byteorder;
use salticidae::{Deserializable, Deserialize, Serializable, Serialize, Stream};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::sync::oneshot;

#[test]
fn test_listen() {
    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    enum Message {
        Hello { name: String, text: String },
        Ack {},
    }

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
                if let Message::Hello { .. } = Message::deserialize(&received_bytes, header.opcode)
                {
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
            Message::deserialize(&received_bytes, header.opcode)
        );
    })
}
