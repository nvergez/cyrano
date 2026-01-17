//! Simple linear resampler for streaming audio.
//!
//! Converts input samples from an arbitrary input sample rate to a target
//! sample rate using linear interpolation. Designed for low-latency streaming.

/// Streaming linear resampler state.
pub struct LinearResampler {
    step: f32,
    next_out_pos: f32,
    input_index: u64,
    prev_sample: f32,
    has_prev: bool,
}

impl LinearResampler {
    /// Create a new resampler that converts from `input_rate` to `output_rate`.
    pub fn new(input_rate: u32, output_rate: u32) -> Self {
        let step = input_rate as f32 / output_rate as f32;
        Self {
            step,
            next_out_pos: 0.0,
            input_index: 0,
            prev_sample: 0.0,
            has_prev: false,
        }
    }

    /// Push a single mono sample and append any generated output samples to `out`.
    pub fn push_sample(&mut self, sample: f32, out: &mut Vec<f32>) {
        if !self.has_prev {
            // First sample (index 0) defines the initial output at t=0.
            if self.next_out_pos == 0.0 {
                out.push(sample);
                self.next_out_pos += self.step;
            }
            self.prev_sample = sample;
            self.has_prev = true;
            self.input_index = 1;
            return;
        }

        let i = self.input_index as f32;
        while self.next_out_pos <= i {
            let frac = self.next_out_pos - (i - 1.0);
            let out_sample = self.prev_sample + (sample - self.prev_sample) * frac;
            out.push(out_sample);
            self.next_out_pos += self.step;
        }

        self.prev_sample = sample;
        self.input_index += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resampler_48k_to_16k_length() {
        let mut resampler = LinearResampler::new(48_000, 16_000);
        let mut out = Vec::new();
        // 1 second of 48k mono samples
        for i in 0..48_000 {
            resampler.push_sample(i as f32, &mut out);
        }
        // Expect ~16k samples
        assert!(out.len() >= 15_900 && out.len() <= 16_100);
    }

    #[test]
    fn test_resampler_44100_to_16k_length() {
        let mut resampler = LinearResampler::new(44_100, 16_000);
        let mut out = Vec::new();
        // 1 second of 44.1k mono samples
        for i in 0..44_100 {
            resampler.push_sample(i as f32, &mut out);
        }
        // Expect ~16k samples
        assert!(out.len() >= 15_900 && out.len() <= 16_100);
    }
}
