mod cli;
mod data;
mod gui;

use crossbeam_channel as channel;
use itertools::izip;
use jack::{AudioIn, AudioOut, Client, Control, MidiIn, Port, ProcessHandler, ProcessScope};
use novation_launch_control::Event;
use structopt::StructOpt;

use crate::cli::Opt;

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;
pub use data::{Msg, State};

/// Main programm runner.
fn run(opts: Opt) -> Result {
    let (client, status) = Client::new(&opts.jack_name, jack::ClientOptions::NO_START_SERVER)?;
    log::info!("client status: {:?}", status);
    log::info!("sample rate: {}", client.sample_rate());
    log::info!("cpu_load: {}", client.cpu_load());
    log::info!("name: {}", client.name());
    log::info!("buffer size: {}", client.buffer_size());

    // a channel for sending updates to the RT thread.
    let (tx_ui, rx_rt) = channel::bounded(1024);
    // a channel for sending updates from the RT thread to the gui.
    let (tx_rt, rx_ui) = channel::bounded(1024);
    // a channel for finding out when the ui has shut down.
    let (shutdown_tx, shutdown_rx) = channel::bounded(0);

    let ports = Ports::setup(&client, tx_rt, rx_rt)?;
    // todo look at shutting down gracefully, whether that is necessary
    let _async_client = client.activate_async((), ports)?;

    let (evt_sink, ui_handle) = gui::run(tx_ui, shutdown_tx)?;
    loop {
        channel::select! {
            recv(rx_ui) -> msg => {
                // translate from non-blocking crossbeam::Channel to blocking to ExtEventSink
                let msg = msg?; // There should never be an error here.
                evt_sink.submit_command(gui::UPDATE, msg, None)?;
            }
            recv(shutdown_rx) -> res => {
                // There should never be an error here.
                let _ = res?;
                break
            }
        }
    }
    ui_handle.join().unwrap()?;
    Ok(())
}

/// This structure holds all the info we need to process the audio/midi signals in the realtime
/// thread.
struct Ports {
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
}

impl Ports {
    /// Our constructor. Here we setup the ports we want and store them in our jack state object.
    fn setup(
        client: &Client,
        tx: channel::Sender<Msg>,
        rx: channel::Receiver<Msg>,
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
        })
    }
}

impl ProcessHandler for Ports {
    fn process(&mut self, _client: &Client, process_scope: &ProcessScope) -> Control {
        use channel::TryRecvError;

        let mut shutdown = false;
        // process control events
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

        // get params
        let fader_4_1 = self.state.fader_4_1 as f32;
        let fader_4_2 = self.state.fader_4_2 as f32;
        let fader_4_3 = self.state.fader_4_3 as f32;
        let fader_4_4 = self.state.fader_4_4 as f32;
        let fader_4_5 = self.state.fader_4_5 as f32;
        let fader_4_6 = self.state.fader_4_6 as f32;
        let fader_4_7 = self.state.fader_4_7 as f32;
        let fader_4_8 = self.state.fader_4_8 as f32;

        // left
        let out_left = self.out_left.as_mut_slice(process_scope);
        let in1_left = self.in1_left.as_slice(process_scope);
        let in2_left = self.in2_left.as_slice(process_scope);
        let in3_left = self.in3_left.as_slice(process_scope);
        let in4_left = self.in4_left.as_slice(process_scope);
        for (out, in1, in2, in3, in4) in izip!(
            out_left.iter_mut(),
            in1_left.iter(),
            in2_left.iter(),
            in3_left.iter(),
            in4_left.iter()
        ) {
            *out = in1 * fader_4_1 + in2 * fader_4_3 + in3 * fader_4_5 + in4 * fader_4_7;
        }

        // right
        let out_right = self.out_right.as_mut_slice(process_scope);
        let in1_right = self.in1_right.as_slice(process_scope);
        let in2_right = self.in2_right.as_slice(process_scope);
        let in3_right = self.in3_right.as_slice(process_scope);
        let in4_right = self.in4_right.as_slice(process_scope);
        for (out, in1, in2, in3, in4) in izip!(
            out_right.iter_mut(),
            in1_right.iter(),
            in2_right.iter(),
            in3_right.iter(),
            in4_right.iter()
        ) {
            *out = in1 * fader_4_2 + in2 * fader_4_4 + in3 * fader_4_6 + in4 * fader_4_8;
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

// boilerplate

/// Wrap the run method so we can pass it command line args, setup logging, and handle errors
/// gracefully.
fn main() {
    let opts = Opt::from_args();
    setup_logger(opts.verbosity);
    if let Err(err) = run(opts) {
        log::error!("{}", err);
        for e in err.chain().skip(1) {
            log::error!("caused by {}", e);
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
