// Copyright 2018 The xi-editor Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A simple bezier path editor.

use druid::kurbo::{BezPath, Circle, Line, Point, Size};
use druid::piet::{Color, FillRule::NonZero, RenderContext};
use druid::shell::window::MouseButton;
use druid::shell::{runloop, WindowBuilder};
use std::sync::Arc;

use druid::{
    Action, BaseState, BoxConstraints, Data, Env, Event, EventCtx, KeyCode, LayoutCtx, MouseEvent,
    PaintCtx, UiMain, UiState, UpdateCtx, WidgetInner,
};

const BG_COLOR: Color = Color::rgb24(0xfb_fb_fb);
const PATH_COLOR: Color = Color::rgb24(0x00_00_00);
const ON_CURVE_POINT_COLOR: Color = Color::rgb24(0x0b_2b_db);
const OFF_CURVE_POINT_COLOR: Color = Color::rgb24(0xbb_bb_bb);
const OFF_CURVE_HANDLE_COLOR: Color = Color::rgb24(0xbb_bb_bb);

const ON_CURVE_POINT_RADIUS: f64 = 3.5;
const OFF_CURVE_POINT_RADIUS: f64 = 2.;
const MIN_POINT_DISTANCE: f64 = 3.0;

struct Canvas;

#[derive(Debug, Clone, PartialEq)]
enum PathSeg {
    Straight { end: Point },
    Cubic { b1: Point, b2: Point, end: Point },
}

impl PathSeg {
    fn end(&self) -> Point {
        match self {
            PathSeg::Straight { end } => *end,
            PathSeg::Cubic { end, .. } => *end,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
struct Path {
    start: Point,
    segs: Arc<Vec<PathSeg>>,
    trailing_off_curve: Option<Point>,
    closed: bool,
}

#[derive(Debug, Clone)]
struct CanvasState {
    tool: Pen,
    /// The paths in the canvas
    contents: Contents,
}

impl CanvasState {
    fn new() -> Self {
        CanvasState {
            tool: Pen(Mouse::Up(Point::ZERO)),
            contents: Contents::default(),
        }
    }

    fn remove_top_path(&mut self) {
        if self.contents.active_path.take().is_none() {
            Arc::make_mut(&mut self.contents.paths).pop();
        }
    }
}

#[derive(Debug, Clone, Default)]
struct Contents {
    paths: Arc<Vec<Path>>,
    active_path: Option<Path>,
}

#[derive(Debug, Clone, PartialEq)]
enum Mouse {
    Down(Point),
    Drag { start: Point, current: Point },
    Up(Point),
}

/// The bezier pen tool.
#[derive(Debug, Clone, PartialEq)]
struct Pen(Mouse);

/// A trait for editor tools (selection, pen, etc). More concretely, this abstracts
/// away different sets of mouse and keyboard handling behaviour.
trait Tool {
    fn event(&mut self, data: &mut Contents, event: &Event) -> bool;
}

impl Pen {
    fn mouse_down(&mut self, canvas: &mut Contents, event: &MouseEvent) -> bool {
        if event.button != MouseButton::Left {
            return false;
        }

        // are we currently drawing?
        if let Some(active) = canvas.active_path.as_mut() {
            // does this close the path?
            if active.start.distance(event.pos) < MIN_POINT_DISTANCE {
                active.add_point(active.start);
                active.close();
            } else {
                active.add_point(event.pos);
            }
        // we're not drawing, start a new path
        } else {
            canvas.active_path = Some(Path::start(event.pos));
        }
        self.0 = Mouse::Down(event.pos);
        true
    }

    fn double_click(&mut self, canvas: &mut Contents, event: &MouseEvent) -> bool {
        if event.button != MouseButton::Left {
            return false;
        }

        if let Some(active) = canvas.active_path.take() {
            Arc::make_mut(&mut canvas.paths).push(active);
        }
        true
    }

    fn mouse_moved(&mut self, canvas: &mut Contents, event: &MouseEvent) -> bool {
        // does this start or change a drag?
        self.0 = match self.0 {
            Mouse::Up(_) => Mouse::Up(event.pos),
            Mouse::Drag { start, .. } => Mouse::Drag {
                start,
                current: event.pos,
            },
            Mouse::Down(point) => {
                if point.distance(event.pos) > MIN_POINT_DISTANCE {
                    Mouse::Drag {
                        start: point,
                        current: event.pos,
                    }
                } else {
                    Mouse::Down(point)
                }
            }
        };
        if let Mouse::Drag { start, current } = self.0 {
            if let Some(active) = canvas.active_path.as_mut() {
                active.update_for_drag(start, current);
            }
            true
        } else {
            false
        }
    }

    fn mouse_up(&mut self, canvas: &mut Contents, event: &MouseEvent) -> bool {
        if event.button != MouseButton::Left {
            return false;
        }
        if canvas
            .active_path
            .as_ref()
            .map(|p| p.closed)
            .unwrap_or(false)
        {
            Arc::make_mut(&mut canvas.paths).push(canvas.active_path.take().unwrap());
        }
        self.0 = Mouse::Up(event.pos);
        true
    }
}

impl Tool for Pen {
    fn event(&mut self, data: &mut Contents, event: &Event) -> bool {
        match event {
            Event::MouseDown(mouse) if mouse.count == 1 => self.mouse_down(data, mouse),
            Event::MouseDown(mouse) if mouse.count == 2 => self.double_click(data, mouse),
            Event::MouseUp(mouse) => self.mouse_up(data, mouse),
            Event::MouseMoved(mouse) => self.mouse_moved(data, mouse),
            _ => false,
        }
    }
}

impl Path {
    fn start(start: Point) -> Path {
        Path {
            start,
            ..Path::default()
        }
    }

    fn add_point(&mut self, point: Point) {
        if let Some(ctrl) = self.trailing_off_curve.take() {
            self.push_cubic(ctrl, point, point);
        } else {
            self.push_line(point);
        }
    }

    /// Update this path in response to the user click-dragging
    fn update_for_drag(&mut self, start: Point, current: Point) {
        // if necessary, convert the last path segment to a cubic.
        let num_segs = self.segs.len();
        let prev_end = if num_segs >= 2 {
            self.segs.iter().nth(num_segs - 2).unwrap().end()
        } else {
            self.start
        };

        if let Some(last @ PathSeg::Straight { .. }) = Arc::make_mut(&mut self.segs).last_mut() {
            *last = PathSeg::Cubic {
                b1: prev_end,
                b2: start,
                end: start,
            };
        }

        // if this is not the first point, adjust the previous point's second control point.
        if let Some(PathSeg::Cubic { b2, .. }) = Arc::make_mut(&mut self.segs).last_mut() {
            *b2 = start - (current - start);
        }

        self.trailing_off_curve = Some(current);
    }

    fn push_cubic(&mut self, b1: Point, b2: Point, end: Point) {
        let seg = PathSeg::Cubic { b1, b2, end };
        Arc::make_mut(&mut self.segs).push(seg)
    }

    fn push_line(&mut self, end: Point) {
        let seg = PathSeg::Straight { end };
        Arc::make_mut(&mut self.segs).push(seg)
    }

    fn close(&mut self) {
        self.closed = true;
    }
}

// It should be able to get this from a derive macro.
impl Data for CanvasState {
    fn same(&self, other: &Self) -> bool {
        self.contents.same(&other.contents) && self.tool == other.tool
    }
}

impl Data for Contents {
    fn same(&self, other: &Self) -> bool {
        self.paths.same(&other.paths) && self.active_path == other.active_path
    }
}

impl Data for Path {
    fn same(&self, other: &Self) -> bool {
        self.segs.same(&other.segs)
            && self.closed.same(&other.closed)
            && self.trailing_off_curve == other.trailing_off_curve
            && self.start == other.start
    }
}

impl WidgetInner<CanvasState> for Canvas {
    fn paint(&mut self, paint_ctx: &mut PaintCtx, _: &BaseState, data: &CanvasState, _env: &Env) {
        paint_ctx.render_ctx.clear(BG_COLOR);
        for path in data.contents.paths.iter() {
            draw_inactive_path(path, paint_ctx);
        }

        if let Some(active) = data.contents.active_path.as_ref() {
            draw_active_path(active, &data.tool, paint_ctx);
        }
    }

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &CanvasState,
        _env: &Env,
    ) -> Size {
        bc.max()
    }

    fn event(
        &mut self,
        event: &Event,
        ctx: &mut EventCtx,
        data: &mut CanvasState,
        _env: &Env,
    ) -> Option<Action> {
        let CanvasState { tool, contents } = data;

        // first we try to handle things at the top level, and then we pass
        // them to the tool.
        //TODO: move this into a separate function.
        match event {
            Event::KeyUp(key) if key.key_code == KeyCode::Escape => {
                data.remove_top_path();
                ctx.invalidate();
                return None;
            }
            _ => (),
        }

        if tool.event(contents, event) {
            ctx.invalidate();
        }
        None
    }

    fn update(&mut self, _: &mut UpdateCtx, _: Option<&CanvasState>, _: &CanvasState, _: &Env) {}
}

fn draw_inactive_path(path: &Path, paint_ctx: &mut PaintCtx) {
    let mut bez = BezPath::new();
    bez.move_to(path.start);
    for seg in path.segs.iter() {
        match seg {
            PathSeg::Straight { end } => bez.line_to(*end),
            PathSeg::Cubic { b1, b2, end } => bez.curve_to(*b1, *b2, *end),
        }
    }

    if path.closed {
        bez.close_path();
    }

    let path_brush = paint_ctx.render_ctx.solid_brush(PATH_COLOR);
    paint_ctx.render_ctx.stroke(bez, &path_brush, 1.0, None);
}

fn draw_active_path(path: &Path, _tool: &Pen, paint_ctx: &mut PaintCtx) {
    draw_inactive_path(path, paint_ctx);

    let on_curve_brush = paint_ctx.render_ctx.solid_brush(ON_CURVE_POINT_COLOR);
    let off_curve_brush = paint_ctx.render_ctx.solid_brush(OFF_CURVE_POINT_COLOR);
    let ctrl_handle_brush = paint_ctx.render_ctx.solid_brush(OFF_CURVE_HANDLE_COLOR);
    //let red_brush = paint_ctx.render_ctx.solid_brush(Color::rgb24(0xff0000));

    let draw_control_point = |pt, selected, ctx: &mut PaintCtx| {
        let circ = Circle::new(pt, OFF_CURVE_POINT_RADIUS);
        if selected {
            ctx.render_ctx.fill(circ, &off_curve_brush, NonZero);
        } else {
            ctx.render_ctx.stroke(circ, &off_curve_brush, 1.0, None);
        }
    };

    let draw_curve_point = |pt, selected, collapsed, ctx: &mut PaintCtx| {
        let circ = Circle::new(pt, ON_CURVE_POINT_RADIUS);
        if selected {
            ctx.render_ctx.fill(circ, &on_curve_brush, NonZero);
        } else {
            ctx.render_ctx.stroke(circ, &on_curve_brush, 1.0, None);
        }
        if collapsed {
            let circ = Circle::new(pt, OFF_CURVE_POINT_RADIUS);
            ctx.render_ctx.fill(circ, &off_curve_brush, NonZero);
        }
    };

    let draw_control_handle = |pt1, pt2, ctx: &mut PaintCtx| {
        let l = Line::new(pt1, pt2);
        ctx.render_ctx.stroke(l, &ctrl_handle_brush, 1.0, None);
    };

    let mut prev_point = path.start;
    let mut collapsed_point = false;
    for seg in path.segs.iter() {
        let mut collapse_next = false;
        if let PathSeg::Cubic { b1, b2, end } = seg {
            draw_control_handle(prev_point, *b1, paint_ctx);
            draw_control_handle(*b2, *end, paint_ctx);
            draw_control_point(*b1, false, paint_ctx);
            draw_control_point(*b2, false, paint_ctx);
            collapse_next = b2 == end;
        }

        draw_curve_point(prev_point, false, collapsed_point, paint_ctx);
        prev_point = seg.end();
        collapsed_point = collapse_next;
    }

    if let Some(trailing) = path.trailing_off_curve.as_ref() {
        draw_control_handle(prev_point, *trailing, paint_ctx);
        draw_control_point(*trailing, true, paint_ctx);
        let circ = Circle::new(*trailing, OFF_CURVE_POINT_RADIUS);
        paint_ctx.render_ctx.fill(circ, &off_curve_brush, NonZero);
    }

    draw_curve_point(prev_point, true, false, paint_ctx);
}

fn main() {
    druid_shell::init();

    let mut run_loop = runloop::RunLoop::new();
    let mut builder = WindowBuilder::new();
    let state = CanvasState::new();
    let mut state = UiState::new(Canvas, state);
    state.set_active(true);
    builder.set_title("Paths");
    builder.set_handler(Box::new(UiMain::new(state)));
    let window = builder.build().unwrap();
    window.show();
    run_loop.run();
}
