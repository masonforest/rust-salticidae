extern crate hex;
use salticidae::{Deserialize, Serialize};

trait Serializable {
    fn serialize(&self) -> Vec<u8>;
}

trait Deserializable {
    fn deserialize(bytes: &[u8], message_type: u8) -> Self;
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum Message {
    Hello { name: String, text: String },
    Ack {},
}

#[test]
fn test_serialize_empty() {
    assert_eq!(Message::Ack {}.serialize(), []);
}

#[test]
fn test_round_trip() {
    let message = Message::Hello {
        name: "name".to_string(),
        text: "text".to_string(),
    };
    let message_bytes = message.serialize();

    println!("{:?}", message_bytes);
    assert_eq!(message, Message::deserialize(&message_bytes, 0));
}
