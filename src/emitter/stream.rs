use anyhow::{anyhow, Context, Result};
use dasp_sample::ToSample;

use std::{
    net::{SocketAddr, UdpSocket},
    sync::Arc,
};

use cpal::{
    traits::{DeviceTrait, StreamTrait},
    Sample, SampleFormat, SizedSample,
};

use crate::protocol::header::{Header, MAX_NUM_SAMPLES};
use crate::utils;
use crate::utils::cpal::{Device, Host};
use crate::utils::log;

pub struct VbanEmitterStreamBuilder {
    device_name: Option<String>,
    device_type: Option<String>,
    host_name: Option<String>,
    ip_address: Option<String>,
    port: Option<u16>,
    stream_name: Option<String>,
}

impl Default for VbanEmitterStreamBuilder {
    fn default() -> Self {
        Self {
            device_name: None,
            device_type: None,
            host_name: None,
            ip_address: None,
            port: None,
            stream_name: None,
        }
    }
}

impl VbanEmitterStreamBuilder {
    pub fn device_name(mut self, device_name: &str) -> Self {
        self.device_name = Some(device_name.to_string());
        self
    }

    pub fn device_type(mut self, device_type: &str) -> Self {
        self.device_type = Some(device_type.to_string());
        self
    }

    pub fn host_name(mut self, host_name: &str) -> Self {
        self.host_name = Some(host_name.to_string());
        self
    }

    pub fn ip_address(mut self, ip_address: &str) -> Self {
        self.ip_address = Some(ip_address.to_string());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn stream_name(mut self, stream_name: &str) -> Self {
        self.stream_name = Some(stream_name.to_string());
        self
    }

    pub fn build(self) -> Result<VbanEmitterStream> {
        let device_name = self.device_name.context("device name is required")?;
        let device_type = self.device_type.context("device type is required")?;
        let host_name = self.host_name.context("host name is required")?;
        let ip_address = self.ip_address.context("ip address is required")?;
        let port = self.port.context("port is required")?;
        let stream_name = self.stream_name.context("stream name is required")?;

        let host = utils::cpal::host_by_name(&host_name)?;
        let device = Arc::new(match device_type.as_str() {
            "input" => host
                .find_input_device(&device_name)
                .ok_or(anyhow!("no input device available"))?,
            "output" => host
                .find_output_device(&device_name)
                .ok_or(anyhow!("no input device available"))?,
            _ => return Err(anyhow!("invalid device type")),
        });
        let addrs = (1..=10)
            .map(|i| SocketAddr::from(([0, 0, 0, 0], port + i)))
            .collect::<Vec<SocketAddr>>();
        let header = Header::new(&stream_name);
        let target = SocketAddr::new(ip_address.parse()?, port);

        let stream = build_stream_for_sample_format(
            device.default_input_config()?.sample_format(),
            StreamParams {
                device: device.clone(),
                header,
                addrs,
                target,
            },
        )?;

        Ok(VbanEmitterStream {
            host,
            device,
            stream,
        })
    }
}

pub struct VbanEmitterStream {
    host: cpal::Host,
    device: Arc<cpal::Device>,
    stream: cpal::Stream,
}

impl VbanEmitterStream {
    pub fn play(&self) -> Result<()> {
        self.stream().play()?;

        Ok(())
    }

    pub fn pause(&self) -> Result<()> {
        self.stream().pause()?;

        Ok(())
    }

    pub fn stream(&self) -> &cpal::Stream {
        &self.stream
    }

    pub fn should_run(&self, device_name: &str) -> bool {
        if device_name == "default" && !self.device.is_default_input(&self.host) {
            return false;
        }

        true
    }
}

struct StreamParams {
    device: Arc<cpal::Device>,
    header: Header,
    addrs: Vec<SocketAddr>,
    target: SocketAddr,
}

fn build_stream_for_sample_format(
    sample_format: SampleFormat,
    params: StreamParams,
) -> Result<cpal::Stream> {
    match sample_format {
        SampleFormat::I8 => build_stream::<i8>(params),
        SampleFormat::I16 => build_stream::<i16>(params),
        SampleFormat::I32 => build_stream::<i32>(params),
        SampleFormat::I64 => build_stream::<i64>(params),
        SampleFormat::U8 => build_stream::<u8>(params),
        SampleFormat::U16 => build_stream::<u16>(params),
        SampleFormat::U32 => build_stream::<u32>(params),
        SampleFormat::U64 => build_stream::<u64>(params),
        SampleFormat::F32 => build_stream::<f32>(params),
        SampleFormat::F64 => build_stream::<f64>(params),
        _ => unreachable!("Unsupported sample format: {:?}", sample_format),
    }
}

fn build_stream<T>(params: StreamParams) -> Result<cpal::Stream>
where
    T: SizedSample + ToSample<i16> + Send + 'static,
{
    let StreamParams {
        device,
        header,
        addrs,
        target,
    } = params;
    let config = device.default_input_config()?;
    let err_fn = move |error| log::error(&format!("an error occurred on stream: {}", error));
    let mut frame_count = 0;

    let socket = UdpSocket::bind(&addrs[..])?;

    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[T], _: &_| write_data::<T>(data, header, &socket, &target, &mut frame_count),
        err_fn,
        None,
    )?;

    Ok(stream)
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
    let total_samples = input.len();
    let mut chunks_amount = total_samples / MAX_NUM_SAMPLES;
    if total_samples % MAX_NUM_SAMPLES > 0 {
        chunks_amount += 1;
    }
    let chunk_num_samples = total_samples / chunks_amount;

    for samples in input.chunks(chunk_num_samples) {
        let mut buffer = Vec::new();

        header.set_num_samples((samples.len() / header.num_channels() as usize - 1) as u8);
        header.set_frame_number(*frame_count);
        let header: [u8; 28] = header.into();
        let data = samples
            .iter()
            .flat_map(|s| s.to_sample::<i16>().to_le_bytes())
            .collect::<Vec<u8>>();

        buffer.extend_from_slice(&header);
        buffer.extend_from_slice(&data);
        if let Err(e) = socket.send_to(&buffer[..buffer.len()], addr) {
            log::error(&format!("error sending data: {}", e));
        }

        *frame_count += 1;
    }
}
