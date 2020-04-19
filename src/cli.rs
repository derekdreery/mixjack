use structopt::StructOpt;

/// Provide a simple adapter to alter the volume of a jack stream.
#[derive(StructOpt, Debug)]
pub struct Opt {
    /// How verbose should we be (normal = info, 1 = debug, 2+ = trace).
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbosity: u32,
    /// A custom name for the adapter in jack. This will be used by e.g. LADISH to reconnect this
    /// widget when it appears.
    #[structopt(long = "jack-name", default_value = "jack-mixer")]
    pub jack_name: String,
}
