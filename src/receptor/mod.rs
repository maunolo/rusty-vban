mod ring_buffer;
mod socket;
mod stream;

use anyhow::Result;
use derive_builder::Builder;

use self::{socket::VbanReceptorSocket, stream::VbanReceptorStream};

#[derive(Builder)]
#[builder(setter(into))]
pub struct Receptor {
    #[builder(default = "16")]
    latency: u32,
    stream_name: String,
    #[builder(default = "2")]
    channels: u8,
    ip_address: String,
    #[builder(default = "6980")]
    port: u16,
    #[builder(default = "\"default\".to_string()")]
    device: String,
    #[builder(default = "\"output\".to_string()")]
    device_type: String,
    #[builder(default = "\"default\".to_string()")]
    backend: String,
}

impl Receptor {
    pub fn start(&self) -> Result<()> {
        let mut stream = VbanReceptorStream::new(&self.device, &self.device_type, &self.backend)?;

        let (producer, consumer) =
            ring_buffer::start_buffer(self.latency as f32, &stream.device_config()?);

        stream.setup_stream(consumer)?;
        stream.play()?;

        let socket = VbanReceptorSocket::new(self.port)?;
        socket.start_receive_loop(
            &self.ip_address,
            &self.stream_name,
            self.channels,
            producer,
            || stream.should_run(&self.device),
        );

        stream.pause()?;
        drop(stream);

        self.start()?;

        Ok(())
    }
}
