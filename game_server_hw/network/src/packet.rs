#[derive(Debug, PartialEq)]
pub enum Message {
    Update,
    Remove,
    Unknown,
}

#[derive(Debug)]
pub enum Packet {
    Complete(Message),
    Incomplete(Vec<u8>),
}
use Packet::*;

impl PartialEq for Packet {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Complete(a), Complete(b)) => a == b,
            (Incomplete(a), Incomplete(b)) => String::from_utf8_lossy(a) == String::from_utf8_lossy(b),
            _ => false,
        }
    }
}
