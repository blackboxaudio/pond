//! WebSocket server for Pond.
//!
//! Handles client connections and forwards touch triggers to the audio engine.

use std::{
    net::UdpSocket,
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
    NetBufferConsumer,
};
use tokio::sync::mpsc;

const NET_BUFFER_CAPACITY: usize = 256;

fn get_local_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}

/// Start the WebSocket server.
///
/// Returns the network buffer consumer for the audio engine and the command sender for shutdown.
pub fn start(running: Arc<AtomicBool>) -> (NetBufferConsumer, mpsc::Sender<ServerCommand>) {
    let (producer, consumer) = net_buffer(NET_BUFFER_CAPACITY);
    let (command_tx, command_rx) = mpsc::channel::<ServerCommand>(256);

    let config = WsServerConfig {
        bind_addr: "0.0.0.0:8080".parse().unwrap(),
        ..Default::default()
    };

    let ws_server = WsServer::new(config, producer, command_rx);

    let command_tx_clone = command_tx.clone();
    let server_running = running;

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

    (consumer, command_tx)
}
