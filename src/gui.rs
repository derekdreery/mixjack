use crate::{
    data::{ChanInfo, Msg, PcmInfo, State, StateChange},
    gui::widgets::{App, Fader, FaderData, Knob},
    Result,
};
use crossbeam_channel as channel;
use druid::{
    lens::Map as LensMap,
    widget::{prelude::*, Flex, Label, MainAxisAlignment, Switch},
    AppDelegate, AppLauncher, Color, Command, Data, DelegateCtx, ExtEventSink, Lens, LensExt,
    LocalizedString, Selector, Target, Widget, WidgetExt, WindowDesc,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub const UPDATE: Selector<Msg> = Selector::new("update");
const APP_TITLE: LocalizedString<GuiState> = LocalizedString::new("app-title");
const PADDING: f64 = 20.0;

mod widgets;

#[derive(Debug, Data, Clone, Lens, Default, PartialEq)]
pub struct GuiState {
    shared: State,
    info: PcmInfo,
    show_levels: [bool; 8],
}

impl GuiState {
    fn process_info(&mut self, info: PcmInfo) {
        self.info = info;
    }
}

fn build_ui(tx: channel::Sender<StateChange>) -> impl Widget<GuiState> {
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

    App::from_parts(
        Flex::column()
            .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
            .with_child(
                Flex::row()
                    .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
                    .must_fill_main_axis(true)
                    .with_child(label_fixed_width("in_1l"))
                    .with_child(label_fixed_width("in_1r"))
                    .with_child(label_fixed_width("in_2l"))
                    .with_child(label_fixed_width("in_2r"))
                    .with_child(label_fixed_width("in_3l"))
                    .with_child(label_fixed_width("in_3r"))
                    .with_child(label_fixed_width("in_4l"))
                    .with_child(label_fixed_width("in_4r")),
            )
            .with_child(
                Flex::row()
                    .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
                    .must_fill_main_axis(true)
                    .with_child(
                        red_fader
                            .clone()
                            .lens(GuiState::shared.then(State::fader_1_1)),
                    )
                    .with_child(
                        red_fader
                            .clone()
                            .lens(GuiState::shared.then(State::fader_1_2)),
                    )
                    .with_child(
                        yellow_fader
                            .clone()
                            .lens(GuiState::shared.then(State::fader_1_3)),
                    )
                    .with_child(
                        yellow_fader
                            .clone()
                            .lens(GuiState::shared.then(State::fader_1_4)),
                    )
                    .with_child(
                        green_fader
                            .clone()
                            .lens(GuiState::shared.then(State::fader_1_5)),
                    )
                    .with_child(
                        green_fader
                            .clone()
                            .lens(GuiState::shared.then(State::fader_1_6)),
                    )
                    .with_child(
                        orange_fader
                            .clone()
                            .lens(GuiState::shared.then(State::fader_1_7)),
                    )
                    .with_child(
                        orange_fader
                            .clone()
                            .lens(GuiState::shared.then(State::fader_1_8)),
                    ),
            )
            .with_child(
                Flex::row()
                    .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
                    .must_fill_main_axis(true)
                    .with_child(
                        red_fader
                            .clone()
                            .lens(GuiState::shared.then(State::fader_2_1)),
                    )
                    .with_child(
                        red_fader
                            .clone()
                            .lens(GuiState::shared.then(State::fader_2_2)),
                    )
                    .with_child(
                        yellow_fader
                            .clone()
                            .lens(GuiState::shared.then(State::fader_2_3)),
                    )
                    .with_child(
                        yellow_fader
                            .clone()
                            .lens(GuiState::shared.then(State::fader_2_4)),
                    )
                    .with_child(
                        green_fader
                            .clone()
                            .lens(GuiState::shared.then(State::fader_2_5)),
                    )
                    .with_child(
                        green_fader
                            .clone()
                            .lens(GuiState::shared.then(State::fader_2_6)),
                    )
                    .with_child(
                        orange_fader
                            .clone()
                            .lens(GuiState::shared.then(State::fader_2_7)),
                    )
                    .with_child(
                        orange_fader
                            .clone()
                            .lens(GuiState::shared.then(State::fader_2_8)),
                    ),
            )
            .with_child(
                Flex::row()
                    .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
                    .must_fill_main_axis(true)
                    .with_child(
                        red_fader
                            .clone()
                            .lens(GuiState::shared.then(State::fader_3_1)),
                    )
                    .with_child(
                        red_fader
                            .clone()
                            .lens(GuiState::shared.then(State::fader_3_2)),
                    )
                    .with_child(
                        yellow_fader
                            .clone()
                            .lens(GuiState::shared.then(State::fader_3_3)),
                    )
                    .with_child(
                        yellow_fader
                            .clone()
                            .lens(GuiState::shared.then(State::fader_3_4)),
                    )
                    .with_child(
                        green_fader
                            .clone()
                            .lens(GuiState::shared.then(State::fader_3_5)),
                    )
                    .with_child(
                        green_fader
                            .clone()
                            .lens(GuiState::shared.then(State::fader_3_6)),
                    )
                    .with_child(
                        orange_fader
                            .clone()
                            .lens(GuiState::shared.then(State::fader_3_7)),
                    )
                    .with_child(
                        orange_fader
                            .clone()
                            .lens(GuiState::shared.then(State::fader_3_8)),
                    ),
            )
            .with_child(
                Flex::row()
                    .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
                    .must_fill_main_axis(true)
                    .with_child(
                        switch().lens(lens_not(GuiState::shared.then(State::filter_passthru_1))),
                    )
                    .with_child(
                        switch().lens(lens_not(GuiState::shared.then(State::filter_passthru_2))),
                    )
                    .with_child(
                        switch().lens(lens_not(GuiState::shared.then(State::filter_passthru_3))),
                    )
                    .with_child(
                        switch().lens(lens_not(GuiState::shared.then(State::filter_passthru_4))),
                    )
                    .with_child(
                        switch().lens(lens_not(GuiState::shared.then(State::filter_passthru_5))),
                    )
                    .with_child(
                        switch().lens(lens_not(GuiState::shared.then(State::filter_passthru_6))),
                    )
                    .with_child(
                        switch().lens(lens_not(GuiState::shared.then(State::filter_passthru_7))),
                    )
                    .with_child(
                        switch().lens(lens_not(GuiState::shared.then(State::filter_passthru_8))),
                    ),
            )
            .with_child(
                Flex::row()
                    .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
                    .must_fill_main_axis(true)
                    .with_child(Fader::new().lens(LensMap::new(
                        |gui_state: &GuiState| FaderData {
                            position: gui_state.shared.fader_4_1,
                            info: gui_state.info.in1,
                            show_levels: gui_state.show_levels[0],
                        },
                        // Ignore fader change
                        |mut gui_state, data| gui_state.shared.fader_4_1 = data.position,
                    )))
                    .with_child(Fader::new().lens(LensMap::new(
                        |gui_state: &GuiState| FaderData {
                            position: gui_state.shared.fader_4_2,
                            info: gui_state.info.in2,
                            show_levels: gui_state.show_levels[1],
                        },
                        |mut gui_state, data| gui_state.shared.fader_4_2 = data.position,
                    )))
                    .with_child(Fader::new().lens(LensMap::new(
                        |gui_state: &GuiState| FaderData {
                            position: gui_state.shared.fader_4_3,
                            info: gui_state.info.in3,
                            show_levels: gui_state.show_levels[2],
                        },
                        |mut gui_state, data| gui_state.shared.fader_4_3 = data.position,
                    )))
                    .with_child(Fader::new().lens(LensMap::new(
                        |gui_state: &GuiState| FaderData {
                            position: gui_state.shared.fader_4_4,
                            info: gui_state.info.in4,
                            show_levels: gui_state.show_levels[3],
                        },
                        |mut gui_state, data| gui_state.shared.fader_4_4 = data.position,
                    )))
                    .with_child(Fader::new().lens(LensMap::new(
                        |gui_state: &GuiState| FaderData {
                            position: gui_state.shared.fader_4_5,
                            info: gui_state.info.in5,
                            show_levels: gui_state.show_levels[4],
                        },
                        |mut gui_state, data| gui_state.shared.fader_4_5 = data.position,
                    )))
                    .with_child(Fader::new().lens(LensMap::new(
                        |gui_state: &GuiState| FaderData {
                            position: gui_state.shared.fader_4_6,
                            info: gui_state.info.in6,
                            show_levels: gui_state.show_levels[5],
                        },
                        |mut gui_state, data| gui_state.shared.fader_4_6 = data.position,
                    )))
                    .with_child(Fader::new().lens(LensMap::new(
                        |gui_state: &GuiState| FaderData {
                            position: gui_state.shared.fader_4_7,
                            info: gui_state.info.in7,
                            show_levels: gui_state.show_levels[6],
                        },
                        |mut gui_state, data| gui_state.shared.fader_4_7 = data.position,
                    )))
                    .with_child(Fader::new().lens(LensMap::new(
                        |gui_state: &GuiState| FaderData {
                            position: gui_state.shared.fader_4_8,
                            info: gui_state.info.in8,
                            show_levels: gui_state.show_levels[7],
                        },
                        |mut gui_state, data| gui_state.shared.fader_4_8 = data.position,
                    ))),
            ),
        tx,
    )
}

struct Delegate {
    // used to debounce feedback from the RT thread.
    last_update: Instant,
    info_acc: PcmInfo,
}

impl Delegate {
    fn new() -> Self {
        Delegate {
            last_update: Instant::now(),
            info_acc: PcmInfo::default(),
        }
    }
}

const DURATION: Duration = Duration::from_millis(200);

impl AppDelegate<GuiState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut GuiState,
        _env: &Env,
    ) -> bool {
        if let Some(msg) = cmd.get(UPDATE) {
            match msg {
                Msg::StateChange(sc) => {
                    data.shared.update(*sc);
                }
                Msg::PcmInfo(info) => {
                    let now = Instant::now();
                    self.info_acc.merge(info);
                    if now - self.last_update > DURATION {
                        // dispatch command
                        data.process_info(self.info_acc);
                        // reset
                        self.last_update = now;
                        self.info_acc.clear();
                    }
                }
            };
            false
        } else {
            true
        }
    }
}

pub fn run(
    tx: channel::Sender<StateChange>,
    shutdown_tx: channel::Sender<()>,
) -> Result<(ExtEventSink, JoinHandle<Result>)> {
    let (oneshot_tx, oneshot_rx) = channel::bounded(0);
    // todo check if the ui should be on the main thread?
    let ui_handle = thread::spawn(move || {
        let window = WindowDesc::new(move || build_ui(tx))
            .title(APP_TITLE.clone().with_placeholder("jack-mixer"))
            .window_size((
                8.0 * widgets::WIDTH + 9.0 * PADDING,
                3.0 * widgets::KNOB_HEIGHT + widgets::FADER_HEIGHT + 5.0 * PADDING,
            ));
        let launcher = AppLauncher::with_window(window).delegate(Delegate::new());
        oneshot_tx.send(launcher.get_external_handle()).unwrap();
        drop(oneshot_tx);

        launcher.launch(GuiState::default())?;
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
