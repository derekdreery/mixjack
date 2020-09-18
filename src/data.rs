use crate::Result;
use crossbeam_channel::Sender;
use druid::{Data, Lens};

#[derive(Debug, Clone, Data, Lens, PartialEq)]
pub struct State {
    pub fader_1_1: f64,
    pub fader_1_2: f64,
    pub fader_1_3: f64,
    pub fader_1_4: f64,
    pub fader_1_5: f64,
    pub fader_1_6: f64,
    pub fader_1_7: f64,
    pub fader_1_8: f64,
    pub fader_2_1: f64,
    pub fader_2_2: f64,
    pub fader_2_3: f64,
    pub fader_2_4: f64,
    pub fader_2_5: f64,
    pub fader_2_6: f64,
    pub fader_2_7: f64,
    pub fader_2_8: f64,
    pub fader_3_1: f64,
    pub fader_3_2: f64,
    pub fader_3_3: f64,
    pub fader_3_4: f64,
    pub fader_3_5: f64,
    pub fader_3_6: f64,
    pub fader_3_7: f64,
    pub fader_3_8: f64,
    pub fader_4_1: f64,
    pub fader_4_2: f64,
    pub fader_4_3: f64,
    pub fader_4_4: f64,
    pub fader_4_5: f64,
    pub fader_4_6: f64,
    pub fader_4_7: f64,
    pub fader_4_8: f64,
    pub filter_passthru_1: bool,
    pub filter_passthru_2: bool,
    pub filter_passthru_3: bool,
    pub filter_passthru_4: bool,
    pub filter_passthru_5: bool,
    pub filter_passthru_6: bool,
    pub filter_passthru_7: bool,
    pub filter_passthru_8: bool,
}

impl State {
    pub fn update(&mut self, msg: StateChange) -> bool {
        match msg {
            StateChange::fader_1_1(v) => {
                if self.fader_1_1 != v {
                    self.fader_1_1 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_1_2(v) => {
                if self.fader_1_2 != v {
                    self.fader_1_2 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_1_3(v) => {
                if self.fader_1_3 != v {
                    self.fader_1_3 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_1_4(v) => {
                if self.fader_1_4 != v {
                    self.fader_1_4 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_1_5(v) => {
                if self.fader_1_5 != v {
                    self.fader_1_5 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_1_6(v) => {
                if self.fader_1_6 != v {
                    self.fader_1_6 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_1_7(v) => {
                if self.fader_1_7 != v {
                    self.fader_1_7 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_1_8(v) => {
                if self.fader_1_8 != v {
                    self.fader_1_8 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_2_1(v) => {
                if self.fader_2_1 != v {
                    self.fader_2_1 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_2_2(v) => {
                if self.fader_2_2 != v {
                    self.fader_2_2 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_2_3(v) => {
                if self.fader_2_3 != v {
                    self.fader_2_3 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_2_4(v) => {
                if self.fader_2_4 != v {
                    self.fader_2_4 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_2_5(v) => {
                if self.fader_2_5 != v {
                    self.fader_2_5 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_2_6(v) => {
                if self.fader_2_6 != v {
                    self.fader_2_6 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_2_7(v) => {
                if self.fader_2_7 != v {
                    self.fader_2_7 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_2_8(v) => {
                if self.fader_2_8 != v {
                    self.fader_2_8 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_3_1(v) => {
                if self.fader_3_1 != v {
                    self.fader_3_1 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_3_2(v) => {
                if self.fader_3_2 != v {
                    self.fader_3_2 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_3_3(v) => {
                if self.fader_3_3 != v {
                    self.fader_3_3 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_3_4(v) => {
                if self.fader_3_4 != v {
                    self.fader_3_4 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_3_5(v) => {
                if self.fader_3_5 != v {
                    self.fader_3_5 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_3_6(v) => {
                if self.fader_3_6 != v {
                    self.fader_3_6 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_3_7(v) => {
                if self.fader_3_7 != v {
                    self.fader_3_7 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_3_8(v) => {
                if self.fader_3_8 != v {
                    self.fader_3_8 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_4_1(v) => {
                if self.fader_4_1 != v {
                    self.fader_4_1 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_4_2(v) => {
                if self.fader_4_2 != v {
                    self.fader_4_2 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_4_3(v) => {
                if self.fader_4_3 != v {
                    self.fader_4_3 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_4_4(v) => {
                if self.fader_4_4 != v {
                    self.fader_4_4 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_4_5(v) => {
                if self.fader_4_5 != v {
                    self.fader_4_5 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_4_6(v) => {
                if self.fader_4_6 != v {
                    self.fader_4_6 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_4_7(v) => {
                if self.fader_4_7 != v {
                    self.fader_4_7 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::fader_4_8(v) => {
                if self.fader_4_8 != v {
                    self.fader_4_8 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::filter_passthru_1(v) => {
                if self.filter_passthru_1 != v {
                    self.filter_passthru_1 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::filter_passthru_2(v) => {
                if self.filter_passthru_2 != v {
                    self.filter_passthru_2 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::filter_passthru_3(v) => {
                if self.filter_passthru_3 != v {
                    self.filter_passthru_3 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::filter_passthru_4(v) => {
                if self.filter_passthru_4 != v {
                    self.filter_passthru_4 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::filter_passthru_5(v) => {
                if self.filter_passthru_5 != v {
                    self.filter_passthru_5 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::filter_passthru_6(v) => {
                if self.filter_passthru_6 != v {
                    self.filter_passthru_6 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::filter_passthru_7(v) => {
                if self.filter_passthru_7 != v {
                    self.filter_passthru_7 = v;
                    true
                } else {
                    false
                }
            }
            StateChange::filter_passthru_8(v) => {
                if self.filter_passthru_8 != v {
                    self.filter_passthru_8 = v;
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn compute_changes(&self, prev: &State, sender: &Sender<StateChange>) -> Result {
        if self.fader_1_1 != prev.fader_1_1 {
            sender.send(StateChange::fader_1_1(self.fader_1_1))?;
        }
        if self.fader_1_2 != prev.fader_1_2 {
            sender.send(StateChange::fader_1_2(self.fader_1_2))?;
        }
        if self.fader_1_3 != prev.fader_1_3 {
            sender.send(StateChange::fader_1_3(self.fader_1_3))?;
        }
        if self.fader_1_4 != prev.fader_1_4 {
            sender.send(StateChange::fader_1_4(self.fader_1_4))?;
        }
        if self.fader_1_5 != prev.fader_1_5 {
            sender.send(StateChange::fader_1_5(self.fader_1_5))?;
        }
        if self.fader_1_6 != prev.fader_1_6 {
            sender.send(StateChange::fader_1_6(self.fader_1_6))?;
        }
        if self.fader_1_7 != prev.fader_1_7 {
            sender.send(StateChange::fader_1_7(self.fader_1_7))?;
        }
        if self.fader_1_8 != prev.fader_1_8 {
            sender.send(StateChange::fader_1_8(self.fader_1_8))?;
        }
        if self.fader_2_1 != prev.fader_2_1 {
            sender.send(StateChange::fader_2_1(self.fader_2_1))?;
        }
        if self.fader_2_2 != prev.fader_2_2 {
            sender.send(StateChange::fader_2_2(self.fader_2_2))?;
        }
        if self.fader_2_3 != prev.fader_2_3 {
            sender.send(StateChange::fader_2_3(self.fader_2_3))?;
        }
        if self.fader_2_4 != prev.fader_2_4 {
            sender.send(StateChange::fader_2_4(self.fader_2_4))?;
        }
        if self.fader_2_5 != prev.fader_2_5 {
            sender.send(StateChange::fader_2_5(self.fader_2_5))?;
        }
        if self.fader_2_6 != prev.fader_2_6 {
            sender.send(StateChange::fader_2_6(self.fader_2_6))?;
        }
        if self.fader_2_7 != prev.fader_2_7 {
            sender.send(StateChange::fader_2_7(self.fader_2_7))?;
        }
        if self.fader_2_8 != prev.fader_2_8 {
            sender.send(StateChange::fader_2_8(self.fader_2_8))?;
        }
        if self.fader_3_1 != prev.fader_3_1 {
            sender.send(StateChange::fader_3_1(self.fader_3_1))?;
        }
        if self.fader_3_2 != prev.fader_3_2 {
            sender.send(StateChange::fader_3_2(self.fader_3_2))?;
        }
        if self.fader_3_3 != prev.fader_3_3 {
            sender.send(StateChange::fader_3_3(self.fader_3_3))?;
        }
        if self.fader_3_4 != prev.fader_3_4 {
            sender.send(StateChange::fader_3_4(self.fader_3_4))?;
        }
        if self.fader_3_5 != prev.fader_3_5 {
            sender.send(StateChange::fader_3_5(self.fader_3_5))?;
        }
        if self.fader_3_6 != prev.fader_3_6 {
            sender.send(StateChange::fader_3_6(self.fader_3_6))?;
        }
        if self.fader_3_7 != prev.fader_3_7 {
            sender.send(StateChange::fader_3_7(self.fader_3_7))?;
        }
        if self.fader_3_8 != prev.fader_3_8 {
            sender.send(StateChange::fader_3_8(self.fader_3_8))?;
        }
        if self.fader_4_1 != prev.fader_4_1 {
            sender.send(StateChange::fader_4_1(self.fader_4_1))?;
        }
        if self.fader_4_2 != prev.fader_4_2 {
            sender.send(StateChange::fader_4_2(self.fader_4_2))?;
        }
        if self.fader_4_3 != prev.fader_4_3 {
            sender.send(StateChange::fader_4_3(self.fader_4_3))?;
        }
        if self.fader_4_4 != prev.fader_4_4 {
            sender.send(StateChange::fader_4_4(self.fader_4_4))?;
        }
        if self.fader_4_5 != prev.fader_4_5 {
            sender.send(StateChange::fader_4_5(self.fader_4_5))?;
        }
        if self.fader_4_6 != prev.fader_4_6 {
            sender.send(StateChange::fader_4_6(self.fader_4_6))?;
        }
        if self.fader_4_7 != prev.fader_4_7 {
            sender.send(StateChange::fader_4_7(self.fader_4_7))?;
        }
        if self.fader_4_8 != prev.fader_4_8 {
            sender.send(StateChange::fader_4_8(self.fader_4_8))?;
        }
        if self.filter_passthru_1 != prev.filter_passthru_1 {
            sender.send(StateChange::filter_passthru_1(self.filter_passthru_1))?;
        }
        if self.filter_passthru_2 != prev.filter_passthru_2 {
            sender.send(StateChange::filter_passthru_2(self.filter_passthru_2))?;
        }
        if self.filter_passthru_3 != prev.filter_passthru_3 {
            sender.send(StateChange::filter_passthru_3(self.filter_passthru_3))?;
        }
        if self.filter_passthru_4 != prev.filter_passthru_4 {
            sender.send(StateChange::filter_passthru_4(self.filter_passthru_4))?;
        }
        if self.filter_passthru_5 != prev.filter_passthru_5 {
            sender.send(StateChange::filter_passthru_5(self.filter_passthru_5))?;
        }
        if self.filter_passthru_6 != prev.filter_passthru_6 {
            sender.send(StateChange::filter_passthru_6(self.filter_passthru_6))?;
        }
        if self.filter_passthru_7 != prev.filter_passthru_7 {
            sender.send(StateChange::filter_passthru_7(self.filter_passthru_7))?;
        }
        if self.filter_passthru_8 != prev.filter_passthru_8 {
            sender.send(StateChange::filter_passthru_8(self.filter_passthru_8))?;
        }
        Ok(())
    }
}

impl Default for State {
    fn default() -> Self {
        State {
            // eq
            fader_1_1: 0.5,
            fader_1_2: 0.5,
            fader_1_3: 0.5,
            fader_1_4: 0.5,
            fader_1_5: 0.5,
            fader_1_6: 0.5,
            fader_1_7: 0.5,
            fader_1_8: 0.5,
            fader_2_1: 0.5,
            fader_2_2: 0.5,
            fader_2_3: 0.5,
            fader_2_4: 0.5,
            fader_2_5: 0.5,
            fader_2_6: 0.5,
            fader_2_7: 0.5,
            fader_2_8: 0.5,
            fader_3_1: 0.5,
            fader_3_2: 0.5,
            fader_3_3: 0.5,
            fader_3_4: 0.5,
            fader_3_5: 0.5,
            fader_3_6: 0.5,
            fader_3_7: 0.5,
            fader_3_8: 0.5,
            // volume
            fader_4_1: 0.0,
            fader_4_2: 0.0,
            fader_4_3: 0.0,
            fader_4_4: 0.0,
            fader_4_5: 0.0,
            fader_4_6: 0.0,
            fader_4_7: 0.0,
            fader_4_8: 0.0,
            // passthru
            filter_passthru_1: true,
            filter_passthru_2: true,
            filter_passthru_3: true,
            filter_passthru_4: true,
            filter_passthru_5: true,
            filter_passthru_6: true,
            filter_passthru_7: true,
            filter_passthru_8: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum StateChange {
    fader_1_1(f64),
    fader_1_2(f64),
    fader_1_3(f64),
    fader_1_4(f64),
    fader_1_5(f64),
    fader_1_6(f64),
    fader_1_7(f64),
    fader_1_8(f64),
    fader_2_1(f64),
    fader_2_2(f64),
    fader_2_3(f64),
    fader_2_4(f64),
    fader_2_5(f64),
    fader_2_6(f64),
    fader_2_7(f64),
    fader_2_8(f64),
    fader_3_1(f64),
    fader_3_2(f64),
    fader_3_3(f64),
    fader_3_4(f64),
    fader_3_5(f64),
    fader_3_6(f64),
    fader_3_7(f64),
    fader_3_8(f64),
    fader_4_1(f64),
    fader_4_2(f64),
    fader_4_3(f64),
    fader_4_4(f64),
    fader_4_5(f64),
    fader_4_6(f64),
    fader_4_7(f64),
    fader_4_8(f64),
    filter_passthru_1(bool),
    filter_passthru_2(bool),
    filter_passthru_3(bool),
    filter_passthru_4(bool),
    filter_passthru_5(bool),
    filter_passthru_6(bool),
    filter_passthru_7(bool),
    filter_passthru_8(bool),
}

#[derive(Debug, Clone)]
pub enum Msg {
    StateChange(StateChange),
    PcmInfo(PcmInfo),
}

#[derive(Debug, Clone, Copy, Default, Data, PartialEq)]
pub struct PcmInfo {
    pub in1: ChanInfo,
    pub in2: ChanInfo,
    pub in3: ChanInfo,
    pub in4: ChanInfo,
    pub in5: ChanInfo,
    pub in6: ChanInfo,
    pub in7: ChanInfo,
    pub in8: ChanInfo,
    pub out1: ChanInfo,
    pub out2: ChanInfo,
}

impl PcmInfo {
    pub fn merge(&mut self, other: &Self) {
        self.in1.merge(&other.in1);
        self.in2.merge(&other.in2);
        self.in3.merge(&other.in3);
        self.in4.merge(&other.in4);
        self.in5.merge(&other.in5);
        self.in6.merge(&other.in6);
        self.in7.merge(&other.in7);
        self.in8.merge(&other.in8);
        self.out1.merge(&other.out1);
        self.out2.merge(&other.out2);
    }

    pub fn clear(&mut self) {
        self.in1.clear();
        self.in2.clear();
        self.in3.clear();
        self.in4.clear();
        self.in5.clear();
        self.in6.clear();
        self.in7.clear();
        self.in8.clear();
        self.out1.clear();
        self.out2.clear();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Data, Default)]
pub struct ChanInfo {
    total_squared: f64,
    // This allows us to calculate rms on merged data.
    weight: f64,
    pub max: f64,
}

impl ChanInfo {
    pub fn new(total_squared: f64, weight: f64, max: f64) -> Self {
        ChanInfo {
            total_squared,
            weight,
            max,
        }
    }
    pub fn rms(&self) -> f64 {
        (self.total_squared / self.weight).sqrt()
    }

    pub fn merge(&mut self, other: &Self) {
        self.total_squared += other.total_squared;
        self.weight += other.weight;
        self.max = self.max.max(other.max);
    }

    pub fn clear(&mut self) {
        self.total_squared = 0.0;
        self.weight = 0.0;
        self.max = 0.0;
    }
}
