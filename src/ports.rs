use crate::{
    effects::{Effect, FIRFilter},
    Msg, Result, State,
};
use crossbeam_channel as channel;
use jack::{AudioIn, AudioOut, Client, Control, MidiIn, Port, ProcessHandler, ProcessScope};
use novation_launch_control::Event;

/// This structure holds all the info we need to process the audio/midi signals in the realtime
/// thread.
pub struct Ports {
    in1_left: Port<AudioIn>,
    in1_right: Port<AudioIn>,
    in2_left: Port<AudioIn>,
    in2_right: Port<AudioIn>,
    in3_left: Port<AudioIn>,
    in3_right: Port<AudioIn>,
    in4_left: Port<AudioIn>,
    in4_right: Port<AudioIn>,
    out_left: Port<AudioOut>,
    out_right: Port<AudioOut>,
    control_in: Port<MidiIn>,
    ui_in: channel::Receiver<Msg>,
    ui_out: channel::Sender<Msg>,
    state: State,
    // filter coeffs
    low_filter_1l: FIRFilter,
    mid_filter_1l: FIRFilter,
    high_filter_1l: FIRFilter,
    low_filter_1r: FIRFilter,
    mid_filter_1r: FIRFilter,
    high_filter_1r: FIRFilter,
    low_filter_2l: FIRFilter,
    mid_filter_2l: FIRFilter,
    high_filter_2l: FIRFilter,
    low_filter_2r: FIRFilter,
    mid_filter_2r: FIRFilter,
    high_filter_2r: FIRFilter,
    low_filter_3l: FIRFilter,
    mid_filter_3l: FIRFilter,
    high_filter_3l: FIRFilter,
    low_filter_3r: FIRFilter,
    mid_filter_3r: FIRFilter,
    high_filter_3r: FIRFilter,
    low_filter_4l: FIRFilter,
    mid_filter_4l: FIRFilter,
    high_filter_4l: FIRFilter,
    low_filter_4r: FIRFilter,
    mid_filter_4r: FIRFilter,
    high_filter_4r: FIRFilter,
    // TODO the rest
}

impl Ports {
    /// Our constructor. Here we setup the ports we want and store them in our jack state object.
    pub fn setup(
        client: &Client,
        tx: channel::Sender<Msg>,
        rx: channel::Receiver<Msg>,
        low_mid_freq: f32,
        mid_high_freq: f32,
        filter_length: usize,
    ) -> Result<Ports> {
        let in1_left = client.register_port("in_1l", AudioIn)?;
        let in1_right = client.register_port("in_1r", AudioIn)?;
        let in2_left = client.register_port("in_2l", AudioIn)?;
        let in2_right = client.register_port("in_2r", AudioIn)?;
        let in3_left = client.register_port("in_3l", AudioIn)?;
        let in3_right = client.register_port("in_3r", AudioIn)?;
        let in4_left = client.register_port("in_4l", AudioIn)?;
        let in4_right = client.register_port("in_4r", AudioIn)?;

        let out_left = client.register_port("out_left", AudioOut)?;
        let out_right = client.register_port("out_right", AudioOut)?;

        let control_in = client.register_port("novation_SCXL_in", MidiIn)?;

        let low_pass =
            FIRFilter::low_pass(low_mid_freq, client.sample_rate() as f32, filter_length);
        let band_pass = FIRFilter::band_pass(
            low_mid_freq,
            mid_high_freq,
            client.sample_rate() as f32,
            filter_length,
        );
        let high_pass =
            FIRFilter::high_pass(mid_high_freq, client.sample_rate() as f32, filter_length);

        Ok(Ports {
            in1_left,
            in1_right,
            in2_left,
            in2_right,
            in3_left,
            in3_right,
            in4_left,
            in4_right,
            out_left,
            out_right,
            control_in,
            ui_out: tx,
            ui_in: rx,
            state: State::default(),
            low_filter_1l: low_pass.clone(),
            mid_filter_1l: band_pass.clone(),
            high_filter_1l: high_pass.clone(),
            low_filter_1r: low_pass.clone(),
            mid_filter_1r: band_pass.clone(),
            high_filter_1r: high_pass.clone(),
            low_filter_2l: low_pass.clone(),
            mid_filter_2l: band_pass.clone(),
            high_filter_2l: high_pass.clone(),
            low_filter_2r: low_pass.clone(),
            mid_filter_2r: band_pass.clone(),
            high_filter_2r: high_pass.clone(),
            low_filter_3l: low_pass.clone(),
            mid_filter_3l: band_pass.clone(),
            high_filter_3l: high_pass.clone(),
            low_filter_3r: low_pass.clone(),
            mid_filter_3r: band_pass.clone(),
            high_filter_3r: high_pass.clone(),
            low_filter_4l: low_pass.clone(),
            mid_filter_4l: band_pass.clone(),
            high_filter_4l: high_pass.clone(),
            low_filter_4r: low_pass.clone(),
            mid_filter_4r: band_pass.clone(),
            high_filter_4r: high_pass.clone(),
        })
    }
}

impl ProcessHandler for Ports {
    fn process(&mut self, _client: &Client, process_scope: &ProcessScope) -> Control {
        use channel::TryRecvError;

        let mut shutdown = false;
        // process midi events
        for raw_midi in self.control_in.iter(process_scope) {
            if let Some(evt) = Event::parse(raw_midi.bytes) {
                if let Some(msg) = convert_midi(evt) {
                    self.state.update(msg);
                    if let Err(e) = self.ui_out.send(msg) {
                        println!("Error communicating with ui: {}", e);
                        shutdown = true;
                    }
                }
            }
        }
        // process ui events
        loop {
            match self.ui_in.try_recv() {
                Ok(msg) => {
                    self.state.update(msg);
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    shutdown = true;
                    break;
                }
            }
        }

        // left
        // get params
        let gain_1_high = self.state.fader_1_1 as f32;
        let gain_1_mid = self.state.fader_2_1 as f32;
        let gain_1_low = self.state.fader_3_1 as f32;
        let gain_1 = self.state.fader_4_1 as f32;
        let gain_3_high = self.state.fader_1_3 as f32;
        let gain_3_mid = self.state.fader_2_3 as f32;
        let gain_3_low = self.state.fader_3_3 as f32;
        let gain_3 = self.state.fader_4_3 as f32;
        let gain_5_high = self.state.fader_1_5 as f32;
        let gain_5_mid = self.state.fader_2_5 as f32;
        let gain_5_low = self.state.fader_3_5 as f32;
        let gain_5 = self.state.fader_4_5 as f32;
        let gain_7_high = self.state.fader_1_7 as f32;
        let gain_7_mid = self.state.fader_2_7 as f32;
        let gain_7_low = self.state.fader_3_7 as f32;
        let gain_7 = self.state.fader_4_7 as f32;

        self.high_filter_1l.set_gain(gain_1_high * gain_1);
        self.mid_filter_1l.set_gain(gain_1_mid * gain_1);
        self.low_filter_1l.set_gain(gain_1_low * gain_1);
        self.high_filter_2l.set_gain(gain_3_high * gain_3);
        self.mid_filter_2l.set_gain(gain_3_mid * gain_3);
        self.low_filter_2l.set_gain(gain_3_low * gain_3);
        self.high_filter_3l.set_gain(gain_5_high * gain_5);
        self.mid_filter_3l.set_gain(gain_5_mid * gain_5);
        self.low_filter_3l.set_gain(gain_5_low * gain_5);
        self.high_filter_4l.set_gain(gain_7_high * gain_7);
        self.mid_filter_4l.set_gain(gain_7_mid * gain_7);
        self.low_filter_4l.set_gain(gain_7_low * gain_7);

        let in1_left = self.in1_left.as_slice(process_scope);
        let in2_left = self.in2_left.as_slice(process_scope);
        let in3_left = self.in3_left.as_slice(process_scope);
        let in4_left = self.in4_left.as_slice(process_scope);
        let out_left = self.out_left.as_mut_slice(process_scope);
        // todo check if this is necessary, I think it is.
        for v in out_left.iter_mut() {
            *v = 0.0;
        }

        // these checks see if we can short-circuit the filter.
        if !(gain_1 == 0.0) {
            if self.state.filter_passthru_1 {
                for (out_s, in_s) in out_left.iter_mut().zip(in1_left.iter()) {
                    *out_s += *in_s * gain_1;
                }
            } else {
                Effect::apply(&mut self.low_filter_1l, in1_left, out_left);
                Effect::apply(&mut self.mid_filter_1l, in1_left, out_left);
                Effect::apply(&mut self.high_filter_1l, in1_left, out_left);
            }
        }
        if !(gain_3 == 0.0) {
            if self.state.filter_passthru_3 {
                for (out_s, in_s) in out_left.iter_mut().zip(in2_left.iter()) {
                    *out_s += *in_s * gain_3;
                }
            } else {
                Effect::apply(&mut self.low_filter_2l, in2_left, out_left);
                Effect::apply(&mut self.mid_filter_2l, in2_left, out_left);
                Effect::apply(&mut self.high_filter_2l, in2_left, out_left);
            }
        }
        if !(gain_5 == 0.0) {
            if self.state.filter_passthru_5 {
                for (out_s, in_s) in out_left.iter_mut().zip(in3_left.iter()) {
                    *out_s += *in_s * gain_5;
                }
            } else {
                Effect::apply(&mut self.low_filter_3l, in3_left, out_left);
                Effect::apply(&mut self.mid_filter_3l, in3_left, out_left);
                Effect::apply(&mut self.high_filter_3l, in3_left, out_left);
            }
        }
        if !(gain_7 == 0.0) {
            if self.state.filter_passthru_7 {
                for (out_s, in_s) in out_left.iter_mut().zip(in4_left.iter()) {
                    *out_s += *in_s * gain_7;
                }
            } else {
                Effect::apply(&mut self.low_filter_4l, in4_left, out_left);
                Effect::apply(&mut self.mid_filter_4l, in4_left, out_left);
                Effect::apply(&mut self.high_filter_4l, in4_left, out_left);
            }
        }

        // right
        // gat params
        let gain_2_high = self.state.fader_1_2 as f32;
        let gain_2_mid = self.state.fader_2_2 as f32;
        let gain_2_low = self.state.fader_3_2 as f32;
        let gain_2 = self.state.fader_4_2 as f32;
        let gain_4_high = self.state.fader_1_4 as f32;
        let gain_4_mid = self.state.fader_2_4 as f32;
        let gain_4_low = self.state.fader_3_4 as f32;
        let gain_4 = self.state.fader_4_4 as f32;
        let gain_6_high = self.state.fader_1_6 as f32;
        let gain_6_mid = self.state.fader_2_6 as f32;
        let gain_6_low = self.state.fader_3_6 as f32;
        let gain_6 = self.state.fader_4_6 as f32;
        let gain_8_high = self.state.fader_1_8 as f32;
        let gain_8_mid = self.state.fader_2_8 as f32;
        let gain_8_low = self.state.fader_3_8 as f32;
        let gain_8 = self.state.fader_4_8 as f32;

        self.high_filter_1r.set_gain(gain_2_high * gain_2);
        self.mid_filter_1r.set_gain(gain_2_mid * gain_2);
        self.low_filter_1r.set_gain(gain_2_low * gain_2);
        self.high_filter_2r.set_gain(gain_4_high * gain_4);
        self.mid_filter_2r.set_gain(gain_4_mid * gain_4);
        self.low_filter_2r.set_gain(gain_4_low * gain_4);
        self.high_filter_3r.set_gain(gain_6_high * gain_6);
        self.mid_filter_3r.set_gain(gain_6_mid * gain_6);
        self.low_filter_3r.set_gain(gain_6_low * gain_6);
        self.high_filter_4r.set_gain(gain_8_high * gain_8);
        self.mid_filter_4r.set_gain(gain_8_mid * gain_8);
        self.low_filter_4r.set_gain(gain_8_low * gain_8);

        let in1_right = self.in1_right.as_slice(process_scope);
        let in2_right = self.in2_right.as_slice(process_scope);
        let in3_right = self.in3_right.as_slice(process_scope);
        let in4_right = self.in4_right.as_slice(process_scope);
        let out_right = self.out_right.as_mut_slice(process_scope);
        for v in out_right.iter_mut() {
            *v = 0.0;
        }

        // these checks see if we can short-circuit the filter.
        if !(gain_2 == 0.0) {
            if self.state.filter_passthru_2 {
                for (out_s, in_s) in out_right.iter_mut().zip(in1_right.iter()) {
                    *out_s += *in_s * gain_2;
                }
            } else {
                Effect::apply(&mut self.low_filter_1r, in1_right, out_right);
                Effect::apply(&mut self.mid_filter_1r, in1_right, out_right);
                Effect::apply(&mut self.high_filter_1r, in1_right, out_right);
            }
        }
        if !(gain_4 == 0.0) {
            if self.state.filter_passthru_4 {
                for (out_s, in_s) in out_right.iter_mut().zip(in2_right.iter()) {
                    *out_s += *in_s * gain_4;
                }
            } else {
                Effect::apply(&mut self.low_filter_2r, in2_right, out_right);
                Effect::apply(&mut self.mid_filter_2r, in2_right, out_right);
                Effect::apply(&mut self.high_filter_2r, in2_right, out_right);
            }
        }
        if !(gain_6 == 0.0) {
            if self.state.filter_passthru_6 {
                for (out_s, in_s) in out_right.iter_mut().zip(in3_right.iter()) {
                    *out_s += *in_s * gain_6;
                }
            } else {
                Effect::apply(&mut self.low_filter_3r, in3_right, out_right);
                Effect::apply(&mut self.mid_filter_3r, in3_right, out_right);
                Effect::apply(&mut self.high_filter_3r, in3_right, out_right);
            }
        }
        if !(gain_8 == 0.0) {
            if self.state.filter_passthru_8 {
                for (out_s, in_s) in out_right.iter_mut().zip(in4_right.iter()) {
                    *out_s += *in_s * gain_8;
                }
            } else {
                Effect::apply(&mut self.low_filter_4r, in4_right, out_right);
                Effect::apply(&mut self.mid_filter_4r, in4_right, out_right);
                Effect::apply(&mut self.high_filter_4r, in4_right, out_right);
            }
        }

        if shutdown {
            Control::Quit
        } else {
            Control::Continue
        }
    }
}

// utils

fn convert_midi(evt: Event) -> Option<Msg> {
    Some(match evt {
        Event::Fader1_1(v) => Msg::fader_1_1(v as f64),
        Event::Fader1_2(v) => Msg::fader_1_2(v as f64),
        Event::Fader1_3(v) => Msg::fader_1_3(v as f64),
        Event::Fader1_4(v) => Msg::fader_1_4(v as f64),
        Event::Fader1_5(v) => Msg::fader_1_5(v as f64),
        Event::Fader1_6(v) => Msg::fader_1_6(v as f64),
        Event::Fader1_7(v) => Msg::fader_1_7(v as f64),
        Event::Fader1_8(v) => Msg::fader_1_8(v as f64),
        Event::Fader2_1(v) => Msg::fader_2_1(v as f64),
        Event::Fader2_2(v) => Msg::fader_2_2(v as f64),
        Event::Fader2_3(v) => Msg::fader_2_3(v as f64),
        Event::Fader2_4(v) => Msg::fader_2_4(v as f64),
        Event::Fader2_5(v) => Msg::fader_2_5(v as f64),
        Event::Fader2_6(v) => Msg::fader_2_6(v as f64),
        Event::Fader2_7(v) => Msg::fader_2_7(v as f64),
        Event::Fader2_8(v) => Msg::fader_2_8(v as f64),
        Event::Fader3_1(v) => Msg::fader_3_1(v as f64),
        Event::Fader3_2(v) => Msg::fader_3_2(v as f64),
        Event::Fader3_3(v) => Msg::fader_3_3(v as f64),
        Event::Fader3_4(v) => Msg::fader_3_4(v as f64),
        Event::Fader3_5(v) => Msg::fader_3_5(v as f64),
        Event::Fader3_6(v) => Msg::fader_3_6(v as f64),
        Event::Fader3_7(v) => Msg::fader_3_7(v as f64),
        Event::Fader3_8(v) => Msg::fader_3_8(v as f64),
        Event::Fader4_1(v) => Msg::fader_4_1(v as f64),
        Event::Fader4_2(v) => Msg::fader_4_2(v as f64),
        Event::Fader4_3(v) => Msg::fader_4_3(v as f64),
        Event::Fader4_4(v) => Msg::fader_4_4(v as f64),
        Event::Fader4_5(v) => Msg::fader_4_5(v as f64),
        Event::Fader4_6(v) => Msg::fader_4_6(v as f64),
        Event::Fader4_7(v) => Msg::fader_4_7(v as f64),
        Event::Fader4_8(v) => Msg::fader_4_8(v as f64),
        _ => return None,
    })
}
