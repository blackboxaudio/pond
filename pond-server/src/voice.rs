//! Water droplet voice synthesis.
//!
//! A simple droplet sound: sine burst with fast attack and medium decay.
//! Each voice has its own position for ambisonic encoding.

use bbx_dsp::{
    block::{Block, BlockId},
    blocks::{
        EnvelopeBlock, GainBlock, LowPassFilterBlock, OscillatorBlock, PannerBlock, VcaBlock,
    },
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use rand::Rng;

/// Number of polyphonic voices.
pub const NUM_VOICES: usize = 8;

/// Envelope timing constants (in seconds).
const ATTACK_TIME: f64 = 0.005;
const DECAY_TIME: f64 = 0.2;
const RELEASE_TIME: f64 = 0.1;

/// Total voice lifetime in buffers (at 48kHz, 512 buffer = ~10.6ms per buffer).
/// We'll use ~30 buffers (~320ms) which covers attack + decay + some margin.
const VOICE_LIFETIME_BUFFERS: u64 = 30;

/// A single droplet voice with oscillator, envelope, VCA, filter, and panner.
pub struct Voice {
    pub oscillator_id: BlockId,
    pub envelope_id: BlockId,
    #[allow(dead_code)]
    vca_id: BlockId,
    #[allow(dead_code)]
    filter_id: BlockId,
    #[allow(dead_code)]
    gain_id: BlockId,
    pub panner_id: BlockId,
    pub active: bool,
    pub age: u64,
}

impl Voice {
    /// Create voice blocks and add them to the graph builder.
    /// Returns the voice with block IDs and the final output block ID.
    pub fn create(
        builder: &mut GraphBuilder<f32>,
        base_frequency: f64,
        sample_rate: f64,
    ) -> (Self, BlockId) {
        let oscillator_id = builder.add(OscillatorBlock::new(base_frequency, Waveform::Sine, None));

        let envelope_id = builder.add(EnvelopeBlock::new(
            ATTACK_TIME,
            DECAY_TIME,
            0.0,
            RELEASE_TIME,
        ));

        let vca_id = builder.add(VcaBlock::new());

        let filter_id = builder.add(LowPassFilterBlock::new(2000.0, 1.5));

        let gain_id = builder.add(GainBlock::new(-6.0, None));

        let mut panner = PannerBlock::new_ambisonic(1);
        panner.set_smoothing(sample_rate, 0.01);
        let panner_id = builder.add(panner);

        builder
            .connect(oscillator_id, 0, vca_id, 0)
            .connect(envelope_id, 0, vca_id, 1)
            .connect(vca_id, 0, filter_id, 0)
            .connect(filter_id, 0, gain_id, 0)
            .connect(gain_id, 0, panner_id, 0);

        let voice = Self {
            oscillator_id,
            envelope_id,
            vca_id,
            filter_id,
            gain_id,
            panner_id,
            active: false,
            age: 0,
        };

        (voice, panner_id)
    }

    /// Trigger the voice with a new droplet sound at an ambisonic position.
    pub fn trigger(
        &mut self,
        graph: &mut Graph<f32>,
        frequency: f32,
        azimuth: f32,
        elevation: f32,
    ) {
        if let Some(bbx_dsp::block::BlockType::Oscillator(osc)) =
            graph.get_block_mut(self.oscillator_id)
        {
            osc.set_midi_frequency(frequency);
        }

        if let Some(bbx_dsp::block::BlockType::Panner(panner)) = graph.get_block_mut(self.panner_id)
        {
            panner.azimuth = bbx_dsp::parameter::Parameter::Constant(azimuth);
            panner.elevation = bbx_dsp::parameter::Parameter::Constant(elevation);
        }

        if let Some(bbx_dsp::block::BlockType::Envelope(env)) =
            graph.get_block_mut(self.envelope_id)
        {
            env.note_on();
        }

        self.active = true;
        self.age = 0;
    }

    /// Check if the voice has finished playing based on elapsed time.
    pub fn is_finished(&self) -> bool {
        self.age >= VOICE_LIFETIME_BUFFERS
    }

    /// Update voice age (call each buffer).
    pub fn tick(&mut self) {
        self.age += 1;
    }
}

/// Convert touch coordinates (0-1) to ambisonic spherical coordinates.
/// Returns (azimuth, elevation) in degrees.
pub fn touch_to_ambisonic(x: f32, y: f32) -> (f32, f32) {
    let dx = x - 0.5;
    let dy = y - 0.5;

    let azimuth = f32::atan2(-dx, -dy).to_degrees();
    let elevation = 0.0;

    (azimuth, elevation)
}

/// Musical frequencies: D major pentatonic scale across two octaves.
const MUSICAL_FREQUENCIES: [f32; 10] = [
    587.33,  // D5
    659.26,  // E5
    739.99,  // F#5
    880.00,  // A5
    987.77,  // B5
    1174.66, // D6
    1318.51, // E6
    1479.98, // F#6
    1760.00, // A6
    1975.53, // B6
];

/// Generate a musical droplet frequency chosen randomly from the scale.
pub fn touch_to_frequency(_y: f32, rng: &mut impl Rng) -> f32 {
    let index = rng.gen_range(0..MUSICAL_FREQUENCIES.len());
    let base = MUSICAL_FREQUENCIES[index];
    let variation = rng.gen_range(-5.0..5.0);
    base + variation
}
