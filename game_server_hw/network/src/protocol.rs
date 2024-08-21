use std::mem::size_of;


#[repr(C, packed)]
#[derive(Debug, PartialEq, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct PacketType(u8);
impl PacketType {
    pub const RAW: Self = Self(0);
    pub const MESSAGE: Self = Self(1);
}

pub type PacketSize = u16;


#[repr(C, packed)]
#[derive(Debug, PartialEq, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct PacketHeader {
    size: PacketSize,
    packet_type: PacketType,
}


#[derive(Debug, PartialEq)]
pub struct RawPacket {
    header: PacketHeader,
    data: Vec<u8>,
}

impl RawPacket {
    pub fn new(packet_type: PacketType, data: &[u8]) -> Self {
        let size = (size_of::<PacketHeader>() + data.len()) as PacketSize;

        Self {
            header: PacketHeader {
                size,
                packet_type,
            },
            data: data.to_vec(),
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut data = bytemuck::bytes_of(&self.header).to_vec();
        data.extend_from_slice(&self.data);

        data
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
        if data.len() < size_of::<PacketHeader>() {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data"));
        }

        let header = *bytemuck::from_bytes::<PacketHeader>(&data[0..size_of::<PacketHeader>()]);
        if data.len() < header.size as usize {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data"));
        }

        let data = data[size_of::<PacketHeader>()..header.size as usize].to_vec();

        Ok(Self {
            header,
            data,
        })
    }
}


#[derive(Debug, PartialEq)]
pub struct MessagePacket {
    pub time: u128,
    pub msg: String,
}

impl MessagePacket {
    pub fn new(time: u128, msg: &str) -> Self {
        Self {
            time,
            msg: msg.to_string(),
        }
    }

    pub fn from_raw(raw: RawPacket) -> Result<Self, std::io::Error> {
        if raw.data().len() < size_of::<u128>() {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data"));
        }
        
        let time = *bytemuck::from_bytes::<u128>(&raw.data()[0..size_of::<u128>()]);
        let msg = String::from_utf8_lossy(&raw.data()[size_of::<u128>()..]);

        Ok(Self::new(time, &msg))
    }

    pub fn as_raw(&self) -> RawPacket {
        let mut data = bytemuck::bytes_of(&self.time).to_vec();
        data.extend_from_slice(self.msg.as_bytes());

        RawPacket::new(PacketType::MESSAGE, &data)
    }
}