//! Synthetic benchmark: flood Art-Net TX at maximum rate.
//!
//! Measures packet construction throughput and UDP send throughput
//! to a black-hole address (no listener required).

use std::time::Instant;
use std::net::UdpSocket;

fn build_artdmx(universe: u16, data: &[u8; 512]) -> Vec<u8> {
    let mut pkt = Vec::with_capacity(530);
    pkt.extend_from_slice(b"Art-Net\0");
    pkt.push(0x00);
    pkt.push(0x50); // OpCode ArtDMX = 0x5000 LE
    pkt.push(0x00);
    pkt.push(14); // ProtVer 14 BE
    pkt.push(0x00); // Sequence
    pkt.push(0x00); // Physical
    pkt.push((universe & 0xFF) as u8); // Universe Lo
    pkt.push(((universe >> 8) & 0xFF) as u8); // Universe Hi
    pkt.push(0x02); // Length Hi  (512 = 0x0200)
    pkt.push(0x00); // Length Lo
    pkt.extend_from_slice(data);
    pkt
}

fn main() {
    const N: usize = 100_000;
    let data = [0u8; 512];

    // ── 1. Packet construction throughput ─────────────────────────────────────
    let t0 = Instant::now();
    for i in 0..N {
        let _ = build_artdmx((i % 256) as u16, &data);
    }
    let dt = t0.elapsed();
    println!(
        "Packet construction: {:>8} pkt in {:?}  =>  {:.1} pkt/sec  ({:.1} Mbps)",
        N,
        dt,
        N as f64 / dt.as_secs_f64(),
        (N * 530 * 8) as f64 / dt.as_secs_f64() / 1_000_000.0
    );

    // ── 2. UDP send throughput (black hole) ───────────────────────────────────
    let sock = UdpSocket::bind("0.0.0.0:0").expect("bind");
    let dest = "127.0.0.1:6454";
    let pkt = build_artdmx(0, &data);

    let t0 = Instant::now();
    for _ in 0..N {
        let _ = sock.send_to(&pkt, dest);
    }
    let dt = t0.elapsed();
    println!(
        "UDP send (black hole): {:>8} pkt in {:?}  =>  {:.1} pkt/sec  ({:.1} Mbps)",
        N,
        dt,
        N as f64 / dt.as_secs_f64(),
        (N * pkt.len() * 8) as f64 / dt.as_secs_f64() / 1_000_000.0
    );
}
