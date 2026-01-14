//! Pond - Ambisonic Water Droplet Installation Server
//!
//! A WebSocket server that responds to touch triggers from mobile devices
//! and plays spatialized water droplet sounds through an ambisonic speaker array.

mod dsp;
mod server;
mod synth;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use bbx_net::websocket::ServerCommand;
use rodio::{OutputStream, Source};

use synth::PondSynth;

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

    let synth = PondSynth::new(consumer);

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

    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(100));
    }

    let _ = command_tx.blocking_send(ServerCommand::Shutdown);

    println!("\nShutting down...");
}
