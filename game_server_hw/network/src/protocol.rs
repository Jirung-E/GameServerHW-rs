use std::mem::size_of;


#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct RawPacket {
    size: u16,
    data: Vec<u8>,
}


#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct MessagePacket {
    size: u16,      // u8로는 부족함
    time: u128,
    msg: String,
}

impl MessagePacket {
    pub fn new(time: u128, msg: &str) -> Self {
        let msg = msg.to_string();

        Self {
            size: size_of::<u16>() as u16 + size_of::<u128>() as u16 + msg.len() as u16,
            time,
            msg,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut data = bincode::serialize(&self.size).unwrap();
        
        let time = bincode::serialize(&self.time).unwrap();
        data.extend_from_slice(&time);

        let msg = self.msg.as_bytes();
        data.extend_from_slice(msg);

        data
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, bincode::Error> {
        if data.len() < size_of::<u16>() + size_of::<u128>() {
            return Err(bincode::Error::from(bincode::ErrorKind::SizeLimit));
        }

        let size = bincode::deserialize(&data[0..2])?;
        if data.len() < size as usize {
            return Err(bincode::Error::from(bincode::ErrorKind::SizeLimit));
        }

        let time = bincode::deserialize(&data[2..size_of::<u128>() + 2])?;
        let msg = String::from_utf8_lossy(&data[size_of::<u128>() + 2..]).to_string();

        Ok(Self {
            size,
            time,
            msg,
        })
    }

    pub fn time(&self) -> u128 {
        self.time
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }
}