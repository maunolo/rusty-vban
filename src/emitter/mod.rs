mod stream;

use anyhow::Result;
use derive_builder::Builder;

use self::stream::VbanEmitterStream;

#[derive(Builder, Debug)]
#[builder(setter(into))]
pub struct Emitter {
    pub stream_name: String,
    #[builder(default = "2")]
    pub channels: u8,
    pub ip_address: String,
    #[builder(default = "6980")]
    pub port: u16,
    #[builder(default = "\"default\".to_string()")]
    pub device: String,
    #[builder(default = "\"input\".to_string()")]
    device_type: String,
}

impl Emitter {
    pub fn start(&self) -> Result<()> {
        let mut stream = VbanEmitterStream::new(
            &self.device,
            &self.device_type,
            &self.stream_name,
            &self.ip_address,
            self.port,
        )?;
        stream.setup_stream()?;
        stream.play()?;

        while stream.should_run(&self.device) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        stream.pause()?;
        drop(stream);

        self.start()?;

        Ok(())
    }
}
