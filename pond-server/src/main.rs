//! Pond - Ambisonic Water Droplet Installation Server
//!
//! A WebSocket server that responds to touch triggers from mobile devices
//! and plays spatialized water droplet sounds using bbx_player for audio output.

mod graph;
mod server;
mod signal;
mod voice;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use bbx_net::websocket::ServerCommand;
use bbx_player::{backends::RodioBackend, Backend};

use graph::build_graph;
use signal::PondSignal;

fn main() {
    println!("Pond - Ambisonic Water Droplet Installation");
    println!("============================================\n");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let (consumer, command_tx) = server::start(running.clone());

    println!("\nWaiting for touch triggers...");
    println!("Press Ctrl+C to exit.\n");

    let (graph, voices, _decoder_id) = build_graph();

    let stop_flag = Arc::new(AtomicBool::new(false));

    let signal = PondSignal::new(graph, voices, consumer, Arc::clone(&stop_flag));

    let backend = match RodioBackend::try_default() {
        Ok(backend) => backend,
        Err(e) => {
            eprintln!("Failed to create audio backend: {e}");
            return;
        }
    };

    if let Err(e) = Box::new(backend).play(Box::new(signal), Arc::clone(&stop_flag)) {
        eprintln!("Failed to start audio playback: {e}");
        return;
    }

    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(100));
    }

    stop_flag.store(true, Ordering::SeqCst);

    let _ = command_tx.blocking_send(ServerCommand::Shutdown);

    println!("\nShutting down...");
}
