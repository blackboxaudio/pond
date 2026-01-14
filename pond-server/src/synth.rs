//! PondSynth - WebSocket-controlled polyphonic droplet synthesizer.
//!
//! Implements `rodio::Source` to stream audio samples.
//! Processes network triggers to spawn droplet sounds.

use std::time::Duration;

use bbx_dsp::{
    block::BlockId,
    buffer::{AudioBuffer, Buffer},
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::Graph,
};
use bbx_net::{NetBufferConsumer, NetMessageType};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rodio::Source;

use crate::dsp::{
    build_graph,
    droplet::{touch_to_ambisonic, touch_to_frequency, DropletVoice},
};

pub struct PondSynth {
    graph: Graph<f32>,
    voices: Vec<DropletVoice>,
    #[allow(dead_code)]
    master_gain_id: BlockId,
    output_buffers: Vec<AudioBuffer<f32>>,
    net_consumer: NetBufferConsumer,
    rng: ChaCha8Rng,
    sample_rate: u32,
    num_channels: usize,
    buffer_size: usize,
    channel_index: usize,
    sample_index: usize,
}

impl PondSynth {
    pub fn new(net_consumer: NetBufferConsumer) -> Self {
        let sample_rate = DEFAULT_SAMPLE_RATE as u32;
        let buffer_size = DEFAULT_BUFFER_SIZE;
        let num_channels = 2;

        let (graph, voices, master_gain_id) = build_graph();

        let mut output_buffers = Vec::with_capacity(num_channels);
        for _ in 0..num_channels {
            output_buffers.push(AudioBuffer::new(buffer_size));
        }

        Self {
            graph,
            voices,
            master_gain_id,
            output_buffers,
            net_consumer,
            rng: ChaCha8Rng::from_entropy(),
            sample_rate,
            num_channels,
            buffer_size,
            channel_index: 0,
            sample_index: 0,
        }
    }

    /// Process incoming network events (triggers).
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

    /// Spawn a new droplet at the given normalized coordinates.
    fn spawn_droplet(&mut self, x: f32, y: f32) {
        // Find a free voice or steal the oldest
        let voice_index = self.find_voice();

        let frequency = touch_to_frequency(y, &mut self.rng);
        let (azimuth, elevation) = touch_to_ambisonic(x, y);

        self.voices[voice_index].trigger(&mut self.graph, frequency, azimuth, elevation);
    }

    /// Find an available voice, or steal the oldest active one.
    fn find_voice(&mut self) -> usize {
        // First, check for finished voices and mark them inactive
        for voice in &mut self.voices {
            if voice.active && voice.is_finished() {
                voice.active = false;
            }
        }

        // Find first inactive voice
        if let Some(index) = self.voices.iter().position(|v| !v.active) {
            return index;
        }

        // All voices active - steal the oldest
        let oldest = self
            .voices
            .iter()
            .enumerate()
            .max_by_key(|(_, v)| v.age)
            .map(|(i, _)| i)
            .unwrap_or(0);

        oldest
    }

    /// Process one audio buffer and return the next sample.
    fn process(&mut self) -> f32 {
        // At the start of each buffer, process events and render
        if self.channel_index == 0 && self.sample_index == 0 {
            self.process_net_events();

            // Update voice ages
            for voice in &mut self.voices {
                if voice.active {
                    voice.tick();
                }
            }

            // Render the DSP graph
            let mut output_refs: Vec<&mut [f32]> = self
                .output_buffers
                .iter_mut()
                .map(|b| b.as_mut_slice())
                .collect();
            self.graph.process_buffers(&mut output_refs);
        }

        // Return current sample (interleaved L/R)
        let sample = self.output_buffers[self.channel_index][self.sample_index];

        // Advance position
        self.channel_index += 1;
        if self.channel_index >= self.num_channels {
            self.channel_index = 0;
            self.sample_index += 1;
            self.sample_index %= self.buffer_size;
        }

        sample
    }
}

impl Iterator for PondSynth {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.process())
    }
}

impl Source for PondSynth {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.num_channels as u16
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
