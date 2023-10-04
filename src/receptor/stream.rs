use anyhow::{anyhow, Context, Result};

use std::{
    mem::MaybeUninit,
    sync::{Arc, Mutex},
};

use cpal::{
    traits::{DeviceTrait, StreamTrait},
    FromSample, Sample, SampleFormat, SizedSample, SupportedStreamConfig,
};

use ringbuf::{Consumer, HeapRb, Producer, SharedRb};

use crate::utils::cpal::{Device, Host};
use crate::utils::{self, log};

pub type VbanStreamConsumer = Consumer<i16, Arc<SharedRb<i16, Vec<MaybeUninit<i16>>>>>;
pub type VbanStreamProducer = Producer<i16, Arc<SharedRb<i16, Vec<MaybeUninit<i16>>>>>;

pub struct StreamWrapper(Arc<Mutex<cpal::Stream>>);

unsafe impl Send for StreamWrapper {}
unsafe impl Sync for StreamWrapper {}

fn start_ring_buffer(
    latency: f32,
    config: &SupportedStreamConfig,
) -> (VbanStreamProducer, VbanStreamConsumer) {
    let latency_frames = (latency / 1_000.0) * config.sample_rate().0 as f32;
    let latency_samples = latency_frames as usize * config.channels() as usize;

    let ring = HeapRb::<i16>::new(latency_samples * 2);
    let (mut producer, consumer) = ring.split();

    // Fill the samples with 0.0 equal to the length of the delay.
    for _ in 0..latency_samples {
        // The ring buffer has twice as much space as necessary to add latency here,
        // so this should never fail
        producer.push(0.0.to_sample::<i16>()).ok();
    }

    (producer, consumer)
}

pub struct VbanReceptorStreamBuilder {
    device_name: Option<String>,
    device_type: Option<String>,
    host_name: Option<String>,
    latency: Option<f32>,
}

impl Default for VbanReceptorStreamBuilder {
    fn default() -> Self {
        Self {
            device_name: None,
            device_type: None,
            host_name: None,
            latency: None,
        }
    }
}

impl VbanReceptorStreamBuilder {
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

    pub fn latency(mut self, latency: f32) -> Self {
        self.latency = Some(latency);
        self
    }

    pub fn build(self) -> Result<(VbanReceptorStream, VbanStreamProducer)> {
        let device_name = self.device_name.context("device name is required")?;
        let device_type = self.device_type.context("device type is required")?;
        let host_name = self.host_name.context("host name is required")?;
        let latency = self.latency.context("latency is required")?;

        let host = Arc::new(utils::cpal::host_by_name(&host_name)?);
        let device = Arc::new(match device_type.as_str() {
            "input" => host
                .find_input_device(&device_name)
                .context("no input device available")?,
            "output" => host
                .find_output_device(&device_name)
                .context("no input device available")?,
            _ => return Err(anyhow!("invalid device type")),
        });
        let device_config = device.default_output_config()?;
        let sample_format = device_config.sample_format();
        let (producer, consumer) = start_ring_buffer(latency, &device_config);
        let stream = StreamWrapper(Arc::new(Mutex::new(build_stream_for_sample_format(
            sample_format,
            StreamParams {
                device: device.clone(),
                consumer,
            },
        )?)));

        Ok((
            VbanReceptorStream {
                host,
                device,
                stream,
            },
            producer,
        ))
    }
}

pub struct VbanReceptorStream {
    host: Arc<cpal::Host>,
    device: Arc<cpal::Device>,
    stream: StreamWrapper,
}

impl VbanReceptorStream {
    pub fn play(&self) -> Result<()> {
        self.stream.0.lock().unwrap().play()?;

        Ok(())
    }

    pub fn pause(&self) -> Result<()> {
        self.stream.0.lock().unwrap().pause()?;

        Ok(())
    }

    pub fn should_run(&self, device_name: &str) -> bool {
        if device_name == "default" && !self.device.is_default_output(&self.host) {
            self.pause().ok();

            return false;
        }

        true
    }
}

struct StreamParams {
    device: Arc<cpal::Device>,
    consumer: VbanStreamConsumer,
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
    T: SizedSample + FromSample<i16> + FromSample<f32> + Send + Sync,
{
    let StreamParams { device, consumer } = params;
    let config = device.default_output_config()?;
    let channels = config.channels() as usize;

    let stream = device.build_output_stream(
        &config.into(),
        build_data_callback::<T>(consumer, channels),
        move |err| {
            log::error(&format!("an error occurred on stream: {}", err));
        },
        None,
    )?;

    Ok(stream)
}

fn build_data_callback<T>(
    mut consumer: VbanStreamConsumer,
    channels: usize,
) -> impl FnMut(&mut [T], &cpal::OutputCallbackInfo) + Send + 'static
where
    T: SizedSample + FromSample<i16> + FromSample<f32> + Send + Sync,
{
    move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
        for frame in data.chunks_mut(channels) {
            for sample in frame {
                *sample = match consumer.pop() {
                    Some(s) => s.to_sample::<T>(),
                    None => 0.0.to_sample::<T>(),
                };
            }
        }
    }
}
