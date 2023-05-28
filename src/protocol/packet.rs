pub const MAX_PACKET_SIZE: usize = 1464;
use std::convert::TryFrom;

pub use super::header::Header;
use super::header::HEADER_SIZE;

#[derive(Debug)]
pub enum Error {
    MissingMagicNumber,
    MalformedFormat,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::MissingMagicNumber => "Missing magic number",
                Error::MalformedFormat => "Malformed format",
            }
        )
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::MissingMagicNumber => "Missing magic number",
            Error::MalformedFormat => "Malformed format",
        }
    }
}

pub struct Packet {
    header: Header,
    pub data: Vec<u8>,
}

impl Packet {
    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn header_mut(&mut self) -> &mut Header {
        &mut self.header
    }
}

impl TryFrom<&[u8]> for Packet {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let pkt = Packet {
            header: Header::try_from(value)?,
            data: Vec::from(&value[HEADER_SIZE..]),
        };
        Ok(pkt)
    }
}

impl From<Packet> for Vec<u8> {
    fn from(pkt: Packet) -> Vec<u8> {
        let mut buf = Vec::with_capacity(HEADER_SIZE + pkt.data.len());
        let header: [u8; HEADER_SIZE] = pkt.header.into();
        buf.extend_from_slice(&header);
        buf.extend_from_slice(&pkt.data);
        buf
    }
}
