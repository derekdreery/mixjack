//! Signal modifiers - currently restricted to 1 input and 1 output.
//!
//! I think the secret to fast filters is to split up the work into 3 stages, the start (0..n), the
//! middle (n..len-n) and the end (len-n..len). Then during the middle (which we hope is most of
//! the work) we don't need to bounds-check.
//!
//! TODO look at choosing the size at compile-time using const generics.
use std::f32::consts::PI;

/// Currently hard-coded for f32. Could be made generic.
pub trait Effect {
    /// Apply the effect to an input buffer to produce an output buffer. Any output from the filter
    /// should be added to the output slice, so filters can be combined additively.
    fn apply(&mut self, input: &[f32], output: &mut [f32]);
}

#[derive(Debug, Clone)]
pub struct FIRFilter {
    weights: Vec<f32>,
    weights_original: Vec<f32>,
    buffer: Vec<f32>,
    gain: f32,
}

impl FIRFilter {
    /// If `len` is even it is increased by 1.
    pub fn low_pass(cutoff: f32, sample_freq: f32, len: usize) -> Self {
        let len = if len % 2 == 0 { len + 1 } else { len };

        let mut weights = vec![0.0; len];

        let cutoff = cutoff / sample_freq;
        let angular_cutoff = 2.0 * PI * cutoff;
        let middle = (len / 2) as isize; // drop remainder

        for i in -middle..=middle {
            if i == 0 {
                weights[middle as usize] = 2.0 * cutoff;
            } else {
                weights[(i + middle) as usize] =
                    (angular_cutoff * i as f32).sin() / (PI * i as f32);
            }
        }
        Self::new(weights)
    }

    pub fn band_pass(low_cutoff: f32, high_cutoff: f32, sample_freq: f32, len: usize) -> Self {
        let len = if len % 2 == 0 { len + 1 } else { len };

        let mut weights = vec![0.0; len];

        let low_cutoff = low_cutoff / sample_freq;
        let high_cutoff = high_cutoff / sample_freq;
        let low_angular = low_cutoff * 2.0 * PI;
        let high_angular = high_cutoff * 2.0 * PI;
        let middle = (len / 2) as isize;

        for i in -middle..=middle {
            if i == 0 {
                weights[middle as usize] = 1.0 - 2.0 * (high_cutoff - low_cutoff);
            } else {
                let i_f = i as f32;
                weights[(middle + i) as usize] = (high_angular * i_f).sin() / (PI * i_f)
                    - (low_angular * i_f).sin() / (PI * i_f);
            }
        }
        Self::new(weights)
    }

    pub fn high_pass(cutoff: f32, sample_freq: f32, len: usize) -> FIRFilter {
        let len = if len % 2 == 0 { len + 1 } else { len };

        let mut weights = vec![0.0; len];

        let cutoff = cutoff / sample_freq;
        let angular_cutoff = 2.0 * PI * cutoff;
        let middle = (len / 2) as isize; // drop remainder

        for i in -middle..=middle {
            if i == 0 {
                weights[middle as usize] = 1.0 - 2.0 * cutoff;
            } else {
                weights[(i + middle) as usize] =
                    -(angular_cutoff * i as f32).sin() / (PI * i as f32);
            }
        }
        Self::new(weights)
    }

    fn new(weights: Vec<f32>) -> Self {
        let len = weights.len();
        Self {
            weights: weights.clone(),
            weights_original: weights,
            buffer: vec![0.0; len - 1],
            gain: 1.0,
        }
    }

    #[inline]
    pub fn set_gain(&mut self, gain: f32) {
        if gain == self.gain {
            return;
        }
        for (orig, weight) in self.weights_original.iter().zip(self.weights.iter_mut()) {
            *weight = orig * gain;
        }
    }
}

impl Effect for FIRFilter {
    fn apply(&mut self, input: &[f32], output: &mut [f32]) {
        assert!(input.len() == output.len());

        let n = self.weights.len();
        let len = input.len();

        // in = prev, out = this
        for (buf_sample, output_sample) in self.buffer.iter().zip(output.iter_mut()) {
            *output_sample += *buf_sample;
        }

        // in = this, out = this
        for input_idx in 0..(len - n) {
            let sample_in = input[input_idx];
            if sample_in != 0.0 {
                for (output_idx, weight) in (input_idx..input_idx + n).zip(self.weights.iter()) {
                    output[output_idx] += sample_in * weight;
                }
            }
        }

        // in = this, out = this + next
        for s in self.buffer.iter_mut() {
            // zero out inter-frame buffer.
            *s = 0.0;
        }
        for input_idx in (len - n)..len {
            let sample_in = input[input_idx];
            if sample_in != 0.0 {
                let mut idx = 0;
                // first add into this frame
                while idx < (len - input_idx) {
                    output[input_idx + idx] += sample_in * self.weights[idx];
                    idx += 1;
                }
                // then add into the next frame via. the inter-frame buffer.
                while idx < n {
                    self.buffer[idx - (len - input_idx)] += sample_in * self.weights[idx];
                    idx += 1;
                }
            }
        }
    }
}

/*
#[derive(Debug)]
pub struct RingBuffer {
    buf: Box<[f32]>,
    pos: usize,
}

impl RingBuffer {
    #[inline]
    fn new(len: usize) -> Self {
        Self {
            buf: vec![0.0; len].into_boxed_slice(),
            pos: 0,
        }
    }

    #[inline]
    fn push(&mut self, val: f32) {
        debug_assert!(self.buf.get(self.pos).is_some());
        unsafe {
            *self.buf.get_unchecked_mut(self.pos) = val;
        };
        self.inc();
    }

    #[inline]
    fn inc(&mut self) {
        self.pos = (self.pos + 1) % self.buf.len();
    }

    #[inline]
    fn iter<'a>(&'a self) -> impl Iterator<Item = f32> + 'a {
        RBIter {
            inner: self,
            idx: self.pos,
        }
    }
}

struct RBIter<'a> {
    inner: &'a RingBuffer,
    idx: usize,
}

impl<'a> Iterator for RBIter<'a> {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let len = self.inner.buf.len();
        let pos = self.inner.pos;
        if self.idx == (pos + len - 1) % len {
            None
        } else {
            let out = Some(self.inner.buf[self.idx]);
            self.idx = (self.idx + 1) % len;
            out
        }
    }
}
*/
