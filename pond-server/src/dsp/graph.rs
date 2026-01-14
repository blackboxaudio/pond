//! DSP graph construction for Pond.

use bbx_dsp::{
    block::BlockId,
    blocks::GainBlock,
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
};

use super::droplet::{DropletVoice, NUM_VOICES};

/// Build the complete DSP graph with voice pool.
/// Returns (graph, voices, master_gain_id).
pub fn build_graph() -> (Graph<f32>, Vec<DropletVoice>, BlockId) {
    let sample_rate = DEFAULT_SAMPLE_RATE;
    let buffer_size = DEFAULT_BUFFER_SIZE;
    let num_channels = 2; // Stereo output for now

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

    // Master output gain
    let master_gain_id = builder.add(GainBlock::new(-12.0, None));

    // Connect all voice outputs to master gain
    // Note: The graph will sum inputs automatically when multiple connections go to the same input
    for output_id in voice_outputs {
        builder.connect(output_id, 0, master_gain_id, 0);
        builder.connect(output_id, 1, master_gain_id, 1);
    }

    let graph = builder.build();

    (graph, voices, master_gain_id)
}
