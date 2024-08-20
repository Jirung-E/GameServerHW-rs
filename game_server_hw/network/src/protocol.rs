use std::mem::size_of;


type PacketSizeType = u16;


#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct RawPacket {
    size: PacketSizeType,
    data: Vec<u8>,
}

impl RawPacket {
    pub fn new(data: &[u8]) -> Self {
        let size = size_of::<PacketSizeType>() + data.len();

        Self {
            size: size as PacketSizeType,
            data: data.to_vec(),
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut data = bincode::serialize(&self.size).unwrap();
        data.extend_from_slice(&self.data);

        data
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, bincode::Error> {
        if data.len() < size_of::<PacketSizeType>() {
            return Err(bincode::Error::from(bincode::ErrorKind::SizeLimit));
        }

        let size: PacketSizeType = bincode::deserialize(&data[0..2])?;
        if data.len() < size as usize {
            return Err(bincode::Error::from(bincode::ErrorKind::SizeLimit));
        }

        let data = data[2..size as usize].to_vec();

        Ok(Self {
            size,
            data,
        })
    }
}


#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct MessagePacket {
    size: PacketSizeType,      // u8로는 부족함
    time: u128,
    msg: String,
}

impl MessagePacket {
    pub fn new(time: u128, msg: &str) -> Self {
        let msg = msg.to_string();

        Self {
            size: (size_of::<PacketSizeType>()
                + size_of::<u128>() 
                + msg.len()) as PacketSizeType,
            time,
            msg,
        }
    }

    pub fn time(&self) -> u128 {
        self.time
    }

    pub fn msg(&self) -> &str {
        &self.msg
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

        let size: u16 = bincode::deserialize(&data[0..2])?;
        if data.len() < size as usize {
            return Err(bincode::Error::from(bincode::ErrorKind::SizeLimit));
        }

        let time = bincode::deserialize(&data[2..size_of::<u128>() + 2])?;
        let msg = String::from_utf8_lossy(&data[size_of::<u128>() + 2..size as usize]).to_string();

        Ok(Self {
            size,
            time,
            msg,
        })
    }

    pub fn from_raw(raw: RawPacket) -> Result<Self, bincode::Error> {
        if raw.data().len() < size_of::<u128>() {
            return Err(bincode::Error::from(bincode::ErrorKind::SizeLimit));
        }
        
        let time = bincode::deserialize(&raw.data()[0..size_of::<u128>()])?;
        let msg = String::from_utf8_lossy(&raw.data()[size_of::<u128>()..]);

        Ok(Self::new(time, &msg))
    }

    pub fn as_raw(&self) -> RawPacket {
        let mut data = bincode::serialize(&self.time).unwrap();
        data.extend_from_slice(self.msg.as_bytes());

        RawPacket::new(&data)
    }
}