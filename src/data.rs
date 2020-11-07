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
    pub max: f64,
    pub rms: f64,
}

impl Metering {
    pub fn new(max: f64, rms: f64) -> Self {
        Metering { max, rms }
    }
}
