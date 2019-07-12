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

use std::any::Any;
use std::fmt::Debug;

use druid::kurbo::{Point, Rect, Size};
use druid::piet::{Color, RenderContext};
use druid::shell::window::Cursor;
use druid::shell::{runloop, WindowBuilder};
use std::sync::Arc;

use druid::{
    Action, BaseState, BoxConstraints, Data, Env, Event, EventCtx, KeyCode, LayoutCtx, PaintCtx,
    UiMain, UiState, UpdateCtx, Widget, WidgetPod,
};

mod draw;
mod pen;
mod select;
mod toolbar;

use draw::draw_paths;
use pen::Pen;
use select::Select;
use toolbar::{Toolbar, ToolbarState};

const BG_COLOR: Color = Color::rgb24(0xfb_fb_fb);
const TOOLBAR_POSITION: Point = Point::new(8., 8.);

pub(crate) const MIN_POINT_DISTANCE: f64 = 3.0;

struct Canvas {
    toolbar: WidgetPod<ToolbarState, Toolbar>,
}

impl Canvas {
    fn new() -> Self {
        Canvas {
            toolbar: WidgetPod::new(Toolbar::default()),
        }
    }
}

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
    tool: Box<dyn Tool>,
    /// The paths in the canvas
    contents: Contents,
    toolbar: ToolbarState,
}

impl CanvasState {
    fn new() -> Self {
        CanvasState {
            tool: Box::new(Pen::new()),
            contents: Contents::default(),
            toolbar: ToolbarState::basic(),
        }
    }

    fn remove_top_path(&mut self) {
        Arc::make_mut(&mut self.contents.paths).pop();
    }

    fn update_tool_if_necessary(&mut self) {
        if self.toolbar.selected_item().name == self.tool.name() {
            return;
        }

        let new_tool: Box<dyn Tool> = match self.toolbar.selected_item().name.as_str() {
            "pen" => Box::new(Pen::new()),
            _ => Box::new(Select::new()),
        };
        self.tool = new_tool;
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Contents {
    paths: Arc<Vec<Path>>,
    /// Selected points, using overall idx as an id.
    selection: Arc<Vec<usize>>,
}

impl Contents {
    pub(crate) fn paths_mut(&mut self) -> &mut Vec<Path> {
        Arc::make_mut(&mut self.paths)
    }

    pub(crate) fn selection_mut(&mut self) -> &mut Vec<usize> {
        Arc::make_mut(&mut self.selection)
    }

    /// Return the index of the path that is currently drawing. To be currently
    /// drawing, there must be a single currently selected point.
    fn active_path_idx(&self) -> Option<usize> {
        if self.selection.len() == 1 {
            let mut points = 0;
            let mut path_idx = 0;
            for path in self.paths.iter() {
                points += path.num_points();
                if points > self.selection[0] {
                    break;
                }
                path_idx += 1;
            }
            Some(path_idx)
        } else {
            None
        }
    }

    pub(crate) fn active_path_mut(&mut self) -> Option<&mut Path> {
        match self.active_path_idx() {
            Some(idx) => self.paths_mut().get_mut(idx),
            None => None,
        }
    }

    pub(crate) fn new_path(&mut self, start: Point) {
        let idx = self.paths.iter().map(Path::num_points).sum();
        self.paths_mut().push(Path::start(start));
        self.selection_mut().clear();
        self.selection_mut().push(idx);
    }

    pub(crate) fn add_point(&mut self, point: Point) {
        let new_points = match self.active_path_mut() {
            Some(path) => {
                let result = if path.trailing_off_curve.is_some() {
                    3
                } else {
                    1
                };
                path.add_point(point);
                if path.start == point {
                    path.close();
                }
                result
            }
            None => return self.new_path(point),
        };

        self.selection_mut()[0] += new_points;
        eprintln!("SEL: {}", self.selection[0]);
    }

    pub(crate) fn update_for_drag(&mut self, start: Point, end: Point) {
        let new_points = self
            .active_path_mut()
            .map(|p| p.update_for_drag(start, end))
            .unwrap_or(0);

        self.selection_mut()[0] += new_points;
        eprintln!("SEL: {}", self.selection[0]);
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
pub(crate) trait Tool: Debug + Any {
    /// Called when the tool should process some event. The tool should modify
    /// `data` as necessary, and return `true` if the event is handled.
    fn event(&mut self, data: &mut Contents, event: &Event) -> bool;
    /// The current position of the tool. Used to set the new tool's position
    /// when the tool changes.

    fn boxed_clone(&self) -> Box<dyn Tool>;
    fn same_impl(&self, other: &dyn Any) -> bool;
    fn name(&self) -> &str;
}

impl Clone for Box<dyn Tool> {
    fn clone(&self) -> Self {
        self.boxed_clone()
    }
}

impl Data for Box<dyn Tool> {
    fn same(&self, other: &Box<dyn Tool>) -> bool {
        self.same_impl(other)
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
    /// Returns the number of new points added by this gesture.
    fn update_for_drag(&mut self, start: Point, current: Point) -> usize {
        // if necessary, convert the last path segment to a cubic.
        let mut result = 0;
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
            result = 2;
        }

        // if this is not the first point, adjust the previous point's second control point.
        if let Some(PathSeg::Cubic { b2, .. }) = Arc::make_mut(&mut self.segs).last_mut() {
            *b2 = start - (current - start);
        }

        if self.trailing_off_curve.is_none() {
            result += 1;
        }
        self.trailing_off_curve = Some(current);
        result
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

    fn num_points(&self) -> usize {
        let mut total = 1; // start point
        for seg in self.segs.iter() {
            total += match seg {
                PathSeg::Cubic { .. } => 3,
                PathSeg::Straight { .. } => 1,
            };
        }
        if self.closed && !self.segs.is_empty() {
            total -= 1;
        }
        if self.trailing_off_curve.is_some() {
            total += 1;
        }
        total
    }
}

// It should be able to get this from a derive macro.
impl Data for CanvasState {
    fn same(&self, other: &Self) -> bool {
        self.contents.same(&other.contents) && self.toolbar.same(&other.toolbar)
        //&& self.tool == other.tool
    }
}

impl Data for Contents {
    fn same(&self, other: &Self) -> bool {
        self.paths.same(&other.paths) && self.selection.same(&other.selection)
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
    fn paint(
        &mut self,
        paint_ctx: &mut PaintCtx,
        _base: &BaseState,
        data: &CanvasState,
        _env: &Env,
    ) {
        paint_ctx.render_ctx.clear(BG_COLOR);
        draw_paths(&data.contents.paths, &data.contents.selection, paint_ctx);
        self.toolbar
            .paint_with_offset(paint_ctx, &data.toolbar, _env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &CanvasState,
        env: &Env,
    ) -> Size {
        let toolbar_size = self.toolbar.layout(ctx, bc, &data.toolbar, env);
        self.toolbar
            .set_layout_rect(Rect::from_origin_size(TOOLBAR_POSITION, toolbar_size));
        bc.max()
    }

    fn event(
        &mut self,
        event: &Event,
        ctx: &mut EventCtx,
        data: &mut CanvasState,
        _env: &Env,
    ) -> Option<Action> {
        // first check for top-level commands
        match event {
            Event::KeyUp(key) if key.key_code == KeyCode::Escape => {
                data.remove_top_path();
                ctx.set_handled();
            }
            Event::KeyUp(key) if data.toolbar.idx_for_key(key).is_some() => {
                let idx = data.toolbar.idx_for_key(key).unwrap();
                data.toolbar.set_selected(idx);
                ctx.set_handled();
            }
            other => {
                self.toolbar.event(other, ctx, &mut data.toolbar, _env);
            }
        }

        // then pass the event to the active tool
        let CanvasState { tool, contents, .. } = data;
        if ctx.is_handled() | tool.event(contents, event) {
            ctx.invalidate();
        }

        data.update_tool_if_necessary();
        None
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old: Option<&CanvasState>,
        new: &CanvasState,
        _env: &Env,
    ) {
        // update the mouse icon if the active tool has changed
        let old = match old {
            Some(old) => old,
            None => return,
        };

        if old.toolbar.selected_idx() != new.toolbar.selected_idx() {
            match new.toolbar.selected_item().name.as_str() {
                "select" => ctx.window().set_cursor(&Cursor::Arrow),
                "pen" => ctx.window().set_cursor(&Cursor::Crosshair),
                other => eprintln!("unknown tool '{}'", other),
            }
            ctx.invalidate();
        }
        self.toolbar.update(ctx, &new.toolbar, _env);
    }
}

fn main() {
    druid_shell::init();

    let mut run_loop = runloop::RunLoop::new();
    let mut builder = WindowBuilder::new();
    let state = CanvasState::new();
    let mut state = UiState::new(Canvas::new(), state);
    state.set_active(true);
    builder.set_title("Paths");
    builder.set_handler(Box::new(UiMain::new(state)));
    let window = builder.build().unwrap();
    window.show();
    run_loop.run();
}