#![no_std]

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Event {
    // top row
    Fader1_1(f32),
    Fader1_2(f32),
    Fader1_3(f32),
    Fader1_4(f32),
    Fader1_5(f32),
    Fader1_6(f32),
    Fader1_7(f32),
    Fader1_8(f32),
    // second row
    Fader2_1(f32),
    Fader2_2(f32),
    Fader2_3(f32),
    Fader2_4(f32),
    Fader2_5(f32),
    Fader2_6(f32),
    Fader2_7(f32),
    Fader2_8(f32),
    // third row
    Fader3_1(f32),
    Fader3_2(f32),
    Fader3_3(f32),
    Fader3_4(f32),
    Fader3_5(f32),
    Fader3_6(f32),
    Fader3_7(f32),
    Fader3_8(f32),
    // fourth row (faders rather than knobs)
    Fader4_1(f32),
    Fader4_2(f32),
    Fader4_3(f32),
    Fader4_4(f32),
    Fader4_5(f32),
    Fader4_6(f32),
    Fader4_7(f32),
    Fader4_8(f32),
    // top row
    Button1_1(bool),
    Button1_2(bool),
    Button1_3(bool),
    Button1_4(bool),
    Button1_5(bool),
    Button1_6(bool),
    Button1_7(bool),
    Button1_8(bool),
    // bottom row
    Button2_1(bool),
    Button2_2(bool),
    Button2_3(bool),
    Button2_4(bool),
    Button2_5(bool),
    Button2_6(bool),
    Button2_7(bool),
    Button2_8(bool),
}

impl Event {
    pub fn parse(raw: &[u8]) -> Option<Self> {
        use Event::*;

        Some(match raw.get(0)? {
            // fader
            0xb0 | 0xb8 => {
                let val = (*raw.get(2)? as f32) / 127.0;
                debug_assert_eq!(val.max(0.0).min(1.0), val);
                match raw.get(1)? {
                    0x0d => Fader1_1(val),
                    0x0e => Fader1_2(val),
                    0x0f => Fader1_3(val),
                    0x10 => Fader1_4(val),
                    0x11 => Fader1_5(val),
                    0x12 => Fader1_6(val),
                    0x13 => Fader1_7(val),
                    0x14 => Fader1_8(val),

                    0x1d => Fader2_1(val),
                    0x1e => Fader2_2(val),
                    0x1f => Fader2_3(val),
                    0x20 => Fader2_4(val),
                    0x21 => Fader2_5(val),
                    0x22 => Fader2_6(val),
                    0x23 => Fader2_7(val),
                    0x24 => Fader2_8(val),

                    0x31 => Fader3_1(val),
                    0x32 => Fader3_2(val),
                    0x33 => Fader3_3(val),
                    0x34 => Fader3_4(val),
                    0x35 => Fader3_5(val),
                    0x36 => Fader3_6(val),
                    0x37 => Fader3_7(val),
                    0x38 => Fader3_8(val),

                    0x4d => Fader4_1(val),
                    0x4e => Fader4_2(val),
                    0x4f => Fader4_3(val),
                    0x50 => Fader4_4(val),
                    0x51 => Fader4_5(val),
                    0x52 => Fader4_6(val),
                    0x53 => Fader4_7(val),
                    0x54 => Fader4_8(val),

                    _ => return None,
                }
            },

            // button on
            0x90 | 0x98 => match raw.get(1)? {
                0x29 => Button1_1(true),
                0x30 => Button1_2(true),
                0x31 => Button1_3(true),
                0x32 => Button1_4(true),

                0x39 => Button1_5(true),
                0x40 => Button1_6(true),
                0x41 => Button1_7(true),
                0x42 => Button1_8(true),

                0x49 => Button2_1(true),
                0x50 => Button2_2(true),
                0x51 => Button2_3(true),
                0x52 => Button2_4(true),

                0x59 => Button2_5(true),
                0x60 => Button2_6(true),
                0x61 => Button2_7(true),
                0x62 => Button2_8(true),

                _ => return None,
            },

            // button off
            0x80 | 0x88 => match raw.get(1)? {
                0x29 => Button1_1(false),
                0x30 => Button1_2(false),
                0x31 => Button1_3(false),
                0x32 => Button1_4(false),

                0x39 => Button1_5(false),
                0x40 => Button1_6(false),
                0x41 => Button1_7(false),
                0x42 => Button1_8(false),

                0x49 => Button2_1(false),
                0x50 => Button2_2(false),
                0x51 => Button2_3(false),
                0x52 => Button2_4(false),

                0x59 => Button2_5(false),
                0x60 => Button2_6(false),
                0x61 => Button2_7(false),
                0x62 => Button2_8(false),

                _ => return None,
            },
            _ => return None,
        })
    }
}

