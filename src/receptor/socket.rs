use anyhow::{anyhow, Context, Result};

use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;

use byteorder::{ByteOrder, LittleEndian};

use crate::protocol::header::{Codec, SubProtocol};
use crate::protocol::packet::{Packet, MAX_PACKET_SIZE};
use crate::utils::log;

use super::stream::VbanStreamProducer;

pub struct VbanReceptorSocketBuilder {
    port: Option<u16>,
    incoming_addr: Option<String>,
    incoming_stream_name: Option<String>,
    channels: Option<u8>,
    producer: Option<VbanStreamProducer>,
}

impl VbanReceptorSocketBuilder {
    pub fn default() -> Self {
        Self {
            port: None,
            incoming_addr: None,
            incoming_stream_name: None,
            channels: None,
            producer: None,
        }
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn incoming_addr(mut self, incoming_addr: &str) -> Self {
        self.incoming_addr = Some(incoming_addr.to_string());
        self
    }

    pub fn incoming_stream_name(mut self, incoming_stream_name: &str) -> Self {
        self.incoming_stream_name = Some(incoming_stream_name.to_string());
        self
    }

    pub fn channels(mut self, channels: u8) -> Self {
        self.channels = Some(channels);
        self
    }

    pub fn producer(mut self, producer: VbanStreamProducer) -> Self {
        self.producer = Some(producer);
        self
    }

    pub fn build(self) -> Result<VbanReceptorSocket> {
        let port = self.port.context("Port is required")?;
        let incoming_addr = self
            .incoming_addr
            .context("Incomming address is required")?;
        let incoming_stream_name = self
            .incoming_stream_name
            .context("Incomming stream name is required")?;
        let channels = self.channels.context("Channels is required")?;
        let producer = self.producer.context("Producer is required")?;
        let addr = SocketAddr::new("0.0.0.0".parse()?, port);
        let socket = UdpSocket::bind(addr)?;

        Ok(VbanReceptorSocket {
            socket: Arc::new(socket),
            incoming_addr: incoming_addr.parse()?,
            incoming_stream_name,
            channels,
            producer: Some(producer),
            player_handle: None,
            player_running: None,
        })
    }
}

pub struct VbanReceptorSocket {
    socket: Arc<UdpSocket>,
    incoming_addr: IpAddr,
    incoming_stream_name: String,
    channels: u8,
    producer: Option<VbanStreamProducer>,
    player_handle: Option<std::thread::JoinHandle<Result<VbanStreamProducer>>>,
    player_running: Option<Arc<AtomicBool>>,
}

impl VbanReceptorSocket {
    pub fn start(&mut self) -> Result<()> {
        let player_running = Arc::new(AtomicBool::new(true));
        let player_running_clone = player_running.clone();
        let socket = self.socket.clone();
        let incoming_addr = self.incoming_addr.clone();
        let incoming_stream_name = self.incoming_stream_name.clone();
        let channels = self.channels;
        let mut producer = self
            .producer
            .take()
            .ok_or(anyhow!("No producer available"))?;

        let player_handle = thread::spawn(move || {
            let mut buf = [0; MAX_PACKET_SIZE];

            while player_running_clone.load(std::sync::atomic::Ordering::Relaxed) {
                let packet = Self::receive_packet(
                    socket.clone(),
                    &incoming_addr,
                    &incoming_stream_name,
                    &channels,
                    &mut buf,
                );
                match packet {
                    Ok(packet) => {
                        for sample in packet.data.chunks_exact(2) {
                            let sample = LittleEndian::read_i16(&sample);
                            producer.push(sample).ok();
                        }
                    }
                    Err(e) => log::warn(&e.to_string()),
                }
            }

            Ok(producer)
        });
        self.player_handle = Some(player_handle);
        self.player_running = Some(player_running);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.player_running
            .clone()
            .ok_or(anyhow!(
                "Player thread is not running, did you call start()?"
            ))?
            .store(false, std::sync::atomic::Ordering::Relaxed);
        let player_handle = self.player_handle.take().ok_or(anyhow!(
            "Player thread is not running, did you call start()?"
        ))?;
        match player_handle.join() {
            Ok(producer) => {
                match producer {
                    Ok(producer) => self.producer = Some(producer),
                    Err(e) => return Err(e),
                };
            }
            Err(_) => return Err(anyhow!("Failed to join player thread")),
        };
        self.player_handle = None;
        self.player_running = None;
        Ok(())
    }

    fn receive_packet(
        socket: Arc<UdpSocket>,
        incoming_addr: &IpAddr,
        incoming_stream_name: &str,
        channels: &u8,
        buf: &mut [u8],
    ) -> Result<Packet> {
        let (amt, src) = (*socket).recv_from(buf).context(format!(
            "Failed to receive packet from socket: {:?}",
            socket
        ))?;

        check_src(&incoming_addr, &src)?;

        let packet = Packet::try_from(&buf[..amt])?;

        check_audio_pkt(&incoming_stream_name, channels, &packet)?;

        Ok(packet)
    }
}

impl Drop for VbanReceptorSocket {
    fn drop(&mut self) {
        let _ = self.stop();
        if let Some(handle) = self.player_handle.take() {
            let _ = handle.join();
        }
    }
}

fn check_src(ip_address: &IpAddr, src: &SocketAddr) -> Result<()> {
    if &src.ip() != ip_address {
        return Err(anyhow!("Wrong source"));
    }

    Ok(())
}

fn check_audio_pkt(stream_name: &str, channels: &u8, pkt: &Packet) -> Result<()> {
    let header = pkt.header();

    // Check stream name
    if header.stream_name() != stream_name {
        return Err(anyhow!("Wrong stream name"));
    }

    if header.num_channels() != *channels {
        return Err(anyhow!("Wrong number of channels"));
    }

    if !matches!(header.sub_protocol(), SubProtocol::Audio) {
        return Err(anyhow!("Wrong sub protocol"));
    }

    if !matches!(header.codec(), Codec::PCM) {
        return Err(anyhow!("Wrong codec"));
    }

    Ok(())
}
