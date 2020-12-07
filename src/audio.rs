use crate::{
    cli::{Config, MidiEffect, MidiEffectKind, MidiKey, MidiLookup},
    data::{ChannelMode, Metering},
    effects::{MonitorSpectrum, SpectralEngine},
    gui::{Level, UiMsg},
    Result,
};
use crossbeam_channel as channel;
use dasp::ring_buffer::Bounded;
use itertools::izip;
use jack::{
    AudioIn, AudioOut, Client, Control, Frames, LatencyType, MidiIn, MidiOut, MidiWriter,
    NotificationHandler, Port, ProcessHandler, ProcessScope,
};
use midi_event::{Event, MidiEvent, MidiEventType, Note, Parse};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, convert::TryFrom, sync::Arc};

mod info;

pub use info::Info;

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

const FFI_LEN: usize = 512;

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

    // Because working in the frequency domain necessitates windowing and therefore latency, we use
    // single-threaded ringbuffers to store incoming/outgoing audio data between frames, as
    // necessary.
    in_bufs: Vec<Bounded<Vec<f32>>>,
    out_bufs: Vec<Bounded<Vec<f32>>>,
    specs: Vec<SpectralEngine>,

    // Channels for communicating with UI.
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
    ) -> Result<Audio> {
        let sample_rate = client.sample_rate() as f32;
        let frame_len = usize::try_from(client.buffer_size()).unwrap();

        // Create ports
        let mut ports_in = Vec::with_capacity(config.channels.len());
        let mut ports_out = Vec::with_capacity(config.channels.len());
        let mut in_bufs = Vec::with_capacity(config.channels.len());
        let mut out_bufs = Vec::with_capacity(config.channels.len());
        let mut specs = Vec::with_capacity(config.channels.len());
        for (chan_name, chan) in config.channels.iter() {
            ports_in.push(client.register_port(&format!("{} in", chan_name), AudioIn)?);
            ports_out.push(client.register_port(&format!("{} out", chan_name), AudioOut)?);
            // loose bound
            let in_buf = Bounded::from(vec![0.0f32; (frame_len * 2).max(1024)]);
            let out_buf = Bounded::from(vec![0.0f32; (frame_len * 2).max(1024)]);
            in_bufs.push(in_buf);
            out_bufs.push(out_buf);
            specs.push(SpectralEngine::new(sample_rate, FFI_LEN, tx.clone()));
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
            in_bufs,
            out_bufs,
            specs,
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

impl Audio {
    pub fn monitor_spectra(&self) -> Vec<(MonitorSpectrum, MonitorSpectrum)> {
        self.specs.iter().map(|s| s.monitor_spectra()).collect()
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

        // process events from ui
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

        // process audio
        // =============

        for (
            idx,
            (
                chan_in,
                mut chan_out,
                mut in_buf,
                mut out_buf,
                mut engine,
                mut chan_info,
                mut meter_acc,
            ),
        ) in izip!(
            &self.ports_in,
            &mut self.ports_out,
            &mut self.in_bufs,
            &mut self.out_bufs,
            &mut self.specs,
            &self.state.channels,
            &mut self.meter_accs
        )
        .enumerate()
        {
            // meter input
            for in_s in chan_in.as_slice(ps).iter() {
                meter_acc.sample_in(*in_s);
            }

            match chan_info.mode {
                ChannelMode::Mute => {
                    // TODO think about whether we have old audio data in the buffers, and whether
                    // this afffects this channel when it's turned back on.
                    // todo add zeros to meter
                    for v in chan_out.as_mut_slice(ps) {
                        *v = 0.;
                    }
                    continue;
                }
                ChannelMode::Bypass => {
                    chan_out
                        .as_mut_slice(ps)
                        .copy_from_slice(chan_in.as_slice(ps));

                    // copy metering from in to out
                    for in_s in chan_in.as_slice(ps).iter() {
                        meter_acc.sample_out(*in_s);
                    }
                    continue;
                }
                // fall thru
                ChannelMode::Normal => (),
            }
            // copy input to ring buffer
            in_buf.extend(chan_in.as_slice(ps));

            engine.process(in_buf, out_buf, idx == 0);

            let data_out = chan_out.as_mut_slice(ps);
            let mut idx = 0;
            while let Some(sample) = out_buf.pop() {
                data_out[idx] = sample * chan_info.gain as f32;
                idx += 1;
                if idx >= data_out.len() {
                    assert_eq!(out_buf.len(), 0);
                    break;
                }
            }

            // meter input
            for out_s in chan_out.as_mut_slice(ps).iter() {
                meter_acc.sample_out(*out_s);
            }
        }

        // process info for UI (metering)
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

impl NotificationHandler for Audio {
    fn sample_rate(&mut self, _: &Client, _: Frames) -> Control {
        // unsupported for now
        Control::Quit
    }

    fn buffer_size(&mut self, _: &Client, size: Frames) -> Control {
        // TODO ensure buffers are large enough. If not, reallocate them in one go.
        Control::Continue
    }

    fn latency(&mut self, client: &Client, mode: LatencyType) {
        match mode {
            LatencyType::Capture => {
                for port in self.ports_in.iter() {
                    let (mut min, mut max) = port.get_latency_range(LatencyType::Capture);
                    min += 0;
                    max += 0;
                    port.set_latency_range(LatencyType::Capture, (min, max));
                }
            }
            LatencyType::Playback => {
                for port in self.ports_out.iter() {
                    let (mut min, mut max) = port.get_latency_range(LatencyType::Playback);
                    min += 0;
                    max += 0;
                    port.set_latency_range(LatencyType::Playback, (min, max));
                }
            }
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
    pub mode: ChannelMode,
}

impl State {
    pub fn new(config: &Config) -> Self {
        let mut channels = Vec::with_capacity(config.channels.len());
        for (name, channel) in config.channels.iter() {
            channels.push(ChannelState {
                gain: 0.0,
                mode: ChannelMode::default(),
            });
        }
        State { channels }
    }

    pub fn update(&mut self, msg: AudioMsg) {
        let channel = &mut self.channels[msg.channel];
        match msg.kind {
            AudioMsgKind::Gain(gain) => channel.gain = gain,
            AudioMsgKind::Mode(mode) => channel.mode = mode,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AudioMsg {
    pub channel: usize,
    // just gain for now
    pub kind: AudioMsgKind,
}

#[derive(Debug, Clone, Copy)]
pub enum AudioMsgKind {
    Gain(f64),
    Mode(ChannelMode),
}

/// A struct that accumulates metering info during a frame.
#[derive(Debug, Copy, Clone)]
pub struct MeterAcc {
    sum_squares_in: f32,
    max_in: f32,
    sum_squares_out: f32,
    max_out: f32,
}

impl MeterAcc {
    fn new() -> Self {
        MeterAcc {
            sum_squares_in: 0.0,
            max_in: 0.0,
            sum_squares_out: 0.0,
            max_out: 0.0,
        }
    }

    fn sample_in(&mut self, val: f32) {
        if self.max_in < val.abs() {
            self.max_in = val.abs();
        }
        self.sum_squares_in += val * val;
    }

    fn sample_out(&mut self, val: f32) {
        if self.max_out < val.abs() {
            self.max_out = val.abs();
        }
        self.sum_squares_out += val * val;
    }

    fn as_metering(&self, count: usize) -> Metering {
        Metering {
            max_in: self.max_in as f64,
            rms_in: (self.sum_squares_in as f64 / count as f64).sqrt(),
            max_out: self.max_out as f64,
            rms_out: (self.sum_squares_out as f64 / count as f64).sqrt(),
        }
    }

    fn clear(&mut self) {
        self.max_in = 0.0;
        self.sum_squares_in = 0.0;
        self.max_out = 0.0;
        self.sum_squares_out = 0.0;
    }
}
