#![windows_subsystem = "windows"] // ← これでコンソールが出なくなる
use macroquad::prelude::*;
use std::io::Write;
use std::net::{TcpStream, UdpSocket};

fn encode_keys() -> u16 {
    let mut bits = 0u16;
    let map = [
        (KeyCode::W, 0),
        (KeyCode::A, 1),
        (KeyCode::S, 2),
        (KeyCode::D, 3),
        (KeyCode::Q, 4),
        (KeyCode::E, 5),
        (KeyCode::R, 6),
        (KeyCode::F, 7),
        (KeyCode::C, 8),
        (KeyCode::Z, 9),
        (KeyCode::X, 10),
    ];
    for (k, b) in map {
        if is_key_down(k) {
            bits |= 1 << b;
        }
    }
    bits
}

#[macroquad::main("MQ Hybrid Client")]
async fn main() {
    prevent_quit();
    // UDP
    let udp = UdpSocket::bind("0.0.0.0:0").unwrap();
    udp.connect("127.0.0.1:5000").unwrap();

    // TCP (event)
    let mut tcp = TcpStream::connect("127.0.0.1:6000").unwrap();
    tcp.write_all(&[1]).ok(); // start

    // let mut next = Instant::now();

    loop {
        // println!("frame_start{i}");

        // println!("frame_end{i}");
        if is_quit_requested() {
            println!("quit requested");
            let _ = tcp.write_all(&[2]); // shutdown

            let _ = tcp.write_all(&[0]); // shutdown
            break;
        }

        let keys = encode_keys();
        let pkt = keys.to_le_bytes();
        let _ = udp.send(&pkt);

        clear_background(BLACK);
        draw_text(&format!("keys: {:016b}", keys), 20.0, 40.0, 30.0, WHITE);
        next_frame().await;
    }
}
