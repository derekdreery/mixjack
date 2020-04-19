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
}

impl State {
    pub fn update(&mut self, msg: Msg) -> bool {
        match msg {
            Msg::fader_1_1(v) => {
                if self.fader_1_1 != v {
                    self.fader_1_1 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_1_2(v) => {
                if self.fader_1_2 != v {
                    self.fader_1_2 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_1_3(v) => {
                if self.fader_1_3 != v {
                    self.fader_1_3 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_1_4(v) => {
                if self.fader_1_4 != v {
                    self.fader_1_4 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_1_5(v) => {
                if self.fader_1_5 != v {
                    self.fader_1_5 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_1_6(v) => {
                if self.fader_1_6 != v {
                    self.fader_1_6 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_1_7(v) => {
                if self.fader_1_7 != v {
                    self.fader_1_7 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_1_8(v) => {
                if self.fader_1_8 != v {
                    self.fader_1_8 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_2_1(v) => {
                if self.fader_2_1 != v {
                    self.fader_2_1 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_2_2(v) => {
                if self.fader_2_2 != v {
                    self.fader_2_2 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_2_3(v) => {
                if self.fader_2_3 != v {
                    self.fader_2_3 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_2_4(v) => {
                if self.fader_2_4 != v {
                    self.fader_2_4 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_2_5(v) => {
                if self.fader_2_5 != v {
                    self.fader_2_5 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_2_6(v) => {
                if self.fader_2_6 != v {
                    self.fader_2_6 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_2_7(v) => {
                if self.fader_2_7 != v {
                    self.fader_2_7 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_2_8(v) => {
                if self.fader_2_8 != v {
                    self.fader_2_8 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_3_1(v) => {
                if self.fader_3_1 != v {
                    self.fader_3_1 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_3_2(v) => {
                if self.fader_3_2 != v {
                    self.fader_3_2 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_3_3(v) => {
                if self.fader_3_3 != v {
                    self.fader_3_3 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_3_4(v) => {
                if self.fader_3_4 != v {
                    self.fader_3_4 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_3_5(v) => {
                if self.fader_3_5 != v {
                    self.fader_3_5 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_3_6(v) => {
                if self.fader_3_6 != v {
                    self.fader_3_6 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_3_7(v) => {
                if self.fader_3_7 != v {
                    self.fader_3_7 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_3_8(v) => {
                if self.fader_3_8 != v {
                    self.fader_3_8 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_4_1(v) => {
                if self.fader_4_1 != v {
                    self.fader_4_1 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_4_2(v) => {
                if self.fader_4_2 != v {
                    self.fader_4_2 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_4_3(v) => {
                if self.fader_4_3 != v {
                    self.fader_4_3 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_4_4(v) => {
                if self.fader_4_4 != v {
                    self.fader_4_4 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_4_5(v) => {
                if self.fader_4_5 != v {
                    self.fader_4_5 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_4_6(v) => {
                if self.fader_4_6 != v {
                    self.fader_4_6 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_4_7(v) => {
                if self.fader_4_7 != v {
                    self.fader_4_7 = v;
                    true
                } else {
                    false
                }
            }
            Msg::fader_4_8(v) => {
                if self.fader_4_8 != v {
                    self.fader_4_8 = v;
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn compute_changes(&self, prev: &State, sender: &Sender<Msg>) -> Result {
        if self.fader_1_1 != prev.fader_1_1 {
            sender.send(Msg::fader_1_1(self.fader_1_1))?;
        }
        if self.fader_1_2 != prev.fader_1_2 {
            sender.send(Msg::fader_1_2(self.fader_1_2))?;
        }
        if self.fader_1_3 != prev.fader_1_3 {
            sender.send(Msg::fader_1_3(self.fader_1_3))?;
        }
        if self.fader_1_4 != prev.fader_1_4 {
            sender.send(Msg::fader_1_4(self.fader_1_4))?;
        }
        if self.fader_1_5 != prev.fader_1_5 {
            sender.send(Msg::fader_1_5(self.fader_1_5))?;
        }
        if self.fader_1_6 != prev.fader_1_6 {
            sender.send(Msg::fader_1_6(self.fader_1_6))?;
        }
        if self.fader_1_7 != prev.fader_1_7 {
            sender.send(Msg::fader_1_7(self.fader_1_7))?;
        }
        if self.fader_1_8 != prev.fader_1_8 {
            sender.send(Msg::fader_1_8(self.fader_1_8))?;
        }
        if self.fader_2_1 != prev.fader_2_1 {
            sender.send(Msg::fader_2_1(self.fader_2_1))?;
        }
        if self.fader_2_2 != prev.fader_2_2 {
            sender.send(Msg::fader_2_2(self.fader_2_2))?;
        }
        if self.fader_2_3 != prev.fader_2_3 {
            sender.send(Msg::fader_2_3(self.fader_2_3))?;
        }
        if self.fader_2_4 != prev.fader_2_4 {
            sender.send(Msg::fader_2_4(self.fader_2_4))?;
        }
        if self.fader_2_5 != prev.fader_2_5 {
            sender.send(Msg::fader_2_5(self.fader_2_5))?;
        }
        if self.fader_2_6 != prev.fader_2_6 {
            sender.send(Msg::fader_2_6(self.fader_2_6))?;
        }
        if self.fader_2_7 != prev.fader_2_7 {
            sender.send(Msg::fader_2_7(self.fader_2_7))?;
        }
        if self.fader_2_8 != prev.fader_2_8 {
            sender.send(Msg::fader_2_8(self.fader_2_8))?;
        }
        if self.fader_3_1 != prev.fader_3_1 {
            sender.send(Msg::fader_3_1(self.fader_3_1))?;
        }
        if self.fader_3_2 != prev.fader_3_2 {
            sender.send(Msg::fader_3_2(self.fader_3_2))?;
        }
        if self.fader_3_3 != prev.fader_3_3 {
            sender.send(Msg::fader_3_3(self.fader_3_3))?;
        }
        if self.fader_3_4 != prev.fader_3_4 {
            sender.send(Msg::fader_3_4(self.fader_3_4))?;
        }
        if self.fader_3_5 != prev.fader_3_5 {
            sender.send(Msg::fader_3_5(self.fader_3_5))?;
        }
        if self.fader_3_6 != prev.fader_3_6 {
            sender.send(Msg::fader_3_6(self.fader_3_6))?;
        }
        if self.fader_3_7 != prev.fader_3_7 {
            sender.send(Msg::fader_3_7(self.fader_3_7))?;
        }
        if self.fader_3_8 != prev.fader_3_8 {
            sender.send(Msg::fader_3_8(self.fader_3_8))?;
        }
        if self.fader_4_1 != prev.fader_4_1 {
            sender.send(Msg::fader_4_1(self.fader_4_1))?;
        }
        if self.fader_4_2 != prev.fader_4_2 {
            sender.send(Msg::fader_4_2(self.fader_4_2))?;
        }
        if self.fader_4_3 != prev.fader_4_3 {
            sender.send(Msg::fader_4_3(self.fader_4_3))?;
        }
        if self.fader_4_4 != prev.fader_4_4 {
            sender.send(Msg::fader_4_4(self.fader_4_4))?;
        }
        if self.fader_4_5 != prev.fader_4_5 {
            sender.send(Msg::fader_4_5(self.fader_4_5))?;
        }
        if self.fader_4_6 != prev.fader_4_6 {
            sender.send(Msg::fader_4_6(self.fader_4_6))?;
        }
        if self.fader_4_7 != prev.fader_4_7 {
            sender.send(Msg::fader_4_7(self.fader_4_7))?;
        }
        if self.fader_4_8 != prev.fader_4_8 {
            sender.send(Msg::fader_4_8(self.fader_4_8))?;
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
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum Msg {
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
}
