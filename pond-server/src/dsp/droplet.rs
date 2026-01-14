//! Water droplet voice synthesis.
//!
//! A simple droplet sound: sine burst with fast attack and medium decay.
//! Each voice has its own position for ambisonic encoding.

use bbx_dsp::{
    block::BlockId,
    blocks::{EnvelopeBlock, GainBlock, LowPassFilterBlock, OscillatorBlock, PannerBlock},
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use rand::Rng;

/// Number of polyphonic voices.
pub const NUM_VOICES: usize = 16;

/// Envelope timing constants (in seconds).
const ATTACK_TIME: f64 = 0.005;
const DECAY_TIME: f64 = 0.2;
const RELEASE_TIME: f64 = 0.1;

/// Total voice lifetime in buffers (at 48kHz, 512 buffer = ~10.6ms per buffer).
/// We'll use ~30 buffers (~320ms) which covers attack + decay + some margin.
const VOICE_LIFETIME_BUFFERS: u64 = 30;

/// A single droplet voice with oscillator, envelope, filter, and panner.
pub struct DropletVoice {
    pub oscillator_id: BlockId,
    pub envelope_id: BlockId,
    #[allow(dead_code)]
    filter_id: BlockId,
    #[allow(dead_code)]
    gain_id: BlockId,
    pub panner_id: BlockId,
    pub active: bool,
    pub age: u64,
}

impl DropletVoice {
    /// Create voice blocks and add them to the graph builder.
    /// Returns the voice with block IDs and the final output block ID.
    pub fn create(builder: &mut GraphBuilder<f32>, base_frequency: f64) -> (Self, BlockId) {
        // Oscillator: sine wave for clean droplet tone
        let oscillator_id = builder.add(OscillatorBlock::new(base_frequency, Waveform::Sine, None));

        // Envelope: fast attack (~5ms), medium decay (~200ms), no sustain
        let envelope_id = builder.add(EnvelopeBlock::new(ATTACK_TIME, DECAY_TIME, 0.0, RELEASE_TIME));

        // Low-pass filter: slight resonance for "plop" character
        let filter_id = builder.add(LowPassFilterBlock::new(2000.0, 1.5));

        // Per-voice gain
        let gain_id = builder.add(GainBlock::new(-6.0, None));

        // Panner: stereo for now (will upgrade to ambisonic)
        let panner_id = builder.add(PannerBlock::new(0.0));

        // Connect: osc -> filter -> gain -> panner
        // Note: envelope modulates gain (handled in process loop)
        builder
            .connect(oscillator_id, 0, filter_id, 0)
            .connect(filter_id, 0, gain_id, 0)
            .connect(gain_id, 0, panner_id, 0);

        let voice = Self {
            oscillator_id,
            envelope_id,
            filter_id,
            gain_id,
            panner_id,
            active: false,
            age: 0,
        };

        (voice, panner_id)
    }

    /// Trigger the voice with a new droplet sound.
    pub fn trigger(
        &mut self,
        graph: &mut Graph<f32>,
        frequency: f32,
        pan_position: f32,
    ) {
        // Set oscillator frequency
        if let Some(bbx_dsp::block::BlockType::Oscillator(osc)) =
            graph.get_block_mut(self.oscillator_id)
        {
            osc.set_midi_frequency(frequency);
        }

        // Set panner position (-100 to +100)
        if let Some(bbx_dsp::block::BlockType::Panner(panner)) =
            graph.get_block_mut(self.panner_id)
        {
            panner.position = bbx_dsp::parameter::Parameter::Constant(pan_position);
        }

        // Trigger envelope
        if let Some(bbx_dsp::block::BlockType::Envelope(env)) =
            graph.get_block_mut(self.envelope_id)
        {
            env.note_on();
        }

        self.active = true;
        self.age = 0;
    }

    /// Check if the voice has finished playing based on elapsed time.
    /// Since envelope stage is private, we estimate based on lifetime.
    pub fn is_finished(&self) -> bool {
        self.age >= VOICE_LIFETIME_BUFFERS
    }

    /// Update voice age (call each buffer).
    pub fn tick(&mut self) {
        self.age += 1;
    }
}

/// Convert touch coordinates (0-1) to pan position (-100 to +100).
pub fn touch_to_pan(x: f32) -> f32 {
    (x - 0.5) * 200.0
}

/// Generate a random droplet frequency based on y position.
/// Higher y (bottom of screen) = lower frequency (bigger drops).
pub fn touch_to_frequency(y: f32, rng: &mut impl Rng) -> f32 {
    let base = 800.0 + (1.0 - y) * 1200.0; // 800-2000 Hz range
    let variation = rng.gen_range(-100.0..100.0);
    (base + variation).clamp(600.0, 2500.0)
}
