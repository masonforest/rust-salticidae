extern crate hex;
extern crate salticidae_derive;
use salticidae_derive::Serialize;

trait Serializable {
    fn serialize(&self) -> Vec<u8>;
}

#[derive(Serialize)]
enum Message {
    Hello { name: String, text: String },
    Ack {},
}

#[test]
fn test_serialize_empty() {
    assert_eq!(Message::Ack{}.serialize(), []);
}

#[test]
fn test_serialize() {
    let message = Message::Hello{
        name: "name".to_string(),
        text: "text".to_string(),
    };

    assert_eq!(message.serialize(), hex::decode("6e616d650400000074657874").unwrap());
}
