use crate::{
    gui::widgets::{App, Fader, Knob},
    Msg, Result, State,
};
use crossbeam_channel as channel;
use druid::{
    widget::{prelude::*, Flex, MainAxisAlignment},
    AppDelegate, AppLauncher, Color, Command, DelegateCtx, ExtEventSink, LocalizedString, Selector,
    Target, Widget, WidgetExt, WindowDesc,
};
use std::thread::{self, JoinHandle};

pub const UPDATE: Selector = Selector::new("update");
const APP_TITLE: LocalizedString<State> = LocalizedString::new("app-title");
const PADDING: f64 = 20.0;

mod widgets;

fn build_ui(tx: channel::Sender<Msg>) -> impl Widget<State> {
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
                    .with_child(red_fader.clone().lens(State::fader_1_1))
                    .with_child(red_fader.clone().lens(State::fader_1_2))
                    .with_child(yellow_fader.clone().lens(State::fader_1_3))
                    .with_child(yellow_fader.clone().lens(State::fader_1_4))
                    .with_child(green_fader.clone().lens(State::fader_1_5))
                    .with_child(green_fader.clone().lens(State::fader_1_6))
                    .with_child(orange_fader.clone().lens(State::fader_1_7))
                    .with_child(orange_fader.clone().lens(State::fader_1_8)),
            )
            .with_child(
                Flex::row()
                    .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
                    .must_fill_main_axis(true)
                    .with_child(red_fader.clone().lens(State::fader_2_1))
                    .with_child(red_fader.clone().lens(State::fader_2_2))
                    .with_child(yellow_fader.clone().lens(State::fader_2_3))
                    .with_child(yellow_fader.clone().lens(State::fader_2_4))
                    .with_child(green_fader.clone().lens(State::fader_2_5))
                    .with_child(green_fader.clone().lens(State::fader_2_6))
                    .with_child(orange_fader.clone().lens(State::fader_2_7))
                    .with_child(orange_fader.clone().lens(State::fader_2_8)),
            )
            .with_child(
                Flex::row()
                    .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
                    .must_fill_main_axis(true)
                    .with_child(red_fader.clone().lens(State::fader_3_1))
                    .with_child(red_fader.clone().lens(State::fader_3_2))
                    .with_child(yellow_fader.clone().lens(State::fader_3_3))
                    .with_child(yellow_fader.clone().lens(State::fader_3_4))
                    .with_child(green_fader.clone().lens(State::fader_3_5))
                    .with_child(green_fader.clone().lens(State::fader_3_6))
                    .with_child(orange_fader.clone().lens(State::fader_3_7))
                    .with_child(orange_fader.clone().lens(State::fader_3_8)),
            )
            .with_child(
                Flex::row()
                    .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
                    .must_fill_main_axis(true)
                    .with_child(Fader::new().lens(State::fader_4_1))
                    .with_child(Fader::new().lens(State::fader_4_2))
                    .with_child(Fader::new().lens(State::fader_4_3))
                    .with_child(Fader::new().lens(State::fader_4_4))
                    .with_child(Fader::new().lens(State::fader_4_5))
                    .with_child(Fader::new().lens(State::fader_4_6))
                    .with_child(Fader::new().lens(State::fader_4_7))
                    .with_child(Fader::new().lens(State::fader_4_8)),
            ),
        tx,
    )
}

struct Delegate;

impl AppDelegate<State> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: &Target,
        cmd: &Command,
        data: &mut State,
        _env: &Env,
    ) -> bool {
        if cmd.selector == UPDATE {
            let msg: &Msg = cmd.get_object().unwrap();
            data.update(*msg);
            false
        } else {
            true
        }
    }
}

pub fn run(
    tx: channel::Sender<Msg>,
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
        let launcher = AppLauncher::with_window(window).delegate(Delegate);
        oneshot_tx.send(launcher.get_external_handle()).unwrap();
        drop(oneshot_tx);

        launcher.launch(State::default())?;
        shutdown_tx.send(())?;
        Ok(())
    });
    let evt_sink = oneshot_rx.recv().unwrap();
    Ok((evt_sink, ui_handle))
}
