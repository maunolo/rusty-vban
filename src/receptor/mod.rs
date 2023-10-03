mod socket;
mod stream;

use anyhow::{Context, Result};

use self::{
    socket::{VbanReceptorSocket, VbanReceptorSocketBuilder},
    stream::{VbanReceptorStream, VbanReceptorStreamBuilder},
};

pub struct ReceptorBuilder {
    latency: u32,
    stream_name: Option<String>,
    channels: u8,
    ip_address: Option<String>,
    port: u16,
    device: String,
    device_type: String,
    backend: String,
}

pub struct ReceptorParams {
    latency: u32,
    stream_name: String,
    channels: u8,
    ip_address: String,
    port: u16,
    device: String,
    device_type: String,
    backend: String,
}

impl ReceptorBuilder {
    pub fn default() -> Self {
        Self {
            latency: 16,
            stream_name: None,
            channels: 2,
            ip_address: None,
            port: 6980,
            device: "default".to_string(),
            device_type: "output".to_string(),
            backend: "default".to_string(),
        }
    }

    pub fn latency(mut self, latency: u32) -> Self {
        self.latency = latency;
        self
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

    pub fn build(self) -> Result<Receptor> {
        let latency = self.latency;
        let stream_name = self.stream_name.context("Stream name is required")?;
        let channels = self.channels;
        let ip_address = self.ip_address.context("IP address is required")?;
        let port = self.port;
        let device = self.device;
        let device_type = self.device_type;
        let backend = self.backend;

        let (stream, producer) = VbanReceptorStreamBuilder::default()
            .device_name(&device)
            .device_type(&device_type)
            .host_name(&backend)
            .latency(latency as f32)
            .build()?;

        let socket = VbanReceptorSocketBuilder::default()
            .port(self.port)
            .incoming_addr(&ip_address)
            .incoming_stream_name(&stream_name)
            .channels(self.channels)
            .producer(producer)
            .build()?;

        let params = ReceptorParams {
            latency,
            stream_name,
            channels,
            ip_address,
            port,
            device,
            device_type,
            backend,
        };

        Ok(Receptor {
            stream,
            socket,
            params,
        })
    }
}

pub struct Receptor {
    stream: VbanReceptorStream,
    socket: VbanReceptorSocket,
    params: ReceptorParams,
}

pub struct ReceptorOptions {
    pub retry: bool,
}

impl Default for ReceptorOptions {
    fn default() -> Self {
        Self { retry: false }
    }
}

impl Receptor {
    pub fn run(mut self, options: ReceptorOptions) -> Result<Self> {
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

    pub fn play(&mut self) -> Result<()> {
        self.stream.play()?;
        self.socket.start()
    }

    pub fn pause(&mut self) -> Result<()> {
        self.stream.pause()?;
        self.socket.stop()
    }

    pub fn rebuild(self) -> Result<Self> {
        let Self {
            params,
            stream,
            socket,
        } = self;

        drop(socket);
        drop(stream);

        let (stream, producer) = VbanReceptorStreamBuilder::default()
            .device_name(&params.device)
            .device_type(&params.device_type)
            .host_name(&params.backend)
            .latency(params.latency as f32)
            .build()?;

        let socket = VbanReceptorSocketBuilder::default()
            .port(params.port)
            .incoming_addr(&params.ip_address)
            .incoming_stream_name(&params.stream_name)
            .channels(params.channels)
            .producer(producer)
            .build()?;

        Ok(Self {
            stream,
            socket,
            params,
        })
    }
}
