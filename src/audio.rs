use crate::{
    cli::{Config, MidiEffect, MidiEffectKind, MidiKey, MidiLookup},
    data::Metering,
    gui::{Level, UiMsg},
    Result,
};
use crossbeam_channel as channel;
use itertools::izip;
use jack::{
    AudioIn, AudioOut, Client, Control, MidiIn, MidiOut, MidiWriter, Port, ProcessHandler,
    ProcessScope,
};
use midi_event::{Event, MidiEvent, MidiEventType, Note, Parse};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

macro_rules! handle_error {
    ($inner:expr, $shutdown:expr, $err_msg:expr) => {
        match $inner {
            Ok(v) => v,
            Err(err) => {
                println!(concat!($err_msg, ": {}"), err);
                $shutdown = true;
            }
        }
    };
}

macro_rules! opt_continue {
    ($value:expr) => {
        match $value {
            Some(v) => v,
            None => continue,
        }
    };
}

/// This structure holds all the info we need to process the audio/midi signals in the realtime
/// thread.
pub struct Audio {
    // audio in ports
    ports_in: Vec<Port<AudioIn>>,
    // audio out ports
    ports_out: Vec<Port<AudioOut>>,
    // midi ports
    control_in: Port<MidiIn>,
    control_out: Port<MidiOut>,

    ui_in: channel::Receiver<AudioMsg>,
    ui_out: channel::Sender<UiMsg>,
    // application state
    state: State,

    frame_len: usize,
    // We want to accumulate metering info so we only send it once every 1/60 second.
    frames_in_meter_frame: usize,
    frames_acc: usize,
    meter_accs: Vec<MeterAcc>,

    //first_iter: bool,
    //novation_out: NovationOut,
    midi_lookup: MidiLookup,
}

impl Audio {
    /// Our constructor. Here we setup the ports we want and store them in our jack state object.
    pub fn setup(
        config: &Config,
        client: &Client,
        tx: channel::Sender<UiMsg>,
        rx: channel::Receiver<AudioMsg>,
        low_mid_freq: f32,
        mid_high_freq: f32,
        filter_length: usize,
    ) -> Result<Audio> {
        let sample_rate = client.sample_rate() as f32;
        let frame_len = usize::try_from(client.buffer_size()).unwrap();

        // Create ports
        let mut ports_in = vec![];
        let mut ports_out = vec![];
        for (chan_name, chan) in config.channels.iter() {
            ports_in.push(client.register_port(&format!("{} in", chan_name), AudioIn)?);
            ports_out.push(client.register_port(&format!("{} out", chan_name), AudioOut)?);
        }

        let control_in = client.register_port("control_in", MidiIn)?;
        let control_out = client.register_port("control_out", MidiOut)?;

        // frames in a second / 60
        let frames_in_meter_frame =
            ((sample_rate as f64 / frame_len as f64) / 60.).floor() as usize;

        Ok(Audio {
            ports_in,
            ports_out,
            control_in,
            control_out,
            ui_out: tx,
            ui_in: rx,
            state: State::new(config),
            frame_len,
            frames_in_meter_frame,
            frames_acc: 0,
            meter_accs: vec![MeterAcc::new(); config.channels.len()],
            //first_iter: true,
            //novation_out: NovationOut::new(),
            midi_lookup: config.midi_lookup(),
        })
    }
}

impl ProcessHandler for Audio {
    fn process(&mut self, _client: &Client, ps: &ProcessScope) -> Control {
        use channel::TryRecvError;

        let mut shutdown = false;

        // reset the controller on the first cycle
        let mut control_out = self.control_out.writer(ps);
        /*
        if self.first_iter {
            handle_error!(
                self.novation_out.reset(&mut control_out),
                shutdown,
                "error resetting LCXL state"
            );
            self.first_iter = false;
        }
        */

        // process midi events
        for raw_midi in self.control_in.iter(ps) {
            if let Some(evt) = MidiEvent::parse(raw_midi.bytes) {
                #[inline]
                fn get_value(evt: &MidiEvent) -> u8 {
                    use midi_event::MidiEventType::{Controller, NoteOn};
                    match evt.event {
                        Controller(_, gain) => gain,
                        NoteOn(_, gain) => gain,
                        _ => unreachable!(),
                    }
                }
                let key = opt_continue!(MidiKey::from_opt(evt));
                let effect = opt_continue!(self.midi_lookup.get(&key));
                match effect.kind {
                    MidiEffectKind::Gain => {
                        let gain = (get_value(&evt) as f64) / 127.0;
                        self.state.channels[effect.channel].gain = gain;
                        handle_error!(
                            self.ui_out.send(UiMsg::Levels {
                                channel: effect.channel,
                                level: Level::Gain(gain)
                            }),
                            shutdown,
                            "error communicating with ui"
                        );
                    }
                }
            }
        }

        // process ui events
        loop {
            match self.ui_in.try_recv() {
                Ok(msg) => {
                    /*
                    handle_error!(
                        self.novation_out
                            .handle_msg(&self.state, msg, &mut control_out),
                        shutdown,
                        "Error updating LCXL state"
                    );
                    */
                    self.state.update(msg);
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    shutdown = true;
                    break;
                }
            }
        }

        for (idx, (chan_in, mut chan_out, ctrl, mut meter_acc)) in izip!(
            &self.ports_in,
            &mut self.ports_out,
            &self.state.channels,
            &mut self.meter_accs
        )
        .enumerate()
        {
            for (out_s, in_s) in izip!(
                chan_out.as_mut_slice(ps).iter_mut(),
                chan_in.as_slice(ps).iter()
            ) {
                // calculate out signal
                *out_s = *in_s * (ctrl.gain as f32);
                // metering
                meter_acc.sample(*out_s);
            }
        }

        self.frames_acc += 1;
        if self.frames_acc >= self.frames_in_meter_frame {
            // Report metering
            for (idx, mut meter_acc) in self.meter_accs.iter_mut().enumerate() {
                handle_error!(
                    self.ui_out.send(UiMsg::Metering {
                        channel: idx,
                        metering: meter_acc.as_metering(self.frames_acc * self.frame_len),
                    }),
                    shutdown,
                    "error communicating with ui"
                );
                meter_acc.clear()
            }
            self.frames_acc = 0;
        }

        if shutdown {
            Control::Quit
        } else {
            Control::Continue
        }
    }
}

// utils

/*
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
        _state: &State,
        msg: StateChange,
        out: &mut MidiWriter<'_>,
    ) -> Result<(), jack::Error> {
        self.set_template(0x08);
        match msg {
            StateChange::filter_passthru_1(v) => {
                if v {
                    self.set_off_led();
                } else {
                    self.set_red_led();
                };
                self.write_strip(0x00, out)?;
            }
            StateChange::filter_passthru_2(v) => {
                if v {
                    self.set_off_led();
                } else {
                    self.set_red_led();
                };
                self.write_strip(0x01, out)?;
            }
            StateChange::filter_passthru_3(v) => {
                if v {
                    self.set_off_led();
                } else {
                    self.set_red_led();
                };
                self.write_strip(0x02, out)?;
            }
            StateChange::filter_passthru_4(v) => {
                if v {
                    self.set_off_led();
                } else {
                    self.set_red_led();
                };
                self.write_strip(0x03, out)?;
            }
            StateChange::filter_passthru_5(v) => {
                if v {
                    self.set_off_led();
                } else {
                    self.set_red_led();
                };
                self.write_strip(0x04, out)?;
            }
            StateChange::filter_passthru_6(v) => {
                if v {
                    self.set_off_led();
                } else {
                    self.set_red_led();
                };
                self.write_strip(0x05, out)?;
            }
            StateChange::filter_passthru_7(v) => {
                if v {
                    self.set_off_led();
                } else {
                    self.set_red_led();
                };
                self.write_strip(0x06, out)?;
            }
            StateChange::filter_passthru_8(v) => {
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
*/

// State

#[derive(Debug, Clone, PartialEq)]
pub struct State {
    pub channels: Vec<ChannelState>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelState {
    pub gain: f64,
}

impl State {
    pub fn new(config: &Config) -> Self {
        let mut channels = Vec::with_capacity(config.channels.len());
        for (name, channel) in config.channels.iter() {
            channels.push(ChannelState { gain: 0.0 });
        }
        State { channels }
    }

    pub fn update(&mut self, msg: AudioMsg) {
        self.channels[msg.channel].gain = msg.gain;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AudioMsg {
    pub channel: usize,
    // just gain for now
    pub gain: f64,
}

/// A struct that accumulates metering info during a frame.
#[derive(Debug, Copy, Clone)]
pub struct MeterAcc {
    sum_squares: f32,
    max: f32,
}

impl MeterAcc {
    fn new() -> Self {
        MeterAcc {
            sum_squares: 0.0,
            max: 0.0,
        }
    }

    fn sample(&mut self, val: f32) {
        if self.max < val.abs() {
            self.max = val.abs();
        }
        self.sum_squares += val * val;
    }

    fn as_metering(&self, count: usize) -> Metering {
        Metering::new(
            self.max as f64,
            (self.sum_squares as f64 / count as f64).sqrt(),
        )
    }

    fn clear(&mut self) {
        self.max = 0.0;
        self.sum_squares = 0.0;
    }
}
