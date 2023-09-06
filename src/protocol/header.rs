use std::convert::TryFrom;

use byteorder::{ByteOrder, LittleEndian};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

use super::packet::Error;

pub const MAX_NUM_SAMPLES: usize = 256;
const SAMPLE_RATE_MASK: u8 = 0b00011111;
const SUB_PROTOCOL_MASK: u8 = 0b11100000;
const RESERVED_MASK: u8 = 0b00001000;
const BIT_RESOLUTION_MASK: u8 = 0b00000111;
const CODEC_MASK: u8 = 0b11110000;
pub const HEADER_SIZE: usize = 28;

#[derive(Copy, Clone, Debug)]
pub struct Header {
    sample_rate: SampleRate,
    sub_protocol: SubProtocol,
    num_samples: u8,
    num_channels: u8,
    bit_resolution: BitResolution,
    codec: Codec,
    stream_name: [u8; 16],
    frame_number: u32,
}

impl Header {
    pub fn new(stream_name: &str) -> Self {
        let mut stream_name_bytes = [0u8; 16];
        stream_name_bytes[..stream_name.len()].copy_from_slice(stream_name.as_bytes());

        Self {
            sample_rate: SampleRate::Hz48000,
            sub_protocol: SubProtocol::Audio,
            num_samples: MAX_NUM_SAMPLES as u8,
            num_channels: 2,
            bit_resolution: BitResolution::Signed16Bit,
            codec: Codec::PCM,
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

    pub fn sub_protocol(&self) -> SubProtocol {
        self.sub_protocol
    }

    pub fn num_samples(&self) -> u8 {
        self.num_samples
    }

    pub fn set_num_samples(&mut self, num_samples: u8) {
        self.num_samples = num_samples;
    }

    pub fn num_channels(&self) -> u8 {
        self.num_channels
    }

    pub fn bit_resolution(&self) -> BitResolution {
        self.bit_resolution
    }

    pub fn codec(&self) -> Codec {
        self.codec
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
        if &data[0..4] != "VBAN".as_bytes() {
            return Err(Error::MissingMagicNumber);
        }
        let sr_sp = data[4];
        let sample_rate = SampleRate::from_u8(sr_sp & SAMPLE_RATE_MASK).unwrap();
        let sub_protocol = SubProtocol::from_u8(sr_sp & SUB_PROTOCOL_MASK).unwrap();
        let samples_per_frame = data[5];
        let channels = data[6];
        let format_codec = data[7];
        if (format_codec & RESERVED_MASK) != 0 {
            return Err(Error::MalformedFormat);
        }
        let bit_resolution = BitResolution::from_u8(format_codec & BIT_RESOLUTION_MASK).unwrap();
        let codec = Codec::from_u8(format_codec & CODEC_MASK).unwrap();
        let mut stream_name: [u8; 16] = [0; 16];
        stream_name.copy_from_slice(&data[8..24]);
        let frame_number = LittleEndian::read_u32(&data[24..28]);
        Ok(Self {
            sample_rate,
            sub_protocol,
            num_samples: samples_per_frame,
            num_channels: channels + 1,
            bit_resolution,
            codec,
            stream_name,
            frame_number,
        })
    }
}

impl From<Header> for [u8; HEADER_SIZE] {
    fn from(header: Header) -> [u8; HEADER_SIZE] {
        let mut result = [0; HEADER_SIZE];
        // Magic number
        'V'.encode_utf8(&mut result[0..]);
        'B'.encode_utf8(&mut result[1..]);
        'A'.encode_utf8(&mut result[2..]);
        'N'.encode_utf8(&mut result[3..]);
        result[4] = header.sample_rate.to_u8().unwrap();
        result[5] = header.num_samples;
        result[6] = header.num_channels - 1;
        result[7] = header.bit_resolution.to_u8().unwrap() + header.codec.to_u8().unwrap();
        for i in 0..16 {
            result[8 + i] = header.stream_name[i];
        }
        LittleEndian::write_u32(&mut result[24..28], header.frame_number);

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

#[derive(Clone, Copy, FromPrimitive, Debug)]
pub enum SubProtocol {
    Audio = 0x00,
    Serial = 0x20,
    Text = 0x40,
    Service = 0x60,
    Undefined1 = 0x80,
    Undefined2 = 0xa0,
    Undefined3 = 0xc0,
    User = 0xe0,
}

#[derive(Clone, Copy, ToPrimitive, FromPrimitive, Debug)]
pub enum BitResolution {
    Unsigned8Bit = 0,
    Signed16Bit,
    Signed24Bit,
    Signed32Bit,
    Float32Bit,
    Float64Bit,
    Signed12Bit,
    Signed10Bit,
}

#[derive(Clone, Copy, ToPrimitive, FromPrimitive, Debug)]
pub enum Codec {
    PCM = 0x00,
    VBCA = 0x10,
    VBCV = 0x20,
    Undefined1 = 0x30,
    Undefined2 = 0x40,
    Undefined3 = 0x50,
    Undefined4 = 0x60,
    Undefined5 = 0x70,
    Undefined6 = 0x80,
    Undefined7 = 0x90,
    Undefined8 = 0xa0,
    Undefined9 = 0xb0,
    Undefined10 = 0xc0,
    Undefined11 = 0xd0,
    Undefined12 = 0xe0,
    User = 0xf0,
}
