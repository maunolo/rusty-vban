use anyhow::{anyhow, Result};
use dasp_sample::ToSample;

use std::{
    net::{SocketAddr, UdpSocket},
    sync::Arc,
};

use cpal::{
    traits::{DeviceTrait, StreamTrait},
    Sample, SampleFormat, SizedSample, SupportedStreamConfig,
};

use crate::protocol::header::{Header, MAX_NUM_SAMPLES};
use crate::utils::cpal::{Device, Host};

pub struct VbanEmitterStream {
    host: cpal::Host,
    device: Arc<cpal::Device>,
    stream: Option<cpal::Stream>,
    bind_address_pool: Vec<SocketAddr>,
    header: Header,
    target_address: SocketAddr,
}

impl VbanEmitterStream {
    pub fn new(
        device_name: &str,
        device_type: &str,
        stream_name: &str,
        ip_address: &str,
        port: u16,
    ) -> Result<Self> {
        let host = cpal::default_host();
        let device = Arc::new(match device_type {
            "input" => host
                .find_input_device(device_name)
                .ok_or(anyhow!("no input device available"))?,
            "output" => host
                .find_output_device(device_name)
                .ok_or(anyhow!("no input device available"))?,
            _ => return Err(anyhow!("invalid device type")),
        });
        let addrs = (1..=10)
            .map(|i| SocketAddr::from(([0, 0, 0, 0], port + i)))
            .collect::<Vec<SocketAddr>>();

        let header = Header::new(stream_name);
        let target_address = SocketAddr::new(ip_address.parse()?, port);

        Ok(Self {
            host,
            device,
            stream: None,
            bind_address_pool: addrs,
            header,
            target_address,
        })
    }

    pub fn setup_stream(&mut self) -> Result<()> {
        let sample_format = self.device_config()?.sample_format();
        self.stream = Some(self.build_stream_for_sample_format(sample_format)?);

        Ok(())
    }

    pub fn device_config(&self) -> Result<SupportedStreamConfig> {
        Ok(self.device.default_config()?)
    }

    pub fn play(&self) -> Result<()> {
        self.stream()?.play()?;

        Ok(())
    }

    pub fn pause(&self) -> Result<()> {
        self.stream()?.pause()?;

        Ok(())
    }

    pub fn stream(&self) -> Result<&cpal::Stream> {
        let error_fn = || anyhow!("you first need to call .setup_stream in the VbanEmitterStream");

        Ok(self.stream.as_ref().ok_or_else(error_fn)?)
    }

    pub fn should_run(&self, device_name: &str) -> bool {
        if device_name == "default" && !self.device.is_default_input(&self.host) {
            return false;
        }

        true
    }

    fn build_stream_for_sample_format(&self, sample_format: SampleFormat) -> Result<cpal::Stream> {
        match sample_format {
            SampleFormat::I8 => self.build_stream::<i8>(),
            SampleFormat::I16 => self.build_stream::<i16>(),
            SampleFormat::I32 => self.build_stream::<i32>(),
            SampleFormat::I64 => self.build_stream::<i64>(),
            SampleFormat::U8 => self.build_stream::<u8>(),
            SampleFormat::U16 => self.build_stream::<u16>(),
            SampleFormat::U32 => self.build_stream::<u32>(),
            SampleFormat::U64 => self.build_stream::<u64>(),
            SampleFormat::F32 => self.build_stream::<f32>(),
            SampleFormat::F64 => self.build_stream::<f64>(),
            _ => unreachable!("Unsupported sample format: {:?}", sample_format),
        }
    }

    fn build_stream<T>(&self) -> Result<cpal::Stream>
    where
        T: SizedSample + ToSample<i16> + Send + 'static,
    {
        let err_fn = move |error| eprintln!("an error occurred on stream: {}", error);
        let mut frame_count = 0;

        let header = self.header.clone();
        let socket = UdpSocket::bind(&self.bind_address_pool[..])?;
        let target_address = self.target_address.clone();

        let stream = self.device.build_input_stream(
            &self.device_config()?.into(),
            move |data: &[T], _: &_| {
                write_data::<T>(data, header, &socket, &target_address, &mut frame_count)
            },
            err_fn,
            None,
        )?;

        Ok(stream)
    }
}

fn write_data<T>(
    input: &[T],
    mut header: Header,
    socket: &UdpSocket,
    addr: &SocketAddr,
    frame_count: &mut u32,
) where
    T: Sample + ToSample<i16>,
{
    let total_samples = input.len() as usize;
    let max = MAX_NUM_SAMPLES as f32;
    let chunks_amount = (total_samples as f32 / max).ceil() as usize;
    let chunk_num_samples = total_samples / chunks_amount;

    for samples in input.chunks_exact(chunk_num_samples) {
        let mut buffer = Vec::new();

        header.set_num_samples(samples.len() as u8 / header.num_channels() - 1);
        header.set_frame_number(*frame_count);
        let header: [u8; 28] = header.into();
        let data = samples
            .iter()
            .flat_map(|s| s.to_sample::<i16>().to_le_bytes())
            .collect::<Vec<u8>>();

        buffer.extend_from_slice(&header);
        buffer.extend_from_slice(&data);
        if let Err(e) = socket.send_to(&buffer[..buffer.len()], addr) {
            eprintln!("error sending data: {}", e);
        }

        *frame_count += 1;
    }
}
