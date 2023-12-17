mod stream;

use anyhow::{Context, Result};

use crate::utils::errors::RustyVbanError;

use self::stream::{VbanEmitterStream, VbanEmitterStreamBuilder};

pub struct EmitterBuilder {
    stream_name: Option<String>,
    channels: u8,
    ip_address: Option<String>,
    port: u16,
    device: String,
    device_type: String,
    backend: String,
}

#[allow(dead_code)]
struct EmitterParams {
    stream_name: String,
    channels: u8,
    ip_address: String,
    port: u16,
    device: String,
    device_type: String,
    backend: String,
}

impl EmitterBuilder {
    pub fn default() -> Self {
        Self {
            stream_name: None,
            channels: 2,
            ip_address: None,
            port: 6980,
            device: "default".to_string(),
            device_type: "input".to_string(),
            backend: "default".to_string(),
        }
    }

    pub fn stream_name<T: Into<String>>(mut self, stream_name: T) -> Self {
        self.stream_name = Some(stream_name.into());
        self
    }

    pub fn channels(mut self, channels: u8) -> Self {
        self.channels = channels;
        self
    }

    pub fn ip_address<T: Into<String>>(mut self, ip_address: T) -> Self {
        self.ip_address = Some(ip_address.into());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn device<T: Into<String>>(mut self, device: T) -> Self {
        self.device = device.into();
        self
    }

    pub fn device_type<T: Into<String>>(mut self, device_type: T) -> Self {
        self.device_type = device_type.into();
        self
    }

    pub fn backend<T: Into<String>>(mut self, backend: T) -> Self {
        self.backend = backend.into();
        self
    }

    pub fn build(self) -> Result<Emitter> {
        let stream_name = self.stream_name.context("Stream name is required")?;
        let ip_address = self.ip_address.context("IP address is required")?;

        let stream = VbanEmitterStreamBuilder::default()
            .device_name(&self.device)
            .device_type(&self.device_type)
            .host_name(&self.backend)
            .ip_address(&ip_address)
            .port(self.port)
            .stream_name(&stream_name)
            .build()?;

        let params = EmitterParams {
            stream_name,
            channels: self.channels,
            ip_address,
            port: self.port,
            device: self.device,
            device_type: self.device_type,
            backend: self.backend,
        };

        Ok(Emitter { stream, params })
    }
}

pub struct Emitter {
    stream: VbanEmitterStream,
    params: EmitterParams,
}

pub struct EmitterOptions {
    pub retry: bool,
}

impl Default for EmitterOptions {
    fn default() -> Self {
        Self { retry: false }
    }
}

impl Emitter {
    pub fn run(mut self, options: EmitterOptions) -> Result<Self, RustyVbanError> {
        self.play()?;

        while self.stream.should_run(&self.params.device) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        Ok(if options.retry {
            self.rebuild()?.run(options)?
        } else {
            self
        })
    }

    pub fn play(&mut self) -> Result<(), RustyVbanError> {
        self.stream.play()?;

        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), RustyVbanError> {
        self.stream.pause()?;

        Ok(())
    }

    pub fn should_run(&self) -> bool {
        self.stream.should_run(&self.params.device)
    }

    pub fn rebuild(self) -> Result<Self> {
        let Self { stream, params } = self;

        drop(stream);

        let stream = VbanEmitterStreamBuilder::default()
            .device_name(&params.device)
            .device_type(&params.device_type)
            .host_name(&params.backend)
            .ip_address(&params.ip_address)
            .port(params.port)
            .stream_name(&params.stream_name)
            .build()?;

        Ok(Self { stream, params })
    }
}
