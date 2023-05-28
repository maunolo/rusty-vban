use std::convert::TryFrom;

use byteorder::{ByteOrder, LittleEndian};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

use super::packet::Error;

pub const MAX_NUM_SAMPLES: usize = 732;
const SAMPLE_RATE_MASK: u8 = 0b00011111;
pub const HEADER_SIZE: usize = 22;

#[derive(Copy, Clone, Debug)]
pub struct Header {
    sample_rate: SampleRate,
    num_channels: u8,
    stream_name: [u8; 16],
    frame_number: u32,
}

impl Header {
    pub fn new(stream_name: &str) -> Self {
        let mut stream_name_bytes = [0u8; 16];
        stream_name_bytes[..stream_name.len()].copy_from_slice(stream_name.as_bytes());

        Self {
            sample_rate: SampleRate::Hz48000,
            num_channels: 2,
            stream_name: stream_name_bytes,
            frame_number: 0,
        }
    }

    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    pub fn stream_name(&self) -> String {
        String::from_utf8_lossy(&self.stream_name).replace("\0", "")
    }

    pub fn num_channels(&self) -> u8 {
        self.num_channels
    }

    pub fn frame_number(&self) -> u32 {
        self.frame_number
    }

    pub fn set_frame_number(&mut self, frame_number: u32) {
        self.frame_number = frame_number;
    }
}

impl TryFrom<&[u8]> for Header {
    type Error = Error;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let sr_sp = data[0];
        let sample_rate = SampleRate::from_u8(sr_sp & SAMPLE_RATE_MASK).unwrap();
        let channels = data[1];
        let mut stream_name: [u8; 16] = [0; 16];
        stream_name.copy_from_slice(&data[2..18]);
        let frame_number = LittleEndian::read_u32(&data[18..22]);
        Ok(Self {
            sample_rate,
            num_channels: channels + 1,
            stream_name,
            frame_number,
        })
    }
}

impl From<Header> for [u8; HEADER_SIZE] {
    fn from(header: Header) -> [u8; HEADER_SIZE] {
        let mut result = [0; HEADER_SIZE];
        // Magic number
        result[0] = header.sample_rate.to_u8().unwrap();
        result[1] = header.num_channels - 1;
        for i in 0..16 {
            result[2 + i] = header.stream_name[i];
        }
        LittleEndian::write_u32(&mut result[18..22], header.frame_number);

        result
    }
}

#[derive(Clone, Copy, FromPrimitive, ToPrimitive, Debug)]
pub enum SampleRate {
    Hz6000 = 0,
    Hz12000,
    Hz24000,
    Hz48000,
    Hz96000,
    Hz192000,
    Hz384000,
    Hz8000,
    Hz16000,
    Hz32000,
    Hz64000,
    Hz128000,
    Hz256000,
    Hz512000,
    Hz11025,
    Hz22050,
    Hz44100,
    Hz88200,
    Hz176400,
    Hz352800,
    Hz705600,
}
