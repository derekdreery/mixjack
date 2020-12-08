use crate::{audio::AudioMsg, data::Metering, gui::State};
use crossbeam_channel as channel;
use druid::{
    piet::{
        kurbo::{Arc, BezPath, Line, PathEl},
        Brush,
    },
    theme,
    widget::{prelude::*, Controller},
    Color, Data, Insets, MouseButton, MouseEvent, Point, Rect, Vec2, Widget, WidgetPod,
};
use std::f64::consts::FRAC_PI_4;

pub const WIDTH: f64 = 50.0;
pub const KNOB_HEIGHT: f64 = 50.0;
pub const FADER_HEIGHT: f64 = 200.0;

const SLIDER_HEIGHT: f64 = 20.0;

pub struct Syncer {
    tx: channel::Sender<AudioMsg>,
}

impl<W: Widget<State>> Controller<State, W> for Syncer {
    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut UpdateCtx,
        old_data: &State,
        data: &State,
        env: &Env,
    ) {
        child.update(ctx, old_data, data, env);
        data.sync_audio(old_data, &self.tx).unwrap();
    }
}

impl Syncer {
    pub fn new(tx: channel::Sender<AudioMsg>) -> Self {
        Self { tx }
    }
}

#[derive(Debug, Data, Copy, Clone)]
pub struct DragStart {
    mouse_y: f64,
    widget_val: f64,
}

#[derive(Debug, Data, Clone)]
pub struct Knob {
    fg_color: Color,
    bg_color: Color,
    drag_start: Option<DragStart>,
}

impl Knob {
    pub fn new() -> Self {
        Knob {
            fg_color: Color::WHITE,
            bg_color: Color::rgb(50, 50, 50),
            drag_start: None,
        }
    }

    pub fn with_fg(mut self, color: Color) -> Self {
        self.fg_color = color;
        self
    }

    pub fn with_bg(mut self, color: Color) -> Self {
        self.bg_color = color;
        self
    }
}

impl Widget<f64> for Knob {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut f64, _env: &Env) {
        const SCALE_FACTOR: f64 = 0.01;
        match event {
            Event::MouseDown(MouseEvent {
                button: MouseButton::Left,
                window_pos,
                ..
            }) => {
                ctx.set_active(true);
                self.drag_start = Some(DragStart {
                    mouse_y: window_pos.y,
                    widget_val: *data,
                });
            }
            Event::MouseMove(MouseEvent { window_pos, .. }) => {
                if let Some(drag_start) = self.drag_start {
                    *data = (drag_start.widget_val
                        + (drag_start.mouse_y - window_pos.y) * SCALE_FACTOR)
                        .max(0.0)
                        .min(1.0);
                }
            }
            Event::MouseUp(MouseEvent {
                button: MouseButton::Left,
                ..
            }) => {
                self.drag_start = None;
                ctx.set_active(false);
            }
            _ => (),
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old: &f64, new: &f64, _env: &Env) {
        if old != new {
            ctx.request_paint();
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &f64, _env: &Env) {}

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &f64,
        _env: &Env,
    ) -> Size {
        ctx.set_paint_insets(Insets::uniform(1.0));
        bc.constrain(Size::new(WIDTH, KNOB_HEIGHT))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &f64, _env: &Env) {
        // Clamp the relative position.
        let data = (*data).min(1.0).max(0.0);

        let center = Rect::from_origin_size(Point::ORIGIN, ctx.size()).center();
        let full_sweep = 6.0 * FRAC_PI_4;
        // Flip the radii as we've rotated by 90 degrees.
        let radii = Vec2 {
            x: center.y,
            y: center.x,
        };

        let bg = Arc {
            center,
            radii,
            start_angle: FRAC_PI_4,
            sweep_angle: full_sweep,
            x_rotation: 2.0 * FRAC_PI_4,
        };
        let fg = Arc {
            center,
            radii,
            start_angle: FRAC_PI_4,
            sweep_angle: data * full_sweep,
            x_rotation: 2.0 * FRAC_PI_4,
        };
        let needle = Line::new(
            center,
            circle_point(fg.center, 0.8 * fg.radii, fg.start_angle + fg.sweep_angle),
        );
        let bg_brush = ctx.solid_brush(self.bg_color.clone());
        let fg_brush = ctx.solid_brush(self.fg_color.clone());
        ctx.stroke(bg, &bg_brush, 2.0);
        ctx.stroke(fg, &fg_brush, 2.0);
        if data > 0.0 {
            ctx.stroke(needle, &fg_brush, 2.0);
        } else {
            ctx.stroke(needle, &bg_brush, 2.0);
        }
    }
}

#[derive(Debug, Data, Clone, PartialEq)]
pub struct FaderData {
    /// position of the fader, between 0 and 1.
    pub position: f64,
    /// Feedback from the mixer.
    pub metering: Metering,
    /// Show the feedback from the mixer
    pub show_levels: bool,
}

#[derive(Debug, Data, Clone)]
pub struct Fader {
    drag_start: Option<DragStart>,
    all_time_max_in: f64,
}

impl Fader {
    pub fn new() -> Self {
        Fader {
            drag_start: None,
            all_time_max_in: 0.0,
        }
    }
}

impl Widget<FaderData> for Fader {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut FaderData, _env: &Env) {
        match event {
            Event::MouseDown(MouseEvent {
                button: MouseButton::Left,
                window_pos,
                ..
            }) => {
                ctx.set_active(true);
                self.drag_start = Some(DragStart {
                    mouse_y: window_pos.y,
                    widget_val: data.position,
                });
            }
            Event::MouseMove(MouseEvent {
                buttons,
                window_pos,
                ..
            }) => {
                if buttons.contains(MouseButton::Left) {
                    if let Some(drag_start) = self.drag_start {
                        data.position = (drag_start.widget_val
                            + (drag_start.mouse_y - window_pos.y)
                                / (ctx.size().height - SLIDER_HEIGHT).max(0.0))
                        .max(0.0)
                        .min(1.0);
                    }
                }
            }
            Event::MouseUp(e) => {
                self.drag_start = None;
                ctx.set_active(false);
            }
            _ => (),
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old: &FaderData, new: &FaderData, _env: &Env) {
        if old != new {
            self.all_time_max_in = new.metering.max_in;
            ctx.request_paint();
        }
    }

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &FaderData,
        _env: &Env,
    ) {
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &FaderData,
        _env: &Env,
    ) -> Size {
        ctx.set_paint_insets(Insets::uniform(1.0));
        bc.constrain(Size::new(WIDTH, FADER_HEIGHT))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &FaderData, _env: &Env) {
        // Clamp the relative position.
        let position = data.position.min(1.0).max(0.0);
        let max_in = data.metering.max_in.min(1.0).max(0.0);
        let rms_in = data.metering.rms_in.min(1.0).max(0.0);
        let max_out = data.metering.max_out.min(1.0).max(0.0);
        let rms_out = data.metering.rms_out.min(1.0).max(0.0);

        let light_brush = ctx.solid_brush(Color::WHITE);
        let dark_brush = ctx.solid_brush(Color::grey(0.5));
        let black_brush = ctx.solid_brush(Color::BLACK);
        let max_brush = ctx.solid_brush(Color::rgba(0.0, 1.0, 0.0, 0.2));
        let rms_brush = ctx.solid_brush(Color::rgb(0.0, 0.6, 0.0));

        let bounds = ctx
            .size()
            // a widget origin is always (0,0)
            .to_rect()
            // leave room for the fader
            .inset(Insets::uniform_xy(0.0, -0.5 * SLIDER_HEIGHT));
        let center = bounds.center();

        let top = Point::new(center.x, bounds.min_y());
        let bottom = Point::new(center.x, bounds.max_y());
        let fader_center = bottom.lerp(top, position);

        let max_in_top = bottom.lerp(top, max_in).y;
        let rms_in_top = bottom.lerp(top, rms_in).y;
        let max_out_top = bottom.lerp(top, max_out).y;
        let rms_out_top = bottom.lerp(top, rms_out).y;

        let level_start_x = lerp(bounds.x0, bounds.x1, 0.2);
        let level_mid_x = lerp(bounds.x0, bounds.x1, 0.5);
        let level_end_x = lerp(bounds.x0, bounds.x1, 0.8);
        if data.show_levels {
            ctx.fill(
                Rect::from_points((level_start_x, rms_in_top), (level_mid_x, bounds.y1)),
                &rms_brush,
            );
            ctx.fill(
                Rect::from_points((level_mid_x, rms_out_top), (level_end_x, bounds.y1)),
                &rms_brush,
            );
        }

        // draw fader
        ctx.stroke(Line::new(top, bottom), &dark_brush, 2.0);
        ctx.stroke(Line::new(fader_center, bottom), &light_brush, 2.0);
        fader(
            Rect::from_center_size(fader_center, (bounds.size().width, SLIDER_HEIGHT)),
            if position == 0.0 {
                &dark_brush
            } else {
                &light_brush
            },
            &black_brush,
            ctx,
        );

        // draw sound level
        if data.show_levels {
            ctx.fill(
                Rect::new(level_start_x, max_in_top, level_mid_x, bounds.y1),
                &max_brush,
            );
            ctx.fill(
                Rect::new(level_mid_x, max_out_top, level_end_x, bounds.y1),
                &max_brush,
            );
        }
    }
}

/// A small radio button that contains an indicator light if currently clicked.
pub struct LightRadio<E> {
    color: Color,
    variant: E,
    /// The variant to go to if the button is selected then clicked.
    off_variant: E,
}

impl<E> LightRadio<E> {
    pub fn new(color: Color, variant: E, off_variant: E) -> Self {
        LightRadio {
            color,
            variant,
            off_variant,
        }
    }
}

impl<E: Copy + PartialEq> Widget<E> for LightRadio<E> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut E, _env: &Env) {
        match event {
            Event::MouseDown(MouseEvent {
                button: MouseButton::Left,
                window_pos,
                ..
            }) => {
                ctx.set_active(true);
            }
            Event::MouseUp(e) => {
                if ctx.is_active() && ctx.is_hot() {
                    if data == &self.variant {
                        *data = self.off_variant;
                    } else {
                        *data = self.variant;
                    }
                }
                ctx.set_active(false);
            }
            _ => (),
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old: &E, new: &E, _env: &Env) {
        if old != new {
            ctx.request_paint();
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &E, _env: &Env) {}

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &E, _env: &Env) -> Size {
        bc.constrain(Size::new(20., 10.))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &E, env: &Env) {
        let rect = ctx.size().to_rect();
        ctx.stroke(rect.inset(-1.), &env.get(theme::BORDER_DARK), 1.0);
        if data == &self.variant {
            ctx.fill(rect, &self.color);
        }
    }
}

/// Paint the actual physical fader that you move up and down.
fn fader(bounds: Rect, fg_brush: &Brush, bg_brush: &Brush, ctx: &mut PaintCtx) {
    const MIDDLE_LINE_BORDER: f64 = SLIDER_HEIGHT * 0.1;
    let middle_y = bounds.center().y;
    let center_line = Line::new(
        (bounds.x0 + MIDDLE_LINE_BORDER, middle_y),
        (bounds.x1 - MIDDLE_LINE_BORDER, middle_y),
    );

    // TODO make this function independent of position (so that the path can be cached)
    const CURVE_FACTOR: f64 = 0.15;
    let shape = [
        PathEl::MoveTo(Point::new(bounds.x0, bounds.y0)),
        PathEl::QuadTo(
            bounds.center() - Vec2::new(0.0, bounds.height() * (0.5 - CURVE_FACTOR)),
            Point::new(bounds.x1, bounds.y0),
        ),
        PathEl::LineTo(Point::new(bounds.x1, bounds.y1)),
        PathEl::QuadTo(
            bounds.center() + Vec2::new(0.0, bounds.height() * (0.5 - CURVE_FACTOR)),
            Point::new(bounds.x0, bounds.y1),
        ),
        PathEl::ClosePath,
    ];

    ctx.fill(&shape[..], bg_brush);
    ctx.stroke(&shape[..], fg_brush, 1.5);
    ctx.stroke(center_line, fg_brush, 2.0);
}

fn circle_point(center: Point, radii: Vec2, angle: f64) -> Point {
    Point {
        x: center.x - angle.sin() * radii.y,
        y: center.y + angle.cos() * radii.x,
    }
}

fn lerp(start: f64, end: f64, t: f64) -> f64 {
    start + (end - start) * t
}
