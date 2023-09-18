use anyhow::Result;
use local_ip_address::local_ip;
use rusty_vban::{emitter, receptor, utils::log};

fn main() -> Result<()> {
    std::thread::spawn(move || -> Result<()> {
        let mut emitter = emitter::EmitterBuilder::default()
            .ip_address(local_ip().unwrap().to_string())
            .port(9000)
            .stream_name("Mic")
            .build()?;

        log::info("Emitter Playing");

        emitter.play()?;

        std::thread::sleep(std::time::Duration::from_secs(5));

        log::info("Emitter Pausing");

        emitter.pause()?;

        std::thread::sleep(std::time::Duration::from_secs(5));

        log::info("Emitter Playing");

        emitter.play()?;

        loop {
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    });

    let mut receptor = receptor::ReceptorBuilder::default()
        .ip_address(local_ip().unwrap().to_string())
        .port(9000)
        .stream_name("Mic")
        .build()?;

    log::info("Receptor Playing");

    receptor.play()?;

    std::thread::sleep(std::time::Duration::from_secs(15));

    log::info("Receptor Pausing");

    receptor.pause()?;

    std::thread::sleep(std::time::Duration::from_secs(5));

    log::info("Receptor Playing");

    receptor.play()?;

    std::thread::sleep(std::time::Duration::from_secs(5));

    log::info("Receptor Pausing");

    receptor.pause()?;

    std::thread::sleep(std::time::Duration::from_secs(5));

    log::info("Receptor Playing");

    receptor.play()?;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
