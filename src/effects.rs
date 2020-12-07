//! Signal modifiers - currently restricted to 1 input and 1 output.
//!
//! I think the secret to fast filters is to split up the work into 3 stages, the start (0..n), the
//! middle (n..len-n) and the end (len-n..len). Then during the middle (which we hope is most of
//! the work) we don't need to bounds-check.
//!
//! TODO look at choosing the size at compile-time using const generics.
use crate::{gui::UiMsg, monitor_data::MonitorData};
use crossbeam_channel as channel;
use dasp::ring_buffer::{Bounded, Slice, SliceMut};
use fftw::{
    array::AlignedVec,
    plan::{R2RPlan, R2RPlan32},
    types::{Flag, R2RKind},
};
use itertools::izip;
use std::{f32::consts::PI, fmt};

pub type MonitorSpectrum = MonitorData<Box<[f32]>>;

/// Currently hard-coded for f32. Could be made generic.
pub trait Effect {
    /// Apply the effect to an input buffer to produce an output buffer. Any output from the filter
    /// should be added to the output slice, so filters can be combined additively (equivalent to
    /// filters running in parallel with the result summed).
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
    ///
    /// This filter uses the sinc function with the blackman window.
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
                let i_f = i as f32;
                let bman = blackman(len, (i + middle) as usize);
                let weight = bman * (angular_cutoff * i_f).sin() / (PI * i_f);
                weights[(i + middle) as usize] = weight;
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
                weights[middle as usize] = (1.0 - 2.0 * (high_cutoff - low_cutoff)) as f32;
            } else {
                let bman = blackman(len, (i + middle) as usize);
                let i_f = i as f32;
                let weight = (high_angular * i_f).sin() / (PI * i_f)
                    - (low_angular * i_f).sin() / (PI * i_f);
                let weight = weight * bman;
                weights[(middle + i) as usize] = weight;
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
                weights[middle as usize] = (1.0 - 2.0 * cutoff) as f32;
            } else {
                let i_f = i as f32;
                let bman = blackman(len, (i + middle) as usize);
                let weight = -bman * (angular_cutoff * i_f).sin() / (PI * i_f);
                weights[(i + middle) as usize] = weight;
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
        self.gain = gain;
    }

    pub fn passthru() -> Self {
        Self::new(vec![1.0])
    }

    pub fn debug_window(&self) -> Vec<f32> {
        let len = self.weights.len();
        (0..len).map(|i| blackman(len, i)).collect()
    }

    pub fn debug_weights(&self) -> Vec<f32> {
        self.weights_original.clone()
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

#[derive(Debug, Clone)]
pub struct IIRFilter {
    in_weights: Vec<f32>,
    in_weights_original: Vec<f32>,
    out_weights: Vec<f32>,
    out_weights_original: Vec<f32>,
    buffer: Vec<f32>,
    output_buffer: Vec<f32>,
    gain: f32,
}

impl IIRFilter {
    /// Note that if the filter is applied to data longer than the frame_len, then this function
    /// will allocate.
    fn new(in_weights: Vec<f32>, out_weights: Vec<f32>, frame_len: usize) -> Self {
        // The current output depends on the current input, then a number of previous inputs and
        // outputs.
        assert!(in_weights.len() == out_weights.len() + 1);
        let buffer_len = out_weights.len();
        Self {
            in_weights: in_weights.clone(),
            in_weights_original: in_weights,
            out_weights: out_weights.clone(),
            out_weights_original: out_weights,
            buffer: vec![0.0; buffer_len],
            output_buffer: vec![0.0; frame_len],
            gain: 1.0,
        }
    }

    #[inline]
    pub fn set_gain(&mut self, gain: f32) {
        if gain == self.gain {
            return;
        }
        for (orig, weight) in self
            .in_weights_original
            .iter()
            .zip(self.in_weights.iter_mut())
        {
            *weight = orig * gain;
        }
        for (orig, weight) in self
            .out_weights_original
            .iter()
            .zip(self.out_weights.iter_mut())
        {
            *weight = orig * gain;
        }
        self.gain = gain;
    }

    pub fn len(&self) -> usize {
        self.in_weights.len()
    }

    pub fn single_pole(a0: f32, a1: f32, b1: f32, frame_len: usize) -> Self {
        let in_weights = vec![a0, a1];
        let out_weights = vec![b1];
        Self::new(in_weights, out_weights, frame_len)
    }

    pub fn low_pass(cutoff: f32, sample_freq: f32, frame_len: usize) -> Self {
        let cutoff = cutoff / sample_freq;

        let x = (-2.0 * PI * cutoff).exp();
        Self::single_pole(1.0 - x, 0.0, x, frame_len)
    }

    pub fn high_pass(cutoff: f32, sample_freq: f32, frame_len: usize) -> Self {
        let cutoff = cutoff / sample_freq;

        let x = (-2.0 * PI * cutoff).exp();
        Self::single_pole(0.5 * (1.0 + x), -0.5 * (1.0 + x), x, frame_len)
    }

    /// Calculate this as what's left after getting the low and high frequences.
    pub fn band_pass(
        low_cutoff: f32,
        high_cutoff: f32,
        sample_freq: f32,
        frame_len: usize,
    ) -> Self {
        assert!(high_cutoff > low_cutoff);
        let low_cutoff = low_cutoff / sample_freq;
        let high_cutoff = high_cutoff / sample_freq;
        let mid = (low_cutoff + high_cutoff) * 0.5;
        let bandwidth = high_cutoff - low_cutoff;
        let angular_mid = 2.0 * PI * mid;
        let r = 1.0 - 3.0 * bandwidth;
        let k = (1.0 - 2.0 * r * angular_mid.cos() + r * r) / (2.0 - 2.0 * angular_mid.cos());

        let in_weights = vec![k, 2.0 * (k - r) * angular_mid.cos(), r * r - k];
        let out_weights = vec![2.0 * r * angular_mid.cos(), -1.0 * r * r];

        Self::new(in_weights, out_weights, frame_len)
    }

    /// An IIR filter than doesn't affect the signal.
    pub fn passthru(frame_len: usize) -> Self {
        Self::single_pole(1.0, 0.0, 0.0, frame_len)
    }

    #[inline]
    fn zero_buffer(&mut self) {
        for s in self.buffer.iter_mut() {
            // zero out inter-frame buffer.
            *s = 0.0;
        }
    }

    #[inline]
    fn zero_output_buffer(&mut self) {
        for s in self.output_buffer.iter_mut() {
            // zero out inter-frame buffer.
            *s = 0.0;
        }
    }

    #[inline]
    fn add_to_output_buffer(&self, buf: &mut [f32]) {
        assert!(self.output_buffer.len() == buf.len());

        // todo make sure this uses memcpy
        for (in_s, out_s) in izip!(self.output_buffer.iter(), buf.iter_mut()) {
            *out_s += *in_s;
        }
    }
}

impl Effect for IIRFilter {
    fn apply(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());

        let filter_len = self.in_weights.len();
        let sample_len = input.len();
        assert!(sample_len > filter_len);
        assert!(filter_len > 0);

        // prepare output buffer
        if self.output_buffer.len() != output.len() {
            eprintln!("Warning: resizing internal output buffer in IIR filter");
            self.output_buffer.resize(output.len(), 0.0);
        }
        self.zero_output_buffer();

        // in = prev, out = this: we just include pre-calculated contributions
        for (output_sample, buf_sample) in izip!(self.output_buffer.iter_mut(), self.buffer.iter())
        {
            *output_sample += *buf_sample;
        }

        // in = this, out = this
        // idx is sample space
        for idx in 0..(sample_len - filter_len) {
            // Finish off this sample
            let sample_in = input[idx];
            self.output_buffer[idx] += sample_in * self.in_weights[0];
            let sample_out = self.output_buffer[idx];

            // Add in all the contributions to future samples from this sample.
            // contrib_idx in filter space
            for contrib_idx in 1..filter_len {
                let output_amt = sample_in * self.in_weights[contrib_idx]
                    + sample_out * self.out_weights[contrib_idx - 1];
                self.output_buffer[idx + contrib_idx] += output_amt;
            }
        }

        // in = this, out = this + next
        self.zero_buffer();
        // idx is in sample space
        for idx in (sample_len - filter_len)..sample_len {
            // Finish off this sample
            let sample_in = input[idx];
            self.output_buffer[idx] += sample_in * self.in_weights[0];
            let sample_out = self.output_buffer[idx];

            // contrib_idx is in sample space.
            let mut contrib_idx = idx + 1;
            // first add into this frame
            while contrib_idx < sample_len {
                // weight_idx is in filter space ( min 1, max sample_len - idx, which is less than
                // filter_len)
                let weight_idx = contrib_idx - idx;
                self.output_buffer[contrib_idx] = self.output_buffer[contrib_idx]
                    + sample_in * self.in_weights[weight_idx]
                    + sample_out * self.out_weights[weight_idx - 1];
                contrib_idx += 1;
            }
            // then add into the next frame via. the inter-frame buffer.
            while contrib_idx - (idx + 1) < filter_len - 1 {
                // buffer_idx is in filter space
                let buffer_idx = contrib_idx - (idx + 1);
                self.buffer[buffer_idx] = self.buffer[buffer_idx]
                    + sample_in * self.in_weights[buffer_idx + 1]
                    + sample_out * self.out_weights[buffer_idx];
                contrib_idx += 1;
            }
        }

        self.add_to_output_buffer(output);
    }
}

/// Inspired by the spectral processing engine in freqtweak.
pub struct SpectralEngine {
    oversample: usize,
    in_gain: f32,
    fft_length: usize,
    input: Vec<f32>,
    window: Vec<f32>,
    windowed_input: AlignedVec<f32>,
    signal_fft: AlignedVec<f32>,
    output_accum: Vec<f32>,
    fft_plan: R2RPlan32,
    ifft_plan: R2RPlan32,
    // low pass filter
    lpf: Vec<f32>,
    // message channel
    tx: channel::Sender<UiMsg>,
    audio_in_spectrum: MonitorData<Box<[f32]>>,
    audio_out_spectrum: MonitorData<Box<[f32]>>,
}

impl SpectralEngine {
    pub fn new(sample_rate: f32, fft_length: usize, tx: channel::Sender<UiMsg>) -> Self {
        let mut windowed_input = AlignedVec::new(fft_length);
        let mut signal_fft = AlignedVec::new(fft_length);
        let mut fft_plan: R2RPlan32 = R2RPlan::new(
            &[fft_length],
            &mut windowed_input,
            &mut signal_fft,
            R2RKind::FFTW_R2HC,
            Flag::ESTIMATE,
        )
        .unwrap();
        let ifft_plan = R2RPlan::new(
            &[fft_length],
            &mut signal_fft,
            // reuse this for output.
            &mut windowed_input,
            R2RKind::FFTW_HC2R,
            Flag::ESTIMATE,
        )
        .unwrap();
        // cheekily use our forward fft plan to process our filters
        low_pass_filter(800., sample_rate, &mut *windowed_input);
        fft_plan.r2r(&mut windowed_input, &mut signal_fft).unwrap();
        let lpf: Vec<_> = signal_fft.iter().copied().collect();
        tx.send(UiMsg::LowPassSpectrum(hc_to_mod(&lpf))).unwrap();
        Self {
            oversample: 4,
            in_gain: 1.0,
            fft_length,
            input: vec![0.; fft_length],
            window: (0..fft_length).map(|i| blackman(fft_length, i)).collect(),
            windowed_input,
            signal_fft,
            output_accum: vec![0.; fft_length * 2],
            fft_plan,
            ifft_plan,
            lpf,
            tx,
            audio_in_spectrum: MonitorData::new(vec![0.; fft_length].into_boxed_slice()),
            audio_out_spectrum: MonitorData::new(vec![0.; fft_length].into_boxed_slice()),
        }
    }

    pub fn step_size(&self) -> usize {
        self.fft_length / self.oversample
    }

    pub fn latency(&self) -> usize {
        self.fft_length - self.step_size()
    }

    /// Get handles on (`audio_in_spectrum`, `audio_out_spectrum`)
    pub fn monitor_spectra(&self) -> (MonitorSpectrum, MonitorSpectrum) {
        (
            self.audio_in_spectrum.clone(),
            self.audio_out_spectrum.clone(),
        )
    }

    pub fn process<S>(
        &mut self,
        input_rb: &mut Bounded<S>,
        output_rb: &mut Bounded<S>,
        report_spectra: bool,
    ) where
        S: Slice<Element = f32> + SliceMut,
    {
        let latency = self.latency();
        let step_size = self.step_size();
        // while there is enough data for another pass
        while input_rb.try_read(&mut self.input[latency..]).is_ok() {
            // apply window and input gain to the data.
            for (winput, input, window) in izip!(
                self.windowed_input.as_slice_mut(),
                &self.input,
                &self.window
            ) {
                *winput = *input * *window * self.in_gain;
            }
            // forward fft
            self.fft_plan
                .r2r(&mut self.windowed_input, &mut self.signal_fft)
                .unwrap();

            if report_spectra {
                self.audio_in_spectrum
                    .update(|data| data.copy_from_slice(&*self.signal_fft));
            }
            // TODO process
            //hc_multiply(&self.lpf, &mut self.signal_fft);
            //println!("{:?}", &*self.lpf);

            if report_spectra {
                self.tx
                    .send(UiMsg::AudioOutSpectrum(hc_to_mod(&self.signal_fft)))
                    .unwrap();
            }
            self.ifft_plan
                .r2r(&mut self.signal_fft, &mut self.windowed_input)
                .unwrap();

            // window & normalize the output
            for (accum, windowed_input, window) in izip!(
                &mut self.output_accum,
                self.windowed_input.as_slice(),
                &self.window
            ) {
                *accum += *windowed_input * *window / (self.fft_length as f32);
            }
            // write out output
            output_rb.extend(&self.output_accum[..step_size]);

            // shift internal buffers back by step_size
            // emulate memmove
            for i in 0..self.fft_length {
                self.output_accum[i] = self.output_accum[i + step_size];
            }
            for i in 0..latency {
                self.input[i] = self.input[i + step_size];
            }
        }
    }
}

/// returns the convolution kernel for a low pass filter
pub fn low_pass_filter(cutoff: f32, sample_freq: f32, buf: &mut [f32]) {
    let len = buf.len();
    let middle = (len as f32 + 1.0) * 0.5;

    let cutoff = cutoff / sample_freq;
    let angular_cutoff = 2.0 * PI * cutoff;

    // Accumulate total sum of blackman window, so we can normalize after.
    for i in 0..len {
        if i as f32 == middle {
            buf[i] = 2.0 * cutoff;
        } else {
            let i_f = i as f32 - middle;
            let bman = blackman(len, i);
            let weight = bman * (angular_cutoff * i_f).sin() / (PI * i_f);
            buf[i] = weight;
        }
    }
}

fn hamming(m: usize, i: usize) -> f32 {
    let r = i as f32 / m as f32;
    0.54 - 0.46 * (2.0 * PI * r).cos()
}

fn blackman(m: usize, i: usize) -> f32 {
    const ALPHA: f32 = 0.16;
    const A0: f32 = (1. - ALPHA) * 0.5;
    const A1: f32 = 0.5;
    const A2: f32 = ALPHA * 0.5;
    let r = i as f32 / m as f32;
    // I added the `0.9` fiddle factor to given equal power before and after spectral processing.
    (A0 - A1 * (2.0 * PI * r).cos() + A2 * (4.0 * PI * r).cos()) * 0.9
}

fn sinc(x: f32) -> f32 {
    x.sin() / x
}

/// Multiply two frames in fftw half-complex form.
///
/// After multiplication `input` will contain the output.
pub fn hc_multiply(params: &[f32], input: &mut [f32]) {
    assert_eq!(params.len(), input.len());
    let n = params.len();

    if n == 0 {
        return;
    }

    // real parts
    input[0] = input[0] * params[0];
    if n > 1 {
        for i in 1..=(n - 1) / 2 {
            let pre = params[i];
            let ire = input[i];
            let pim = params[n - i];
            let iim = input[n - i];
            input[i] = pre * ire - pim * iim;
            input[n - i] = pre * iim + ire * pim;
        }
        if n % 2 == 0 {
            let n2 = n / 2;
            input[n2] = input[n2] * params[n2];
        }
    }
}

pub fn hc_mod(input: &[f32], output: &mut [f32]) {
    assert_eq!(input.len() / 2 + 1, output.len());
    let n = input.len();

    if n == 0 {
        return;
    }

    output[0] = input[0].abs();
    if n > 1 {
        for i in 1..=(n - 1) / 2 {
            output[i] = (input[i].powi(2) + input[n - i].powi(2)).sqrt();
        }
        if n % 2 == 0 {
            let n2 = n / 2;
            output[n2] = input[n2].abs();
        }
    }
}

// allocates
pub fn hc_to_mod(input: &[f32]) -> Vec<f32> {
    let mut output = vec![0.; input.len() / 2 + 1];
    hc_mod(input, &mut output);
    output
}

#[cfg(test)]
mod test {
    #[test]
    fn hc_multiply() {
        use super::hc_multiply;
        for (expected, (params, mut input)) in vec![
            (&[1.][..], (&[1.][..], &mut [1.][..])),
            (&[1., 1.][..], (&[1., 1.][..], &mut [1., 1.][..])),
            (
                &[1., 0., 2.][..],
                (&[1., 1., 1.][..], &mut [1., 1., 1.][..]),
            ),
        ] {
            hc_multiply(&params, &mut input);
            assert_eq!(expected, input);
        }
    }

    #[test]
    fn hc_mod() {
        use super::hc_mod;
        for (expected, input) in vec![
            (&[1.][..], &[1.][..]),
            (&[1., 1.][..], &[1., 1.][..]),
            (&[1., 0.][..], &[1., 1., 1.][..]),
        ] {
            let mut output = vec![0.; expected.len()];
            hc_mod(&input, &mut output);
            assert_eq!(expected, output);
        }
    }
}
