use crate::{
    audio::{AudioMsg, AudioMsgKind},
    cli::Config,
    data::{ChannelMode, Metering},
    gui::widgets::{Fader, FaderData, Knob, Syncer},
    Result,
};
use crossbeam_channel as channel;
use druid::{
    lens::{Constant, Map as LensMap},
    widget::{prelude::*, Flex, Label, List, MainAxisAlignment, RadioGroup, Scroll, Switch},
    AppDelegate, AppLauncher, ArcStr, Color, Command, Data, DelegateCtx, ExtEventSink, Handled,
    Lens, LensExt, LocalizedString, MenuDesc, MenuItem, Selector, Target, Widget, WidgetExt,
    WindowDesc,
};
use druid_graphs::{LineChart, LineChartData, LineChartDataLensBuilder, Range};
use im::{vector, Vector};
use itertools::izip;
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const PADDING: f64 = 20.0;
pub const UPDATE: Selector<UiMsg> = Selector::new("mixjack.update");
const SHOW_LOW_PASS: Selector<()> = Selector::new("mixjack.show-low-pass");
const SHOW_INPUT_SPECTRUM: Selector<()> = Selector::new("mixjack.show-input-spectrum");
const SHOW_OUTPUT_SPECTRUM: Selector<()> = Selector::new("mixjack.show-output-spectrum");

const APP_TITLE: LocalizedString<State> = LocalizedString::new("app-title");
const SPECTRA_MENU: LocalizedString<State> = LocalizedString::new("mixjack.spectra-menu");
const LOW_PASS_MENU_ITEM: LocalizedString<State> =
    LocalizedString::new("mixjack.low-pass-menu-item");
const INPUT_SPECTRUM_MENU_ITEM: LocalizedString<State> =
    LocalizedString::new("mixjack.input-spectrum-menu-item");
const OUTPUT_SPECTRUM_MENU_ITEM: LocalizedString<State> =
    LocalizedString::new("mixjack.output-spectrum-menu-item");

mod widgets;

#[derive(Debug, Data, Clone, Lens, PartialEq)]
pub struct State {
    /// The spectrum of the low pass filter, in half-complex form
    /// `(r0, r1, .. rn/2, i(n+1)/2-1 .., i1)`.
    // TODO use Arc<Vec> because we don't do random changes.
    low_pass_spectrum: Vector<f64>,
    audio_in_spectrum: Vector<f64>,
    audio_out_spectrum: Vector<f64>,
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
                mode: ChannelMode::default(),
            });
        }
        State {
            low_pass_spectrum: vector![],
            audio_in_spectrum: vector![],
            audio_out_spectrum: vector![],
            channels,
        }
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
            UiMsg::LowPassSpectrum(mod_spectrum) => {
                self.low_pass_spectrum = mod_spectrum.iter().map(|v| *v as f64).collect();
            }
            UiMsg::AudioInSpectrum(mod_spectrum) => {
                self.audio_in_spectrum = mod_spectrum.iter().map(|v| *v as f64).collect();
            }
            UiMsg::AudioOutSpectrum(mod_spectrum) => {
                self.audio_out_spectrum = mod_spectrum.iter().map(|v| *v as f64).collect();
            }
        }
    }

    /// Send the required messages to audio to sync its parameters with the ui.
    fn sync_audio(&self, prev: &Self, tx: &channel::Sender<AudioMsg>) -> Result<()> {
        for (idx, (next, prev)) in izip!(self.channels.iter(), prev.channels.iter()).enumerate() {
            if next.gain != prev.gain {
                tx.send(AudioMsg {
                    channel: idx,
                    kind: AudioMsgKind::Gain(next.gain),
                })?;
            }
            if next.mode != prev.mode {
                tx.send(AudioMsg {
                    channel: idx,
                    kind: AudioMsgKind::Mode(next.mode),
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
    mode: ChannelMode,
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
    LowPassSpectrum(Vec<f32>),
    AudioInSpectrum(Vec<f32>),
    AudioOutSpectrum(Vec<f32>),
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
            .with_child(
                RadioGroup::new(
                    [
                        ("normal", ChannelMode::Normal),
                        ("bypass", ChannelMode::Bypass),
                        ("mute", ChannelMode::Mute),
                    ]
                    .iter()
                    .copied(),
                )
                .lens(ChannelState::mode),
            )
            .with_spacer(10.)
    })
    .horizontal()
    .with_spacing(10.);

    Flex::column()
        .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
        .with_child(Scroll::new(channels.lens(State::channels)).horizontal())
        .padding(10.)
        .controller(Syncer::new(tx))
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
        ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut State,
        _env: &Env,
    ) -> Handled {
        if let Some(msg) = cmd.get(UPDATE) {
            data.update(msg);
            Handled::Yes
        } else if let Some(()) = cmd.get(SHOW_LOW_PASS) {
            ctx.new_window(low_pass_window());
            Handled::Yes
        } else if let Some(()) = cmd.get(SHOW_INPUT_SPECTRUM) {
            ctx.new_window(input_spectrum_window());
            Handled::Yes
        } else if let Some(()) = cmd.get(SHOW_OUTPUT_SPECTRUM) {
            ctx.new_window(output_spectrum_window());
            Handled::Yes
        } else {
            Handled::No
        }
    }
}

fn main_menu() -> MenuDesc<State> {
    MenuDesc::new(SPECTRA_MENU.with_placeholder("Spectra"))
        .append(MenuItem::new(
            LOW_PASS_MENU_ITEM.with_placeholder("Low pass filter"),
            SHOW_LOW_PASS,
        ))
        .append(MenuItem::new(
            INPUT_SPECTRUM_MENU_ITEM.with_placeholder("Input spectrum"),
            SHOW_INPUT_SPECTRUM,
        ))
        .append(MenuItem::new(
            OUTPUT_SPECTRUM_MENU_ITEM.with_placeholder("Output spectrum"),
            SHOW_OUTPUT_SPECTRUM,
        ))
}

fn low_pass_window() -> WindowDesc<State> {
    WindowDesc::new(|| {
        LineChart::new().lens(
            line_chart_base()
                .title(Constant(ArcStr::from("Low pass filter")))
                .y_data(State::low_pass_spectrum)
                .build(),
        )
    })
    .title(LOW_PASS_MENU_ITEM.with_placeholder("Low pass filter"))
}

fn input_spectrum_window() -> WindowDesc<State> {
    WindowDesc::new(|| {
        WidgetExt::lens(
            LineChart::new(),
            line_chart_base()
                .title(Constant(ArcStr::from("Input spectrum")))
                .y_data(State::audio_in_spectrum)
                .build(),
        )
    })
    .title(INPUT_SPECTRUM_MENU_ITEM.with_placeholder("Input spectrum"))
}

fn output_spectrum_window() -> WindowDesc<State> {
    WindowDesc::new(|| {
        WidgetExt::lens(
            LineChart::new(),
            line_chart_base()
                .title(Constant(ArcStr::from("Output spectrum")))
                .y_data(State::audio_out_spectrum)
                .build(),
        )
    })
    .title(OUTPUT_SPECTRUM_MENU_ITEM.with_placeholder("Output spectrum"))
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
            .title(APP_TITLE.with_placeholder("mixjack"))
            .menu(main_menu())
            .window_size((
                len_channels * widgets::WIDTH + (len_channels + 1.) * PADDING,
                3.0 * widgets::KNOB_HEIGHT + widgets::FADER_HEIGHT + 5.0 * PADDING,
            ));
        let launcher = AppLauncher::with_window(window)
            .configure_env(|env, _| druid_graphs::add_to_env(env))
            .delegate(Delegate::new());
        oneshot_tx.send(launcher.get_external_handle()).unwrap();
        drop(oneshot_tx);

        launcher.launch(State::new(&*config))?;
        shutdown_tx.send(())?;
        Ok(())
    });
    let evt_sink = oneshot_rx.recv().unwrap();
    Ok((evt_sink, ui_handle))
}

fn line_chart_base<L1, L2>() -> LineChartDataLensBuilder<
    L1,
    Constant<ArcStr>,
    Constant<Option<Range>>,
    Constant<bool>,
    Constant<bool>,
    Constant<Option<Vector<f64>>>,
    Constant<Option<Range>>,
    Constant<bool>,
    Constant<bool>,
    L2,
> {
    LineChartData::<ArcStr, ArcStr>::lens_builder()
        .x_axis_label(Constant(ArcStr::from("Frequency (Hz)")))
        .x_range(Constant(None))
        .draw_x_tick_labels(Constant(true))
        .draw_x_axis(Constant(false))
        // hard code bin frequencies for now (assume sampling frequency is 44_100 Hz)
        .x_data(Constant(bins()))
        .y_range(Constant(Some(Range::new(0., 1.))))
        .draw_y_tick_labels(Constant(true))
        .draw_y_axis(Constant(true))
}

fn bins() -> Option<Vector<f64>> {
    Some(
        (0..256)
            .into_iter()
            .map(|idx| idx as f64 * 44_100. / 256.)
            .collect::<Vector<_>>(),
    )
}
