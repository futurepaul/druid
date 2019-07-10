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

use druid::kurbo::{Point, Size};
use druid::piet::{Color, RenderContext};
use druid::shell::{runloop, WindowBuilder};
use std::sync::Arc;

use druid::{
    Action, BaseState, BoxConstraints, Data, Env, Event, EventCtx, KeyCode, LayoutCtx, PaintCtx,
    UiMain, UiState, UpdateCtx, Widget,
};

mod draw;
mod pen;

use draw::{draw_active_path, draw_inactive_path};
use pen::Pen;

const BG_COLOR: Color = Color::rgb24(0xfb_fb_fb);
pub(crate) const MIN_POINT_DISTANCE: f64 = 3.0;

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
            tool: Pen::new(),
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
pub(crate) struct Contents {
    paths: Arc<Vec<Path>>,
    active_path: Option<Path>,
}

impl Contents {
    pub(crate) fn paths_mut(&mut self) -> &mut Vec<Path> {
        Arc::make_mut(&mut self.paths)
    }

    fn finish_active(&mut self) {
        if let Some(active) = self.active_path.take() {
            self.paths_mut().push(active);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Mouse {
    Down(Point),
    Drag { start: Point, current: Point },
    Up(Point),
}

/// A trait for editor tools (selection, pen, etc). More concretely, this abstracts
/// away different sets of mouse and keyboard handling behaviour.
pub(crate) trait Tool {
    fn event(&mut self, data: &mut Contents, event: &Event) -> bool;
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

impl Widget<CanvasState> for Canvas {
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
