//! Drawing algorithms and helpers

use super::{Path, PathSeg, Pen};
use druid::kurbo::{BezPath, Circle, Line};
use druid::piet::{Color, FillRule::NonZero, RenderContext};
use druid::PaintCtx;

const PATH_COLOR: Color = Color::rgb24(0x00_00_00);
const ON_CURVE_POINT_COLOR: Color = Color::rgb24(0x0b_2b_db);
const OFF_CURVE_POINT_COLOR: Color = Color::rgb24(0xbb_bb_bb);
const OFF_CURVE_HANDLE_COLOR: Color = Color::rgb24(0xbb_bb_bb);

const ON_CURVE_POINT_RADIUS: f64 = 3.5;
const OFF_CURVE_POINT_RADIUS: f64 = 2.;

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

pub(crate) fn draw_active_path(path: &Path, _tool: &Pen, paint_ctx: &mut PaintCtx) {
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
