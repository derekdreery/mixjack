use crate::{Msg, State};
// todo bounded crossbeam channels are not completely lock-free - maybe there is a better choice.
use crossbeam_channel as channel;
use druid::{
    piet::{
        kurbo::{Arc, BezPath, Line},
        Brush,
    },
    widget::prelude::*,
    Color, Data, Insets, MouseButton, MouseEvent, Point, Rect, Vec2, Widget, WidgetPod,
};
use std::f64::consts::FRAC_PI_4;

pub const WIDTH: f64 = 50.0;
pub const KNOB_HEIGHT: f64 = 50.0;
pub const FADER_HEIGHT: f64 = 200.0;

const SLIDER_HEIGHT: f64 = 20.0;

pub struct App {
    inner: WidgetPod<State, Box<dyn Widget<State>>>,
    tx: channel::Sender<Msg>,
}

impl App {
    pub fn from_parts(inner: impl Widget<State> + 'static, tx: channel::Sender<Msg>) -> Self {
        App {
            inner: WidgetPod::new(inner).boxed(),
            tx,
        }
    }
}

impl Widget<State> for App {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut State, env: &Env) {
        let old_data = data.clone();
        self.inner.event(ctx, event, data, env);
        if *data != old_data {
            data.compute_changes(&old_data, &self.tx).unwrap();
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &State, env: &Env) {
        self.inner.lifecycle(ctx, event, data, env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &State, new_data: &State, env: &Env) {
        self.inner.update(ctx, new_data, env)
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        new_data: &State,
        env: &Env,
    ) -> Size {
        self.inner.set_layout_rect(bc.max().to_rect());
        self.inner.layout(ctx, bc, new_data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &State, env: &Env) {
        self.inner.paint(ctx, data, env)
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
            Event::MouseMoved(MouseEvent {
                button: MouseButton::Left,
                window_pos,
                ..
            }) => {
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
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &f64,
        _env: &Env,
    ) -> Size {
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
        ctx.stroke(arc(bg), &bg_brush, 2.0);
        ctx.stroke(arc(fg), &fg_brush, 2.0);
        if data > 0.0 {
            ctx.stroke(needle, &fg_brush, 2.0);
        } else {
            ctx.stroke(needle, &bg_brush, 2.0);
        }
    }
}

#[derive(Debug, Data, Clone)]
pub struct Fader {
    drag_start: Option<DragStart>,
}

impl Fader {
    pub fn new() -> Self {
        Fader { drag_start: None }
    }
}

impl Widget<f64> for Fader {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut f64, _env: &Env) {
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
            Event::MouseMoved(MouseEvent {
                button: MouseButton::Left,
                window_pos,
                ..
            }) => {
                if let Some(drag_start) = self.drag_start {
                    *data = (drag_start.widget_val
                        + (drag_start.mouse_y - window_pos.y)
                            / (ctx.size().height - SLIDER_HEIGHT).max(0.0))
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
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &f64,
        _env: &Env,
    ) -> Size {
        bc.constrain(Size::new(WIDTH, FADER_HEIGHT))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &f64, _env: &Env) {
        // Clamp the relative position.
        let data = (*data).min(1.0).max(0.0);
        let light_brush = ctx.solid_brush(Color::WHITE);
        let dark_brush = ctx.solid_brush(Color::grey(0.5));
        let black_brush = ctx.solid_brush(Color::BLACK);

        let bounds = ctx
            .size()
            // a widget origin is always (0,0)
            .to_rect()
            // leave room for the fader
            .inset(Insets::uniform_xy(0.0, -0.5 * SLIDER_HEIGHT));
        let center = bounds.center();

        let top = Point::new(center.x, bounds.min_y());
        let bottom = Point::new(center.x, bounds.max_y());
        let fader_center = bottom.lerp(top, data);

        ctx.stroke(Line::new(top, bottom), &dark_brush, 2.0);
        ctx.stroke(Line::new(fader_center, bottom), &light_brush, 2.0);
        fader(
            from_center_size(fader_center, (bounds.size().width, SLIDER_HEIGHT)),
            if data == 0.0 {
                &dark_brush
            } else {
                &light_brush
            },
            &black_brush,
            ctx,
        )
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

    ctx.fill(bounds, bg_brush);
    ctx.stroke(bounds, fg_brush, 1.0);
    ctx.stroke(center_line, fg_brush, 2.0);
}

fn arc(arc: Arc) -> BezPath {
    let start = circle_point(arc.center, arc.radii, arc.start_angle);

    let mut out = BezPath::new();
    out.move_to(start);
    for el in arc.append_iter(1.0) {
        out.push(el);
    }
    out
}

fn circle_point(center: Point, radii: Vec2, angle: f64) -> Point {
    Point {
        x: center.x - angle.sin() * radii.y,
        y: center.y + angle.cos() * radii.x,
    }
}

fn from_center_size(center: impl Into<Point>, size: impl Into<Size>) -> Rect {
    let size = 0.5 * size.into();
    let center = center.into();
    Rect {
        x0: center.x - size.width,
        y0: center.y - size.height,
        x1: center.x + size.width,
        y1: center.y + size.height,
    }
}
