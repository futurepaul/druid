// Copyright 2019 The xi-editor Authors.
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

//! A checkbox widget.

use crate::kurbo::{Circle, Point, Rect, RoundedRect, Size, Vec2, BezPath};
use crate::piet::{Color, FillRule, Gradient, GradientStop, LinearGradient, RenderContext};
use crate::{
    Action, BaseState, BoxConstraints, Env, Event, EventCtx, LayoutCtx, PaintCtx, UpdateCtx, Widget,
};

const BACKGROUND_GREY_LIGHT: Color = Color::rgba32(0x3a_3a_3a_ff);
const BACKGROUND_GREY_DARK: Color = Color::rgba32(0x31_31_31_ff);

const ALMOST_WHITE: Color = Color::rgba32(0xf9_f9_f9_ff);
const BORDER_LIGHT: Color = Color::rgba32(0xa1_a1_a1_ff);

const SIZE: f64 = 18.;

#[derive(Debug, Clone, Default)]
pub struct CheckBox {
}

impl Widget<bool> for CheckBox {
    fn paint(&mut self, paint_ctx: &mut PaintCtx, base_state: &BaseState, data: &bool, _env: &Env) {
        let rect = RoundedRect::from_origin_size(Point::ORIGIN, Size::new(SIZE, SIZE).to_vec2(), 2.);

        //Paint the background
        let gradient_brush = paint_ctx
            .render_ctx
            .gradient(Gradient::Linear(LinearGradient {
                start: rect.origin().to_vec2(),
                end: (rect.origin() + Vec2::new(0., SIZE)).to_vec2(),
                stops: vec![
                    GradientStop {
                        pos: 0.0,
                        color: BACKGROUND_GREY_LIGHT,
                    },
                    GradientStop {
                        pos: 1.0,
                        color: BACKGROUND_GREY_DARK,
                    },
                ],
            }))
            .unwrap();

        paint_ctx
            .render_ctx
            .fill(rect, &gradient_brush, FillRule::NonZero);

        let border_color = if base_state.is_hot() { BORDER_LIGHT } else { BACKGROUND_GREY_LIGHT };

        let border_brush = paint_ctx.render_ctx.solid_brush(border_color);

        paint_ctx
            .render_ctx
            .stroke(rect, &border_brush, 1., None);

        if *data {
            let mut path = BezPath::new();
            path.move_to((3.0, 9.0));
            path.line_to((7.0, 13.0));
            path.line_to((13.0, 5.0));

            let check_brush = paint_ctx.render_ctx.solid_brush(ALMOST_WHITE);

            paint_ctx
                .render_ctx
                .stroke(path, &check_brush, 2., None);
        }
    }

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &bool,
        _env: &Env,
    ) -> Size {
        bc.constrain(bc.max())
    }

    fn event(
        &mut self,
        event: &Event,
        ctx: &mut EventCtx,
        data: &mut bool,
        _env: &Env,
    ) -> Option<Action> {
        match event {
            Event::MouseDown(_) => {
                ctx.set_active(true);
                ctx.invalidate();
            }
            Event::MouseUp(_) => {
                if ctx.is_active() {
                    ctx.set_active(false);
                    ctx.invalidate();
                    if ctx.is_hot() {
                        if *data {
                            *data = false;
                        } else {
                            *data = true;
                        }
                    }
                }
            }
            Event::MouseMoved(_) => {
                ctx.invalidate();
            }
            _ => (),
        }
        None
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: Option<&bool>, data: &bool, _env: &Env) {
        ctx.invalidate();
    }
}
