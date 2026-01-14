//! Pond - Ambisonic Water Droplet Installation Server
//!
//! A WebSocket server that responds to touch triggers from mobile devices
//! and plays spatialized water droplet sounds through an ambisonic speaker array.

mod dsp;
mod server;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use bbx_net::{
    net_buffer,
    websocket::{ServerCommand, WsServer, WsServerConfig},
};
use rodio::{OutputStream, Source};
use tokio::sync::mpsc;

use server::PondSynth;

const NET_BUFFER_CAPACITY: usize = 256;

fn get_local_ip() -> Option<String> {
    use std::net::UdpSocket;

    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}

fn main() {
    println!("Pond - Ambisonic Water Droplet Installation");
    println!("============================================\n");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    // Create lock-free network buffer for audio thread communication
    let (producer, consumer) = net_buffer(NET_BUFFER_CAPACITY);
    let (command_tx, command_rx) = mpsc::channel::<ServerCommand>(256);

    // Configure WebSocket server
    let config = WsServerConfig {
        bind_addr: "0.0.0.0:8080".parse().unwrap(),
        ..Default::default()
    };

    let ws_server = WsServer::new(config, producer, command_rx);

    let command_tx_clone = command_tx.clone();
    let server_running = running.clone();

    // Spawn WebSocket server thread
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

        rt.block_on(async {
            let server_handle = tokio::spawn(async move {
                let _ = ws_server.run().await;
            });

            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

            let (response_tx, response_rx) = tokio::sync::oneshot::channel();
            let _ = command_tx_clone
                .send(ServerCommand::CreateRoom {
                    response: response_tx,
                })
                .await;

            match response_rx.await {
                Ok(room_code) => {
                    println!("Room created: {room_code}");
                    println!("\nClients should join with room code: {room_code}");
                }
                Err(_) => {
                    println!("Failed to receive room creation response");
                    return;
                }
            }

            let _ = server_handle.await;
        });

        server_running.store(false, Ordering::SeqCst);
    });

    thread::sleep(Duration::from_millis(100));

    if let Some(ip) = get_local_ip() {
        println!("WebSocket server listening on {ip}:8080");
    } else {
        println!("WebSocket server listening on port 8080");
    }

    println!("\nWaiting for touch triggers...");
    println!("Press Ctrl+C to exit.\n");

    // Create synthesizer with network consumer
    let synth = PondSynth::new(consumer);

    // Connect audio output via rodio
    let (_stream, stream_handle) = match OutputStream::try_default() {
        Ok(result) => result,
        Err(e) => {
            println!("Failed to open audio output: {e}");
            return;
        }
    };

    if let Err(e) = stream_handle.play_raw(synth.convert_samples()) {
        println!("Failed to start audio playback: {e}");
        return;
    }

    // Main loop - keep alive until Ctrl+C
    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(100));
    }

    let _ = command_tx.blocking_send(ServerCommand::Shutdown);

    println!("\nShutting down...");
}
