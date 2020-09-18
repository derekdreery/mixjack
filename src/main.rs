mod cli;
mod data;
mod effects;
mod gui;
mod ports;

use crossbeam_channel as channel;
use druid::Target;
use jack::Client;
use structopt::StructOpt;

use crate::{
    cli::Opt,
    data::{ChanInfo, Msg, PcmInfo, State, StateChange},
    ports::Ports,
};

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

const LOW_CUTOFF: f32 = 200.0;
const HIGH_CUTOFF: f32 = 2000.0;
const FILTER_LENGTH: usize = 21;

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

    let ports = Ports::setup(
        &client,
        tx_rt,
        rx_rt,
        LOW_CUTOFF,
        HIGH_CUTOFF,
        FILTER_LENGTH,
    )?;
    // todo look at shutting down gracefully, whether that is necessary
    let _async_client = client.activate_async((), ports)?;

    let (evt_sink, ui_handle) = gui::run(tx_ui, shutdown_tx)?;

    loop {
        channel::select! {
            recv(rx_ui) -> msg => {
                // translate from non-blocking crossbeam::Channel to blocking to ExtEventSink
                let msg = msg?; // There should never be an error here.
                evt_sink.submit_command(gui::UPDATE, msg, Target::Global)?;
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
