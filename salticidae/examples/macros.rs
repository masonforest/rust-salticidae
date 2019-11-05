use salticidae_derive::Serialize;
#[derive(Serialize)]
enum Message {
    Hello { name: String, text: String },
    Ack {},
}
trait Serializable {
    fn serialize(&self) -> Vec<u8>;
}
