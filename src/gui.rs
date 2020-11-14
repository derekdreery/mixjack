use crate::{
    audio::AudioMsg,
    cli::Config,
    data::Metering,
    gui::widgets::{App, Fader, FaderData, Knob},
    Result,
};
use crossbeam_channel as channel;
use druid::{
    lens::Map as LensMap,
    widget::{prelude::*, Flex, Label, List, MainAxisAlignment, Scroll, Switch},
    AppDelegate, AppLauncher, Color, Command, Data, DelegateCtx, ExtEventSink, Handled, Lens,
    LensExt, LocalizedString, Selector, Target, Widget, WidgetExt, WindowDesc,
};
use im::Vector;
use itertools::izip;
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

pub const UPDATE: Selector<UiMsg> = Selector::new("mixjack.update");
const APP_TITLE: LocalizedString<State> = LocalizedString::new("app-title");
const PADDING: f64 = 20.0;

mod widgets;

#[derive(Debug, Data, Clone, Lens, PartialEq)]
pub struct State {
    channels: Vector<ChannelState>,
}

impl State {
    pub fn new(config: &Config) -> Self {
        let mut channels = Vector::new();
        for (name, channel) in config.channels.iter() {
            channels.push_back(ChannelState {
                name: Arc::new(name.to_owned()),
                gain: 0.0,
                metering_on: false,
                metering: Metering::default(),
            });
        }
        State { channels }
    }

    fn update(&mut self, msg: &UiMsg) {
        match msg {
            UiMsg::Metering { channel, metering } => {
                self.channels[*channel].metering = *metering;
            }
            UiMsg::Levels {
                channel,
                level: Level::Gain(gain),
            } => {
                self.channels[*channel].gain = *gain;
            }
            UiMsg::ToggleMetering { channel } => {
                let mut metering_on = &mut self.channels[*channel].metering_on;
                *metering_on = !*metering_on;
            }
        }
    }

    /// Send the required messages to audio to sync its parameters with the ui.
    fn sync_audio(&self, prev: &Self, tx: &channel::Sender<AudioMsg>) -> Result<()> {
        for (idx, (next, prev)) in izip!(self.channels.iter(), prev.channels.iter()).enumerate() {
            if next.gain != prev.gain {
                tx.send(AudioMsg {
                    channel: idx,
                    gain: next.gain,
                })?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Lens)]
pub struct ChannelState {
    name: Arc<String>,
    gain: f64,
    metering_on: bool,
    metering: Metering,
}

impl Data for ChannelState {
    fn same(&self, other: &Self) -> bool {
        Data::same(&self.name, &other.name)
            && Data::same(&self.gain, &other.gain)
            && Data::same(&self.metering_on, &other.metering_on)
            && (Data::same(&self.metering, &other.metering) || !self.metering_on)
    }
}

#[derive(Debug, Clone)]
pub enum UiMsg {
    Levels { channel: usize, level: Level },
    Metering { channel: usize, metering: Metering },
    ToggleMetering { channel: usize },
}

#[derive(Debug, Clone)]
pub enum Level {
    Gain(f64),
}

fn build_ui(tx: channel::Sender<AudioMsg>) -> impl Widget<State> {
    let red_hue = 10.0;
    let yellow_hue = 90.0;
    let green_hue = 120.0;
    let orange_hue = 60.0;
    fn fg_color(hue: f64) -> Color {
        Color::hlc(hue, 80.0, 80.0)
    }
    fn bg_color(hue: f64) -> Color {
        Color::hlc(hue, 30.0, 60.0)
    }

    let red_fader = Knob::new()
        .with_fg(fg_color(red_hue))
        .with_bg(bg_color(red_hue));
    let yellow_fader = Knob::new()
        .with_fg(fg_color(yellow_hue))
        .with_bg(bg_color(yellow_hue));
    let green_fader = Knob::new()
        .with_fg(fg_color(green_hue))
        .with_bg(bg_color(green_hue));
    let orange_fader = Knob::new()
        .with_fg(fg_color(orange_hue))
        .with_bg(bg_color(orange_hue));

    let channels = List::new(|| {
        Flex::column()
            .with_spacer(10.)
            .with_child(Label::raw().lens(ChannelState::name))
            .with_spacer(10.)
            .with_child(Fader::new().lens(LensMap::new(
                |state: &ChannelState| FaderData {
                    position: state.gain,
                    metering: state.metering,
                    show_levels: state.metering_on,
                },
                |mut state, data| {
                    state.gain = data.position;
                },
            )))
            .with_spacer(10.)
            .with_child(Switch::new().lens(ChannelState::metering_on))
            .with_spacer(10.)
    })
    .horizontal()
    .with_spacing(10.);

    App::from_parts(
        Flex::column()
            .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
            .with_child(Scroll::new(channels.lens(State::channels)).horizontal()),
        tx,
    )
}

struct Delegate {
    // used to debounce feedback from the RT thread.
    last_update: Instant,
    //info_acc: PcmInfo,
}

impl Delegate {
    fn new() -> Self {
        Delegate {
            last_update: Instant::now(),
            //info_acc: PcmInfo::default(),
        }
    }
}

impl AppDelegate<State> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut State,
        _env: &Env,
    ) -> Handled {
        if let Some(msg) = cmd.get(UPDATE) {
            data.update(msg);
            Handled::Yes
        } else {
            Handled::No
        }
    }
}

pub fn run(
    tx: channel::Sender<AudioMsg>,
    shutdown_tx: channel::Sender<()>,
    config: Arc<Config>,
) -> Result<(ExtEventSink, JoinHandle<Result>)> {
    let (oneshot_tx, oneshot_rx) = channel::bounded(0);
    // todo check if the ui should be on the main thread?
    let len_channels = config.channels.len() as f64;
    let ui_handle = thread::spawn(move || {
        let window = WindowDesc::new(move || build_ui(tx))
            .title(APP_TITLE.clone().with_placeholder("mixjack"))
            .window_size((
                len_channels * widgets::WIDTH + (len_channels + 1.) * PADDING,
                3.0 * widgets::KNOB_HEIGHT + widgets::FADER_HEIGHT + 5.0 * PADDING,
            ));
        let launcher = AppLauncher::with_window(window).delegate(Delegate::new());
        oneshot_tx.send(launcher.get_external_handle()).unwrap();
        drop(oneshot_tx);

        launcher.launch(State::new(&*config))?;
        shutdown_tx.send(())?;
        Ok(())
    });
    let evt_sink = oneshot_rx.recv().unwrap();
    Ok((evt_sink, ui_handle))
}

// util

fn lens_not<A>(input: impl Lens<A, bool>) -> impl Lens<A, bool> {
    input.map(|v| !v, |data, value| *data = !value)
}

fn label_fixed_width<T: Data>(label: &str) -> impl Widget<T> {
    Label::new(label).center().fix_width(50.0)
}

fn switch() -> impl Widget<bool> {
    Switch::new().center()
}
