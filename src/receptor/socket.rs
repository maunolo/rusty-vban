use anyhow::{anyhow, Context, Result};

use std::net::{IpAddr, SocketAddr, UdpSocket};

use byteorder::{ByteOrder, LittleEndian};

use super::ring_buffer::VbanStreamProducer;
use crate::protocol::header::{Codec, SubProtocol};
use crate::protocol::packet::{Packet, MAX_PACKET_SIZE};

#[derive(Debug)]
pub struct VbanReceptorSocket {
    socket: UdpSocket,
    buf: [u8; MAX_PACKET_SIZE],
}

impl VbanReceptorSocket {
    pub fn new(port: u16) -> Result<Self> {
        let addr = SocketAddr::new("0.0.0.0".parse()?, port);

        println!("Binding to: {:?}", addr);

        let socket = UdpSocket::bind(addr)?;

        Ok(VbanReceptorSocket {
            socket,
            buf: [0; MAX_PACKET_SIZE],
        })
    }

    pub fn start_receive_loop<F>(
        mut self,
        ip_address: &str,
        stream_name: &str,
        channels: u8,
        mut producer: VbanStreamProducer,
        should_run_callback: F,
    ) where
        F: Fn() -> bool,
    {
        while should_run_callback() {
            match self.receive_packet(ip_address, stream_name, channels) {
                Ok(pkt) => {
                    for sample in pkt.data.chunks_exact(2) {
                        let sample = LittleEndian::read_i16(&sample);
                        producer.push(sample).ok();
                    }
                }
                Err(e) => println!("Warning: {}", e),
            }
        }
    }

    fn receive_packet(
        &mut self,
        ip_address: &str,
        stream_name: &str,
        channels: u8,
    ) -> Result<Packet> {
        let (amt, src) = self.socket.recv_from(&mut self.buf).context(format!(
            "Failed to receive packet from socket: {:?}",
            self.socket
        ))?;

        check_src(ip_address, &src)?;

        let packet = Packet::try_from(&self.buf[..amt])?;

        check_audio_pkt(stream_name, channels, &packet)?;

        Ok(packet)
    }
}

fn check_src(ip_address: &str, src: &SocketAddr) -> Result<()> {
    if src.ip()
        != ip_address
            .parse::<IpAddr>()
            .context("Informed IP address is invalid")?
    {
        return Err(anyhow!("Wrong source"));
    }

    Ok(())
}

fn check_audio_pkt(stream_name: &str, channels: u8, pkt: &Packet) -> Result<()> {
    let header = pkt.header();

    // Check stream name
    if header.stream_name() != stream_name {
        return Err(anyhow!("Wrong stream name"));
    }

    if header.num_channels() != channels {
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
