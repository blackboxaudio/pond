//! PondSignal - Audio signal iterator for bbx_player backend integration.
//!
//! Processes network events and renders the DSP graph, yielding interleaved
//! f32 samples compatible with bbx_player's Backend trait.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use bbx_core::StackVec;
use bbx_dsp::{
    buffer::{AudioBuffer, Buffer},
    graph::{Graph, MAX_BLOCK_OUTPUTS},
};
use bbx_net::{NetBufferConsumer, NetMessageType};
use bbx_player::Source;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::voice::{touch_to_ambisonic, touch_to_frequency, Voice};

/// Audio signal iterator that processes network events and renders the DSP graph.
///
/// Implements `Iterator<Item = f32> + Send` for compatibility with bbx_player backends.
pub struct PondSignal {
    graph: Graph<f32>,
    voices: Vec<Voice>,
    output_buffers: Vec<AudioBuffer<f32>>,
    net_consumer: NetBufferConsumer,
    rng: ChaCha8Rng,

    sample_rate: u32,
    num_channels: usize,
    buffer_size: usize,

    channel_index: usize,
    sample_index: usize,

    stop_flag: Arc<AtomicBool>,
}

impl PondSignal {
    pub fn new(
        graph: Graph<f32>,
        voices: Vec<Voice>,
        net_consumer: NetBufferConsumer,
        stop_flag: Arc<AtomicBool>,
    ) -> Self {
        let sample_rate = graph.context().sample_rate as u32;
        let buffer_size = graph.context().buffer_size;
        let num_channels = graph.context().num_channels;

        let mut output_buffers = Vec::with_capacity(num_channels);
        for _ in 0..num_channels {
            output_buffers.push(AudioBuffer::new(buffer_size));
        }

        Self {
            graph,
            voices,
            output_buffers,
            net_consumer,
            rng: ChaCha8Rng::from_entropy(),
            sample_rate,
            num_channels,
            buffer_size,
            channel_index: 0,
            sample_index: 0,
            stop_flag,
        }
    }

    fn process_net_events(&mut self) {
        let events = self.net_consumer.drain_into_stack();

        for msg in events {
            if msg.message_type != NetMessageType::Trigger {
                continue;
            }

            let (x, y) = msg.payload.coordinates().unwrap_or((0.5, 0.5));
            self.spawn_droplet(x, y);
        }
    }

    fn spawn_droplet(&mut self, x: f32, y: f32) {
        let voice_index = self.find_voice();

        let frequency = touch_to_frequency(y, &mut self.rng);
        let (azimuth, elevation) = touch_to_ambisonic(x, y);

        self.voices[voice_index].trigger(&mut self.graph, frequency, azimuth, elevation);
    }

    fn find_voice(&mut self) -> usize {
        for voice in &mut self.voices {
            if voice.active && voice.is_finished() {
                voice.active = false;
            }
        }

        if let Some(index) = self.voices.iter().position(|v| !v.active) {
            return index;
        }

        self.voices
            .iter()
            .enumerate()
            .max_by_key(|(_, v)| v.age)
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    fn render_buffer(&mut self) {
        self.process_net_events();

        for voice in &mut self.voices {
            if voice.active {
                voice.tick();
            }
        }

        let mut output_refs: StackVec<&mut [f32], MAX_BLOCK_OUTPUTS> = StackVec::new();
        for buf in self.output_buffers.iter_mut() {
            let _ = output_refs.push(buf.as_mut_slice());
        }
        self.graph.process_buffers(output_refs.as_mut_slice());
    }

    fn next_sample(&mut self) -> f32 {
        if self.channel_index == 0 && self.sample_index == 0 {
            self.render_buffer();
        }

        let sample = self.output_buffers[self.channel_index][self.sample_index];

        self.channel_index += 1;
        if self.channel_index >= self.num_channels {
            self.channel_index = 0;
            self.sample_index += 1;
            self.sample_index %= self.buffer_size;
        }

        sample
    }
}

impl Iterator for PondSignal {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stop_flag.load(Ordering::SeqCst) {
            return None;
        }
        Some(self.next_sample())
    }
}

impl Source<f32> for PondSignal {
    fn channels(&self) -> u16 {
        self.num_channels as u16
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}
