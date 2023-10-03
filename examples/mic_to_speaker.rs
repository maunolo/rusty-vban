use anyhow::Result;
use local_ip_address::local_ip;
use rusty_vban::{
    emitter::{self, EmitterOptions},
    receptor::{self, ReceptorOptions},
};

fn main() -> Result<()> {
    std::thread::spawn(move || -> Result<()> {
        let _ = emitter::EmitterBuilder::default()
            .ip_address(local_ip().unwrap().to_string())
            .port(9000)
            .stream_name("Mic")
            .build()?
            .run(EmitterOptions { retry: true });

        Ok(())
    });

    let _ = receptor::ReceptorBuilder::default()
        .ip_address(local_ip().unwrap().to_string())
        .port(9000)
        .stream_name("Mic")
        .build()?
        .run(ReceptorOptions { retry: true });

    Ok(())
}
