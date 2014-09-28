mod cli;
mod gui;

use jack::{AudioIn, AudioOut, Client, Control, MidiIn, Port, ProcessHandler, ProcessScope};
use nom_midi::{MidiEvent, MidiEventType};
use std::{
    error::Error as StdError,
    io::{self, BufRead},
    str::FromStr,
    sync::atomic::{AtomicI8, Ordering},
};
use structopt::StructOpt;

use crate::cli::Opt;

/// I'm using i8 because it is what's used in MIDI.
///
/// Actually, in MIDI i8 and u8 are the same because you never use the top bit (at least for
/// controller amounts).
static VOLUME: AtomicI8 = AtomicI8::new(127);
/// So I don't have to click so often
const VOL_MULTIPLIER: i8 = 3;

/// Main programm runner.
fn run(opts: Opt) -> Result<(), Box<dyn StdError>> {
    // setup initial volume
    let init_volume = (((opts.init_volume.0 as u32) * 127) / 100).min(127).max(0) as i8;
    VOLUME.store(init_volume, Ordering::SeqCst);

    let (client, status) = Client::new(&opts.jack_name, jack::ClientOptions::NO_START_SERVER)?;
    log::info!("client status: {:?}", status);
    log::info!("sample rate: {}", client.sample_rate());
    log::info!("cpu_load: {}", client.cpu_load());
    log::info!("name: {}", client.name());
    log::info!("buffer size: {}", client.buffer_size());
    if let Some(midi_conf) = opts.midi {
        log::info!(
            "receiving midi controller events on channel {}, control number {}",
            midi_conf.channel,
            midi_conf.controller
        );
    }
    let ports = Ports::setup(&client, opts.midi)?;
    let async_client = client.activate_async((), ports)?;
    if opts.cli {
        // loop waiting for commands and pass them to the realtime thread.
        //
        // You probably want to use the gui or the midi interface to alter the volume.
        for line in io::stdin().lock().lines() {
            let msg = match Msg::from_str(&line?) {
                Ok(msg) => msg,
                Err(e) => {
                    log::error!("{}", e);
                    continue;
                }
            };
            change_volume(msg);
        }
    } else {
        gui::run_gtk(async_client.as_client().name());
    }
    Ok(())
}

/// This method sets the global volume by incrementing or decrementing it based on the message.
///
/// It also returns the new volume.
fn change_volume(msg: Msg) -> i8 {
    log::debug!("Got command {:?}", msg);
    let old_volume = VOLUME.load(Ordering::Relaxed);
    log::debug!("old volume: {}", old_volume);
    let new_volume = match msg {
        Msg::Up => old_volume.saturating_add(1 * VOL_MULTIPLIER),
        Msg::Down => old_volume.saturating_sub(1 * VOL_MULTIPLIER).max(0),
    };
    VOLUME.compare_and_swap(old_volume, new_volume, Ordering::AcqRel);
    debug_assert_eq!(new_volume, VOLUME.load(Ordering::SeqCst));
    log::debug!("new volume: {}", new_volume);
    new_volume
}

/// This structure holds all the info we need to process the audio/midi signals in the realtime
/// thread.
///
/// It consists of the 4 audio ports (it's hard-coded for stereo duplex RN), and optionally a midi
/// port/controller number for midi control.
struct Ports {
    in_left: Port<AudioIn>,
    in_right: Port<AudioIn>,
    out_left: Port<AudioOut>,
    out_right: Port<AudioOut>,
    /// If we are using midi, the configuration for it and the port.
    ///
    /// We wrap these both in the option, as it is the best way to express in the type system that
    /// they are either both present or both absent. If we didn't do this, we'd need to use
    /// `unreaps`.
    control_in: Option<(cli::MidiConf, Port<MidiIn>)>,
}

impl Ports {
    /// Our constructor. Here we setup the ports we want and store them in our jack state object.
    fn setup(client: &Client, midi: Option<cli::MidiConf>) -> Result<Self, Box<dyn StdError>> {
        let in_left = client.register_port("in_left", AudioIn)?;
        let in_right = client.register_port("in_right", AudioIn)?;
        let out_left = client.register_port("out_left", AudioOut)?;
        let out_right = client.register_port("out_right", AudioOut)?;
        let control_in = match midi {
            Some(conf) => Some((conf, client.register_port("control_in", MidiIn)?)),
            None => None,
        };

        Ok(Ports {
            in_left,
            in_right,
            out_left,
            out_right,
            control_in,
        })
    }
}

impl ProcessHandler for Ports {
    fn process(&mut self, _client: &Client, process_scope: &ProcessScope) -> Control {
        // process control events
        if let Some((midi_conf, ref port)) = self.control_in {
            for raw_midi in port.iter(process_scope) {
                match nom_midi::parser::parse_midi_event(raw_midi.bytes) {
                    Ok((
                        _,
                        MidiEvent {
                            channel,
                            event: MidiEventType::Controller(ctrl_no, amt),
                        },
                    )) if channel == midi_conf.channel && ctrl_no == midi_conf.controller => {
                        // We don't really care how long it takes for other threads to see this
                        // change. Usually, if you're using the midi interface you probably aren't
                        // using any other, and even if you are, the results won't really matter (a
                        // slightly different volume until you change the midi controller again).
                        VOLUME.store(amt as i8, Ordering::Relaxed)
                    }
                    _ => (), // do nothing for all other midi events
                }
            }
        }
        // process audio
        //
        // Here we copy it first and then operate on it, since maybe this aids cache performance. I
        // haven't benchmarked though so I might be making it slower. I also want to investigate
        // whether using simd would be quicker, or whether LLVM does this for me already.
        let volume = convert_volume(VOLUME.load(Ordering::Relaxed));
        let out_left = self.out_left.as_mut_slice(process_scope);
        out_left.copy_from_slice(self.in_left.as_slice(process_scope));
        for val in out_left.iter_mut() {
            (*val) *= volume;
        }
        let out_right = self.out_right.as_mut_slice(process_scope);
        out_right.copy_from_slice(self.in_right.as_slice(process_scope));
        for val in out_right.iter_mut() {
            (*val) *= volume;
        }
        Control::Continue
    }
}

/// We want to either increment or decrement the volume
#[derive(Debug, Clone)]
pub enum Msg {
    /// Volume up
    Up,
    /// Volume down
    Down,
}

impl FromStr for Msg {
    type Err = &'static str;

    fn from_str(msg: &str) -> Result<Self, Self::Err> {
        if msg.eq_ignore_ascii_case("up") {
            Ok(Msg::Up)
        } else if msg.eq_ignore_ascii_case("down") {
            Ok(Msg::Down)
        } else {
            Err("unrecognised command")
        }
    }
}

/// Convert an i8 volume into a f32 volume
fn convert_volume(volume: i8) -> f32 {
    //log::trace!("volume in i8: {}", volume);
    debug_assert!(volume >= 0);
    let volume = ((volume as f32) / 127.0).max(0.0).min(1.0);
    //log::trace!("volume in f32: {}", volume);
    volume
}

// boilerplate

/// Wrap the run method so we can pass it command line args, setup logging, and handle errors
/// gracefully.
fn main() {
    let opts = Opt::from_args();
    setup_logger(opts.verbosity);
    if let Err(err) = run(opts) {
        log::error!("{}", err);
        let mut e = &*err;
        while let Some(err) = e.source() {
            log::error!("caused by {}", err);
            e = err;
        }
    }
}

/// Make the logger match our verbosity. This is custom because we don't want to see all messages
/// from other packages, only `jack-volume`.
fn setup_logger(verbosity: u32) {
    use log::LevelFilter;
    let level = match verbosity {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    pretty_env_logger::formatted_timed_builder()
        .filter(None, LevelFilter::Warn)
        .filter(Some("jack_volume"), level)
        .init()
}
