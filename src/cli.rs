use nom::IResult;
use std::str::FromStr;
use structopt::StructOpt;

/// Provide a simple adapter to alter the volume of a jack stream.
#[derive(StructOpt, Debug)]
pub struct Opt {
    /// How verbose should we be (normal = info, 1 = debug, 2+ = trace).
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbosity: u32,
    /// Whether to run jack-volume in cli mode, where commands are issued over stdin.
    #[structopt(short = "c", long = "cli")]
    pub cli: bool,
    /// A custom name for the adapter in jack. This will be used by e.g. LADISH to reconnect this
    /// widget when it appears.
    #[structopt(long = "jack-name", default_value = "jack-volume")]
    pub jack_name: String,
    /// The volume that this instance of jack-volume will start at (between 0-100)
    #[structopt(long = "vol", default_value = "100")]
    pub init_volume: InitVolume,
    /// The channel and controller number of a midi controller for changing the volume (e.g.
    /// `--midi 8:77`) for channel 8, controller 77.
    #[structopt(long = "midi")]
    pub midi: Option<MidiConf>,
}

/// this is a newtype wrapper struct so we can implement `FromStr` for it and get nice error
/// messages from StructOpt/clap.
#[derive(Debug)]
pub struct InitVolume(pub u8);

impl FromStr for InitVolume {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input
            .chars()
            .filter(|ch| !ch.is_ascii_digit())
            .next()
            .is_some()
        {
            return Err(format!(
                "expected a number between 0 and 100, found \"{}\"",
                input
            ));
        }
        let input = input.parse::<usize>().map_err(|e| e.to_string())?;
        if input > 100 {
            return Err(format!(
                "expected a number between 0 and 100, found {}",
                input
            ));
        }
        Ok(InitVolume(input as u8))
    }
}

/// This is a newtype wrapper struct so we can implement `FromStr` for it and get nice error
/// messages from StructOpt/clap.
#[derive(Debug, Copy, Clone)]
pub struct MidiConf {
    pub channel: u8,
    pub controller: u8,
}

impl FromStr for MidiConf {
    type Err = &'static str;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        parse_midi_conf(input)
            .map(|(_, conf)| conf)
            .map_err(|_| "invalid midi configuration")
    }
}

/// `nom` helper to parse our midi config syntax (`<channel>:<controller number>`).
///
/// Strictly speaking we should restrict the channel and controller more, as not all u8 values are
/// valid, but because we only match on these later it's harmless not to do this.
fn parse_midi_conf(i: &str) -> IResult<&str, MidiConf> {
    use nom::bytes::complete::tag;
    let (i, channel) = parse_u8(i)?;
    let (i, _) = tag(":")(i)?;
    let (i, controller) = parse_u8(i)?;
    Ok((
        i,
        MidiConf {
            channel,
            controller,
        },
    ))
}

/// Get 1-3 digits and parse them into a u8.
fn parse_u8(i: &str) -> IResult<&str, u8> {
    use nom::{bytes::complete::take_while_m_n, combinator::map_res};
    map_res(
        take_while_m_n(1, 3, |ch: char| ch.is_ascii_digit()),
        u8::from_str,
    )(i)
}
