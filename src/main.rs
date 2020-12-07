use mixjack::{
    cli::{Config, Opt},
    effects::FIRFilter,
    run_mixer, Result,
};
use std::sync::Arc;
use structopt::StructOpt;

fn run(opts: Opt) -> Result {
    let config = Arc::new(Config::load(opts.config_file.as_ref())?);
    if opts.print_filters {
        use fftw::{
            array::AlignedVec,
            plan::{R2RPlan, R2RPlan32},
            types::{Flag, R2RKind},
        };
        let lpfweights = FIRFilter::low_pass(800., 44100., 200).debug_weights();
        let n = lpfweights.len();
        let mut a = AlignedVec::new(n);
        let mut b = AlignedVec::new(n);
        let mut plan: R2RPlan32 =
            R2RPlan::new(&[n], &mut a, &mut b, R2RKind::FFTW_R2HC, Flag::ESTIMATE).unwrap();
        a.as_slice_mut().copy_from_slice(&lpfweights);
        plan.r2r(&mut a, &mut b).unwrap();
        println!("{:?}", b.as_slice());
        return Ok(());
    }
    if opts.print_window {
        use fftw::{
            array::AlignedVec,
            plan::{R2RPlan, R2RPlan32},
            types::{Flag, R2RKind},
        };
        let lpfwin = FIRFilter::low_pass(800., 44100., 512).debug_window();
        let n = lpfwin.len();
        let mut a = AlignedVec::new(n);
        let mut b = AlignedVec::new(n);
        let mut plan: R2RPlan32 =
            R2RPlan::new(&[n], &mut a, &mut b, R2RKind::FFTW_R2HC, Flag::ESTIMATE).unwrap();
        a.as_slice_mut().copy_from_slice(&lpfwin);
        plan.r2r(&mut a, &mut b).unwrap();
        println!("{:?}", b.as_slice());
        return Ok(());
    }
    run_mixer(config, opts)
}

/// Wrap the run method so we can pass it command line args, setup logging, and handle errors
/// gracefully.
fn main() {
    let opts = Opt::from_args();
    setup_logger(opts.verbosity);
    if let Err(err) = run(opts) {
        log::error!("{:?}", err);
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
        .filter(Some("mixjack"), level)
        .init()
}
