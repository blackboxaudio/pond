//! DSP graph construction for Pond.

use bbx_dsp::{
    block::BlockId,
    blocks::{effectors::mixer::NormalizationStrategy, BinauralDecoderBlock, MixerBlock},
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
};

use super::droplet::{DropletVoice, NUM_VOICES};

/// Build the complete DSP graph with voice pool and binaural decoder.
/// Returns (graph, voices, decoder_id).
pub fn build_graph() -> (Graph<f32>, Vec<DropletVoice>, BlockId) {
    let sample_rate = DEFAULT_SAMPLE_RATE;
    let buffer_size = DEFAULT_BUFFER_SIZE;
    let num_channels = 2; // Final stereo output (headphones)

    let mut builder = GraphBuilder::new(sample_rate, buffer_size, num_channels);

    // Create voice pool
    let mut voices = Vec::with_capacity(NUM_VOICES);
    let mut voice_outputs = Vec::with_capacity(NUM_VOICES);

    for i in 0..NUM_VOICES {
        // Stagger base frequencies slightly for variety
        let base_freq = 1000.0 + (i as f64 * 50.0);
        let (voice, output_id) = DropletVoice::create(&mut builder, base_freq);
        voices.push(voice);
        voice_outputs.push(output_id);
    }

    // Create two submixers to cascade voice outputs (avoid exceeding MAX_BLOCK_INPUTS)
    // Each MixerBlock receives 4 sources × 4 channels = 16 inputs
    let submixer_a_id =
        builder.add(MixerBlock::new(4, 4).with_normalization(NormalizationStrategy::Average));
    let submixer_b_id =
        builder.add(MixerBlock::new(4, 4).with_normalization(NormalizationStrategy::Average));

    // Connect voices 0-3 to submixer A
    // MixerBlock input layout: source_idx * num_channels + channel_idx
    for (source_idx, output_id) in voice_outputs[0..4].iter().enumerate() {
        for ch in 0..4 {
            let input_idx = source_idx * 4 + ch;
            builder.connect(*output_id, ch, submixer_a_id, input_idx);
        }
    }

    // Connect voices 4-7 to submixer B
    for (source_idx, output_id) in voice_outputs[4..8].iter().enumerate() {
        for ch in 0..4 {
            let input_idx = source_idx * 4 + ch;
            builder.connect(*output_id, ch, submixer_b_id, input_idx);
        }
    }

    // Master mixer combines 2 submixers × 4 channels = 8 inputs
    // Gain staging: per-voice -6dB, submixer /4, master /2 = total ~-18dB
    let master_mixer_id =
        builder.add(MixerBlock::new(2, 4).with_normalization(NormalizationStrategy::Average));
    for ch in 0..4 {
        builder.connect(submixer_a_id, ch, master_mixer_id, ch); // Source 0
        builder.connect(submixer_b_id, ch, master_mixer_id, 4 + ch); // Source 1
    }

    // Binaural decoder: B-format (4ch) → Stereo (2ch) with HRTF for headphones
    let decoder_id = builder.add(BinauralDecoderBlock::new(1));

    // Connect master mixer directly to decoder (GainBlock is mono, can't use for 4ch)
    for ch in 0..4 {
        builder.connect(master_mixer_id, ch, decoder_id, ch);
    }

    let graph = builder.build();

    (graph, voices, decoder_id)
}
