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

    pub fn stream_name(mut self, stream_name: &str) -> Self {
        self.stream_name = Some(stream_name.to_string());
        self
    }

    pub fn channels(mut self, channels: u8) -> Self {
        self.channels = channels;
        self
    }

    pub fn ip_address(mut self, ip_address: &str) -> Self {
        self.ip_address = Some(ip_address.to_string());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn device(mut self, device: &str) -> Self {
        self.device = device.to_string();
        self
    }

    pub fn device_type(mut self, device_type: &str) -> Self {
        self.device_type = device_type.to_string();
        self
    }

    pub fn backend(mut self, backend: &str) -> Self {
        self.backend = backend.to_string();
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
    pub fn run(&mut self, options: ReceptorOptions) -> Result<()> {
        self.play()?;

        while self.stream.should_run(&self.params.device) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        if options.retry {
            self.rebuild()?;
            self.run(options)?;
        }

        Ok(())
    }

    pub fn play(&mut self) -> Result<()> {
        self.stream.play()?;
        self.socket.start()
    }

    pub fn pause(&mut self) -> Result<()> {
        self.stream.pause()?;
        self.socket.stop()
    }

    pub fn rebuild(&mut self) -> Result<()> {
        let _ = self.pause();

        let (stream, producer) = VbanReceptorStreamBuilder::default()
            .device_name(&self.params.device)
            .device_type(&self.params.device_type)
            .host_name(&self.params.backend)
            .latency(self.params.latency as f32)
            .build()?;

        let socket = VbanReceptorSocketBuilder::default()
            .port(self.params.port)
            .incoming_addr(&self.params.ip_address)
            .incoming_stream_name(&self.params.stream_name)
            .channels(self.params.channels)
            .producer(producer)
            .build()?;

        self.stream = stream;
        self.socket = socket;

        Ok(())
    }
}
