use crate::Result;
use anyhow::format_err;
use directories::ProjectDirs;
use fnv::FnvHashMap as HashMap;
use im::{ordmap, OrdMap};
use itertools::izip;
use midi_event::MidiEvent;
use serde::Deserialize;
use std::{
    convert::TryFrom,
    env, fs, io,
    ops::Deref,
    path::{Path, PathBuf},
};
use structopt::StructOpt;

const CONFIG_FILE_NAME: &str = "config.toml";

/// Configurable mixer for jack, with optional midi control.
#[derive(StructOpt, Debug)]
pub struct Opt {
    /// How verbose should we be (normal = info, 1 = debug, 2+ = trace).
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbosity: u32,
    /// A custom name for the adapter in jack. This will be used by e.g. LADISH to reconnect this
    /// widget when it appears.
    #[structopt(long = "jack-name", default_value = "mixjack")]
    pub jack_name: String,
    /// Location of the config file to load. Will try default locations otherwise (cwd and standard
    /// location.
    #[structopt(long = "config-file", parse(from_os_str))]
    pub config_file: Option<PathBuf>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    // the order of channels matters.
    pub channels: OrdMap<String, Channel>,
}

impl Default for Config {
    fn default() -> Self {
        let channels = ordmap! {
            "left".into() => Channel::empty(),
            "right".into() => Channel::empty()
        };
        Config { channels }
    }
}

impl Config {
    /// Loads the config, if there is one.
    ///
    /// The rules for finding a config file are:
    ///
    ///  - Look at the location given as a parameter (if given).
    ///  - Look in the current directory
    ///  - Look in the project config directory (as defined by `directories` crate)
    pub fn load(loc: Option<impl AsRef<Path>>) -> Result<Self> {
        // parameter
        match loc {
            Some(loc) => {
                let loc = loc.as_ref();
                // Break on all errors, including not found
                log::info!("using config at \"{}\"", loc.display());
                let conf_raw = fs::read(loc)?;
                return Ok(toml::from_slice(&conf_raw)?);
            }
            None => (), // continue
        }

        // current dir
        fn load_from_current() -> Result<Option<Config>> {
            let current_dir = match env::current_dir() {
                Ok(dir) => dir,
                Err(e) if matches!(e.kind(), io::ErrorKind::NotFound) => return Ok(None),
                Err(e) => return Err(e.into()),
            };
            let current_dir_path = current_dir.join(CONFIG_FILE_NAME);
            let conf_raw = match fs::read(&current_dir_path) {
                Ok(x) => x,
                Err(e) if matches!(e.kind(), io::ErrorKind::NotFound) => return Ok(None),
                Err(e) => return Err(e.into()),
            };
            log::info!("using config at \"{}\"", current_dir_path.display());
            Ok(Some(toml::from_slice(&conf_raw)?))
        }
        match load_from_current()? {
            Some(conf) => return Ok(conf),
            None => (), // continue
        }

        // project dir
        let dirs = ProjectDirs::from("org", "derekdreery", "mixjack")
            .ok_or(format_err!("could not load project directories"))?;
        let config_path = dirs.config_dir().join(CONFIG_FILE_NAME);
        log::info!("using config at \"{}\"", config_path.display());
        match fs::read(config_path) {
            Ok(conf_raw) => Ok(toml::from_slice(&conf_raw)?),
            Err(e) if matches!(e.kind(), io::ErrorKind::NotFound) => Ok(Config::default()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn midi_lookup(&self) -> MidiLookup {
        MidiLookup::construct(self)
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct Channel {
    pub high: Option<MidiKey>,
    pub mid: Option<MidiKey>,
    pub low: Option<MidiKey>,
    pub volume: Option<MidiKey>,
}

impl Channel {
    pub fn empty() -> Self {
        Channel {
            high: None,
            mid: None,
            low: None,
            volume: None,
        }
    }
}

#[derive(Deserialize, Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[serde(try_from = "MidiKeyRaw")]
pub struct MidiKey {
    pub channel: u8,
    pub kind: MidiKeyKind,
}

impl MidiKey {
    /// If the event is one that we might handle, convert it to our type.
    pub fn from_opt(evt: MidiEvent) -> Option<Self> {
        use midi_event::MidiEventType::*;
        let kind = match evt.event {
            Controller(ctrl, _) => MidiKeyKind::Controller(ctrl),
            NoteOn(note, _) => MidiKeyKind::Note(note.into()),
            _ => return None,
        };
        Some(MidiKey {
            channel: evt.channel,
            kind,
        })
    }
}

impl MidiKey {
    /// Constructor for controller key.
    pub fn controller(channel: u8, controller: u8) -> Self {
        MidiKey {
            channel,
            kind: MidiKeyKind::Controller(controller),
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum MidiKeyKind {
    Controller(u8),
    Note(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MidiEffect {
    pub channel: usize,
    pub kind: MidiEffectKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MidiEffectKind {
    // ToggleEq
    // High
    // Mid
    // Low
    Gain,
}

// a data structure for quick midi -> action lookups.
#[derive(Debug, Clone, PartialEq)]
pub struct MidiLookup(HashMap<MidiKey, MidiEffect>);

impl MidiLookup {
    pub fn construct(config: &Config) -> Self {
        let mut map = HashMap::default();
        for (idx, (_, chan)) in config.channels.iter().enumerate() {
            if let Some(volume) = chan.volume.as_ref() {
                map.insert(
                    *volume,
                    MidiEffect {
                        channel: idx,
                        kind: MidiEffectKind::Gain,
                    },
                );
            }
        }
        Self(map)
    }
}

impl Deref for MidiLookup {
    type Target = HashMap<MidiKey, MidiEffect>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// helper for deserialize

#[derive(Deserialize, Debug)]
struct MidiKeyRaw(u8, String, u8);

impl TryFrom<MidiKeyRaw> for MidiKey {
    type Error = anyhow::Error;
    fn try_from(raw: MidiKeyRaw) -> Result<Self, Self::Error> {
        let kind = match raw.1.as_str() {
            "ctrl" => MidiKeyKind::Controller(raw.2),
            "note" => MidiKeyKind::Note(raw.2),
            o => return Err(format_err!("unrecognised fader midi kind: {}", o)),
        };
        Ok(MidiKey {
            channel: raw.0,
            kind,
        })
    }
}
