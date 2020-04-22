use crate::{
    effects::{Effect, FIRFilter},
    Msg, Result, State,
};
use crossbeam_channel as channel;
use jack::{
    AudioIn, AudioOut, Client, Control, MidiIn, MidiOut, MidiWriter, Port, ProcessHandler,
    ProcessScope,
};
use midi_event::{MidiEvent, Parse};

/// This structure holds all the info we need to process the audio/midi signals in the realtime
/// thread.
pub struct Ports {
    // audio ports
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
    // midi ports
    control_in: Port<MidiIn>,
    control_out: Port<MidiOut>,

    ui_in: channel::Receiver<Msg>,
    ui_out: channel::Sender<Msg>,
    // application state
    state: State,
    // filter coeffs - fixed
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

    first_iter: bool,
    novation_out: NovationOut,
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

        let control_in = client.register_port("control_in", MidiIn)?;
        let control_out = client.register_port("novation_SCXL_out", MidiOut)?;

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
            control_out,
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
            first_iter: true,
            novation_out: NovationOut::new(),
        })
    }
}

impl ProcessHandler for Ports {
    fn process(&mut self, _client: &Client, process_scope: &ProcessScope) -> Control {
        use channel::TryRecvError;

        let mut shutdown = false;

        // reset the controller on the first cycle
        let mut control_out = self.control_out.writer(process_scope);
        if self.first_iter {
            if let Err(e) = self.novation_out.reset(&mut control_out) {
                println!("Error resetting LCXL state: {}", e);
                shutdown = true;
            }
            self.first_iter = false;
        }

        // process midi events
        for raw_midi in self.control_in.iter(process_scope) {
            if let Some(evt) = MidiEvent::parse(raw_midi.bytes) {
                if let Some(msg) = convert_midi(evt, &self.state) {
                    if let Err(e) = self
                        .novation_out
                        .handle_msg(&self.state, msg, &mut control_out)
                    {
                        println!("Error updating LCXL state: {}", e);
                        shutdown = true;
                    }
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
                    if let Err(e) = self
                        .novation_out
                        .handle_msg(&self.state, msg, &mut control_out)
                    {
                        println!("Error updating LCXL state: {}", e);
                        shutdown = true;
                    }
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

// for now mapping from novation soundcontrol xl factory settings 1
fn convert_midi(evt: MidiEvent, state: &State) -> Option<Msg> {
    use midi_event::{MidiEventType::*, Note::*};

    Some(match evt.event {
        Controller(0x0d, amt) => Msg::fader_1_1((amt as f64) / 127.0),
        Controller(0x0e, amt) => Msg::fader_1_2((amt as f64) / 127.0),
        Controller(0x0f, amt) => Msg::fader_1_3((amt as f64) / 127.0),
        Controller(0x10, amt) => Msg::fader_1_4((amt as f64) / 127.0),
        Controller(0x11, amt) => Msg::fader_1_5((amt as f64) / 127.0),
        Controller(0x12, amt) => Msg::fader_1_6((amt as f64) / 127.0),
        Controller(0x13, amt) => Msg::fader_1_7((amt as f64) / 127.0),
        Controller(0x14, amt) => Msg::fader_1_8((amt as f64) / 127.0),

        Controller(0x1d, amt) => Msg::fader_2_1((amt as f64) / 127.0),
        Controller(0x1e, amt) => Msg::fader_2_2((amt as f64) / 127.0),
        Controller(0x1f, amt) => Msg::fader_2_3((amt as f64) / 127.0),
        Controller(0x20, amt) => Msg::fader_2_4((amt as f64) / 127.0),
        Controller(0x21, amt) => Msg::fader_2_5((amt as f64) / 127.0),
        Controller(0x22, amt) => Msg::fader_2_6((amt as f64) / 127.0),
        Controller(0x23, amt) => Msg::fader_2_7((amt as f64) / 127.0),
        Controller(0x24, amt) => Msg::fader_2_8((amt as f64) / 127.0),

        Controller(0x31, amt) => Msg::fader_3_1((amt as f64) / 127.0),
        Controller(0x32, amt) => Msg::fader_3_2((amt as f64) / 127.0),
        Controller(0x33, amt) => Msg::fader_3_3((amt as f64) / 127.0),
        Controller(0x34, amt) => Msg::fader_3_4((amt as f64) / 127.0),
        Controller(0x35, amt) => Msg::fader_3_5((amt as f64) / 127.0),
        Controller(0x36, amt) => Msg::fader_3_6((amt as f64) / 127.0),
        Controller(0x37, amt) => Msg::fader_3_7((amt as f64) / 127.0),
        Controller(0x38, amt) => Msg::fader_3_8((amt as f64) / 127.0),

        Controller(0x4d, amt) => Msg::fader_4_1((amt as f64) / 127.0),
        Controller(0x4e, amt) => Msg::fader_4_2((amt as f64) / 127.0),
        Controller(0x4f, amt) => Msg::fader_4_3((amt as f64) / 127.0),
        Controller(0x50, amt) => Msg::fader_4_4((amt as f64) / 127.0),
        Controller(0x51, amt) => Msg::fader_4_5((amt as f64) / 127.0),
        Controller(0x52, amt) => Msg::fader_4_6((amt as f64) / 127.0),
        Controller(0x53, amt) => Msg::fader_4_7((amt as f64) / 127.0),
        Controller(0x54, amt) => Msg::fader_4_8((amt as f64) / 127.0),

        NoteOn(F2, _) => Msg::filter_passthru_1(!state.filter_passthru_1),
        NoteOn(Fs2, _) => Msg::filter_passthru_2(!state.filter_passthru_2),
        NoteOn(G2, _) => Msg::filter_passthru_3(!state.filter_passthru_3),
        NoteOn(Gs2, _) => Msg::filter_passthru_4(!state.filter_passthru_4),
        NoteOn(A3, _) => Msg::filter_passthru_5(!state.filter_passthru_5),
        NoteOn(As3, _) => Msg::filter_passthru_6(!state.filter_passthru_6),
        NoteOn(B3, _) => Msg::filter_passthru_7(!state.filter_passthru_7),
        NoteOn(C4, _) => Msg::filter_passthru_8(!state.filter_passthru_8),
        _ => return None,
    })
}

pub struct NovationOut {
    buf: [u8; 11],
}

impl NovationOut {
    fn new() -> Self {
        NovationOut {
            buf: [
                0xf0, 0x00, 0x20, 0x29, 0x02, 0x11, 0x78, 0x00, 0x00, 0x00, 0xf7,
            ],
        }
    }

    fn handle_msg(
        &mut self,
        state: &State,
        msg: Msg,
        out: &mut MidiWriter<'_>,
    ) -> Result<(), jack::Error> {
        self.set_template(0x08);
        match msg {
            Msg::filter_passthru_1(v) => {
                if v {
                    self.set_off_led();
                } else {
                    self.set_red_led();
                };
                self.write_strip(0x00, out)?;
            }
            Msg::filter_passthru_2(v) => {
                if v {
                    self.set_off_led();
                } else {
                    self.set_red_led();
                };
                self.write_strip(0x01, out)?;
            }
            Msg::filter_passthru_3(v) => {
                if v {
                    self.set_off_led();
                } else {
                    self.set_red_led();
                };
                self.write_strip(0x02, out)?;
            }
            Msg::filter_passthru_4(v) => {
                if v {
                    self.set_off_led();
                } else {
                    self.set_red_led();
                };
                self.write_strip(0x03, out)?;
            }
            Msg::filter_passthru_5(v) => {
                if v {
                    self.set_off_led();
                } else {
                    self.set_red_led();
                };
                self.write_strip(0x04, out)?;
            }
            Msg::filter_passthru_6(v) => {
                if v {
                    self.set_off_led();
                } else {
                    self.set_red_led();
                };
                self.write_strip(0x05, out)?;
            }
            Msg::filter_passthru_7(v) => {
                if v {
                    self.set_off_led();
                } else {
                    self.set_red_led();
                };
                self.write_strip(0x06, out)?;
            }
            Msg::filter_passthru_8(v) => {
                if v {
                    self.set_off_led();
                } else {
                    self.set_red_led();
                };
                self.write_strip(0x07, out)?;
            }
            _ => (),
        }
        Ok(())
    }

    #[inline(always)]
    fn set_green_led(&mut self) {
        self.buf[9] = 0b0011_1100
    }

    #[inline(always)]
    fn set_red_led(&mut self) {
        self.buf[9] = 0b0000_1111
    }

    #[inline(always)]
    fn set_off_led(&mut self) {
        self.buf[9] = 0b0000_1100
    }

    #[inline(always)]
    fn set_template(&mut self, template: u8) {
        self.buf[7] = template;
    }

    #[inline(always)]
    fn set_index(&mut self, index: u8) {
        self.buf[8] = index;
    }

    #[inline]
    fn write_strip(&mut self, strip: u8, writer: &mut MidiWriter<'_>) -> Result<(), jack::Error> {
        self.set_index(strip);
        self.write_current(writer)?;
        self.set_index(strip + 0x8);
        self.write_current(writer)?;
        self.set_index(strip + 0x10);
        self.write_current(writer)?;
        self.set_index(strip + 0x18);
        self.write_current(writer)?;
        Ok(())
    }

    fn reset(&mut self, writer: &mut MidiWriter<'_>) -> Result<(), jack::Error> {
        writer.write(&jack::RawMidi {
            time: 0,
            bytes: &[0xb8, 0x00, 0x00],
        })
    }

    #[inline(always)]
    fn write_current(&self, writer: &mut MidiWriter<'_>) -> Result<(), jack::Error> {
        writer.write(&jack::RawMidi {
            time: 0,
            bytes: &self.buf,
        })
    }
}
