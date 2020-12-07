mod audio;
pub mod cli;
mod data;
pub mod effects;
mod gui;
mod monitor_data;

use crossbeam_channel as channel;
use druid::Target;
use jack::Client;
use std::{sync::Arc, thread};
use structopt::StructOpt;

use crate::{
    audio::{Audio, Info as AudioInfo},
    cli::{Config, Opt},
    effects::hc_to_mod,
    gui::UiMsg,
};

pub type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

const LOW_CUTOFF: f32 = 200.0;
const HIGH_CUTOFF: f32 = 2000.0;

/// Main programm runner.
pub fn run_mixer(config: Arc<Config>, opts: Opt) -> Result {
    let (client, status) = Client::new(&opts.jack_name, jack::ClientOptions::NO_START_SERVER)?;
    let info = AudioInfo::from_client(&client);
    info.log();
    if !status.is_empty() {
        log::warn!("client status: {:?}", status);
    }
    log::info!("cpu_load: {}", client.cpu_load());

    // a channel for sending updates to the RT thread.
    let (tx_ui, rx_rt) = channel::bounded(1024);
    // a channel for sending updates from the RT thread to the gui.
    let (tx_rt, rx_ui) = channel::bounded(1024);
    let tx_spectra_in = tx_rt.clone();
    let tx_spectra_out = tx_rt.clone();
    // a channel for finding out when the ui has shut down.
    let (shutdown_tx, shutdown_rx) = channel::bounded(0);

    let audio = Audio::setup(&*config, &client, tx_rt, rx_rt, LOW_CUTOFF, HIGH_CUTOFF)?;
    let (audio_in_spectrum, audio_out_spectrum) =
        audio.monitor_spectra().into_iter().next().unwrap();
    // todo look at shutting down gracefully, whether that is necessary
    let _async_client = client.activate_async((), audio)?;

    let (evt_sink, ui_handle) = gui::run(tx_ui, shutdown_tx, config)?;

    thread::spawn(move || {
        audio_in_spectrum.on_changed(|spec| {
            tx_spectra_in
                .send(UiMsg::AudioInSpectrum(hc_to_mod(spec)))
                .unwrap()
        });
    });

    thread::spawn(move || {
        audio_out_spectrum.on_changed(|spec| {
            tx_spectra_out
                .send(UiMsg::AudioOutSpectrum(hc_to_mod(spec)))
                .unwrap()
        });
    });

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
