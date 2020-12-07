use crate::{
    cli::{Config, MidiKey, MidiLookup},
    Result,
};
use crossbeam_channel::Sender;
use druid::{Data, Lens};
use im::{vector, Vector};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Data, Default)]
pub struct Metering {
    pub max_in: f64,
    pub rms_in: f64,
    pub max_out: f64,
    pub rms_out: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Data)]
pub enum ChannelMode {
    Normal,
    Bypass,
    Mute,
}

impl Default for ChannelMode {
    fn default() -> Self {
        Self::Mute
    }
}
