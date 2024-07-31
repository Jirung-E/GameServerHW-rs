use bytes::Bytes;


#[derive(Debug, PartialEq)]
pub enum Packet {
    Complete(Bytes),
    Incomplete(Bytes),
}
