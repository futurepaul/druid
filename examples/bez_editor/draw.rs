//! Drawing algorithms and helpers

use super::{Path, PathSeg, Pen};
use druid::kurbo::{BezPath, Circle, Line, Point};
use druid::piet::{Color, FillRule::NonZero, RenderContext};
use druid::PaintCtx;

const PATH_COLOR: Color = Color::rgb24(0x00_00_00);
const ON_CURVE_POINT_COLOR: Color = Color::rgb24(0x0b_2b_db);
const OFF_CURVE_POINT_COLOR: Color = Color::rgb24(0xbb_bb_bb);
const OFF_CURVE_HANDLE_COLOR: Color = Color::rgb24(0xbb_bb_bb);

const ON_CURVE_RADIUS: f64 = 3.5;
const ON_CURVE_SELECTED_RADIUS: f64 = 4.;
const OFF_CURVE_RADIUS: f64 = 2.;
const OFF_CURVE_SELECTED_RADIUS: f64 = 2.5;

trait PaintHelpers: RenderContext {
    fn draw_control_handle(&mut self, p1: Point, p2: Point) {
        let brush = self.solid_brush(OFF_CURVE_HANDLE_COLOR);
        let l = Line::new(p1, p2);
        self.stroke(l, &brush, 1.0, None);
    }

    fn draw_on_curve_point(&mut self, p: Point, selected: bool) {
        let radius = if selected {
            ON_CURVE_SELECTED_RADIUS
        } else {
            ON_CURVE_RADIUS
        };
        let brush = self.solid_brush(ON_CURVE_POINT_COLOR);
        let circ = Circle::new(p, radius);
        if selected {
            self.fill(circ, &brush, NonZero);
        } else {
            self.stroke(circ, &brush, 1.0, None);
        }
    }

    fn draw_off_curve_point(&mut self, p: Point, selected: bool) {
        let radius = if selected {
            OFF_CURVE_SELECTED_RADIUS
        } else {
            OFF_CURVE_RADIUS
        };
        let brush = self.solid_brush(OFF_CURVE_POINT_COLOR);
        let circ = Circle::new(p, radius);
        if selected {
            self.fill(circ, &brush, NonZero);
        } else {
            self.stroke(circ, &brush, 1.0, None);
        }
    }
}

impl<T: RenderContext> PaintHelpers for T {}

pub(crate) fn draw_inactive_path(path: &Path, paint_ctx: &mut PaintCtx) {
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

fn draw_control_point_lines(path: &Path, paint_ctx: &mut PaintCtx) {
    let mut prev_point = path.start;
    for seg in path.segs.iter() {
        if let PathSeg::Cubic { b1, b2, end } = seg {
            paint_ctx.render_ctx.draw_control_handle(prev_point, *b1);
            paint_ctx.render_ctx.draw_control_handle(*b2, *end);
        }
        prev_point = seg.end();
    }

    if let Some(trailing) = path.trailing_off_curve.as_ref() {
        paint_ctx
            .render_ctx
            .draw_control_handle(prev_point, *trailing);
    }
}

pub(crate) fn draw_paths(paths: &[Path], sels: &[usize], ctx: &mut PaintCtx) {
    let mut point_idx = 0;
    for path in paths {
        draw_inactive_path(path, ctx);
        draw_control_point_lines(path, ctx);

        ctx.render_ctx
            .draw_on_curve_point(path.start, sels.contains(&point_idx));
        point_idx += 1;
        for seg in path.segs.iter() {
            match seg {
                PathSeg::Straight { end } => {
                    ctx.render_ctx
                        .draw_on_curve_point(*end, sels.contains(&point_idx));
                    point_idx += 1;
                }
                PathSeg::Cubic { b1, b2, end } => {
                    ctx.render_ctx
                        .draw_off_curve_point(*b1, sels.contains(&point_idx));
                    point_idx += 1;
                    ctx.render_ctx
                        .draw_off_curve_point(*b2, sels.contains(&point_idx));
                    point_idx += 1;
                    ctx.render_ctx
                        .draw_on_curve_point(*end, sels.contains(&point_idx));
                    point_idx += 1;
                }
            }
        }
        if let Some(pt) = path.trailing_off_curve.as_ref() {
            ctx.render_ctx
                .draw_off_curve_point(*pt, sels.contains(&point_idx));
            point_idx += 1;
        }
    }
}
