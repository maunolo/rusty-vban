use anyhow::Result;
use local_ip_address::local_ip;
use rusty_vban::{emitter, receptor};

fn main() -> Result<()> {
    std::thread::spawn(move || -> Result<()> {
        emitter::EmitterBuilder::default()
            .ip_address(&local_ip().unwrap().to_string())
            .stream_name("Mic")
            .build()?
            .start()?;

        Ok(())
    });

    receptor::ReceptorBuilder::default()
        .ip_address(&local_ip().unwrap().to_string())
        .stream_name("Mic")
        .build()?
        .start()?;

    Ok(())
}
