use anyhow::{anyhow, Result};

use std::sync::Arc;

use cpal::{
    traits::{DeviceTrait, StreamTrait},
    FromSample, Sample, SampleFormat, SizedSample, SupportedStreamConfig,
};

use crate::utils;
use crate::utils::cpal::{Device, Host};

use super::ring_buffer::VbanStreamConsumer;

pub struct VbanReceptorStream {
    host: cpal::Host,
    device: Arc<cpal::Device>,
    stream: Option<cpal::Stream>,
}

impl VbanReceptorStream {
    pub fn new(device_name: &str, device_type: &str, host_name: &str) -> Result<Self> {
        let host = utils::cpal::host_by_name(host_name)?;
        let device = Arc::new(match device_type {
            "input" => host
                .find_input_device(device_name)
                .ok_or(anyhow!("no input device available"))?,
            "output" => host
                .find_output_device(device_name)
                .ok_or(anyhow!("no input device available"))?,
            _ => return Err(anyhow!("invalid device type")),
        });

        Ok(Self {
            host,
            device,
            stream: None,
        })
    }

    pub fn setup_stream(&mut self, consumer: VbanStreamConsumer) -> Result<()> {
        let sample_format = self.device_config()?.sample_format();
        self.stream = Some(self.build_stream_for_sample_format(sample_format, consumer)?);

        Ok(())
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
        let error_fn = || anyhow!("you first need to call .setup_stream in the VbanReceptorStream");

        Ok(self.stream.as_ref().ok_or_else(error_fn)?)
    }

    pub fn device_config(&self) -> Result<SupportedStreamConfig> {
        Ok(self.device.default_output_config()?)
    }

    pub fn should_run(&self, device_name: &str) -> bool {
        if device_name == "default" && !self.device.is_default_output(&self.host) {
            self.pause().ok();

            return false;
        }

        true
    }

    fn build_stream_for_sample_format(
        &mut self,
        sample_format: SampleFormat,
        consumer: VbanStreamConsumer,
    ) -> Result<cpal::Stream> {
        match sample_format {
            SampleFormat::I8 => self.build_stream::<i8>(consumer),
            SampleFormat::I16 => self.build_stream::<i16>(consumer),
            SampleFormat::I32 => self.build_stream::<i32>(consumer),
            SampleFormat::I64 => self.build_stream::<i64>(consumer),
            SampleFormat::U8 => self.build_stream::<u8>(consumer),
            SampleFormat::U16 => self.build_stream::<u16>(consumer),
            SampleFormat::U32 => self.build_stream::<u32>(consumer),
            SampleFormat::U64 => self.build_stream::<u64>(consumer),
            SampleFormat::F32 => self.build_stream::<f32>(consumer),
            SampleFormat::F64 => self.build_stream::<f64>(consumer),
            _ => unreachable!("Unsupported sample format: {:?}", sample_format),
        }
    }

    fn build_stream<T>(&mut self, consumer: VbanStreamConsumer) -> Result<cpal::Stream>
    where
        T: SizedSample + FromSample<i16> + FromSample<f32> + Send + Sync,
    {
        let config = self.device.default_output_config()?;
        let channels = config.channels() as usize;

        let stream = self.device.build_output_stream(
            &config.into(),
            Self::build_data_callback::<T>(consumer, channels),
            move |err| {
                eprintln!("an error occurred on stream: {}", err);
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
}
