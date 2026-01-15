//! DSP graph construction for Pond.

use bbx_dsp::{
    block::BlockId,
    blocks::{
        effectors::{binaural_decoder::BinauralStrategy, mixer::NormalizationStrategy},
        BinauralDecoderBlock, MixerBlock,
    },
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
};

use crate::voice::{Voice, NUM_VOICES};

/// Build the complete DSP graph with voice pool and binaural decoder.
/// Returns (graph, voices, decoder_id).
pub fn build_graph() -> (Graph<f32>, Vec<Voice>, BlockId) {
    let sample_rate = DEFAULT_SAMPLE_RATE;
    let buffer_size = DEFAULT_BUFFER_SIZE;
    let num_channels = 2;

    let mut builder = GraphBuilder::new(sample_rate, buffer_size, num_channels);

    let mut voices = Vec::with_capacity(NUM_VOICES);
    let mut voice_outputs = Vec::with_capacity(NUM_VOICES);

    for i in 0..NUM_VOICES {
        let base_freq = 1000.0 + (i as f64 * 50.0);
        let (voice, output_id) = Voice::create(&mut builder, base_freq, sample_rate);
        voices.push(voice);
        voice_outputs.push(output_id);
    }

    let submixer_a_id =
        builder.add(MixerBlock::new(4, 4).with_normalization(NormalizationStrategy::Average));
    let submixer_b_id =
        builder.add(MixerBlock::new(4, 4).with_normalization(NormalizationStrategy::Average));

    for (source_idx, output_id) in voice_outputs[0..4].iter().enumerate() {
        for ch in 0..4 {
            let input_idx = source_idx * 4 + ch;
            builder.connect(*output_id, ch, submixer_a_id, input_idx);
        }
    }

    for (source_idx, output_id) in voice_outputs[4..8].iter().enumerate() {
        for ch in 0..4 {
            let input_idx = source_idx * 4 + ch;
            builder.connect(*output_id, ch, submixer_b_id, input_idx);
        }
    }

    let master_mixer_id =
        builder.add(MixerBlock::new(2, 4).with_normalization(NormalizationStrategy::Average));

    for ch in 0..4 {
        builder.connect(submixer_a_id, ch, master_mixer_id, ch);
    }
    for ch in 0..4 {
        builder.connect(submixer_b_id, ch, master_mixer_id, 4 + ch);
    }

    let decoder_id = builder.add(BinauralDecoderBlock::with_strategy(
        1,
        BinauralStrategy::Matrix,
    ));

    for ch in 0..4 {
        builder.connect(master_mixer_id, ch, decoder_id, ch);
    }

    let graph = builder.build();

    (graph, voices, decoder_id)
}
