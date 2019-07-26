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

//! A progress bar widget.

use crate::kurbo::{Point, RoundedRect, Size, Vec2};
use crate::piet::{Color, FillRule, Gradient, GradientStop, LinearGradient, RenderContext};
use crate::{
    Action, BaseState, BoxConstraints, Env, Event, EventCtx, LayoutCtx, PaintCtx, UpdateCtx, Widget,
};

const PRIMARY_LIGHT: Color = Color::rgba32(0x5c_c4_ff_ff);
const PRIMARY_DARK: Color = Color::rgba32(0x00_8d_dd_ff);

const BACKGROUND_GREY_LIGHT: Color = Color::rgba32(0x3a_3a_3a_ff);
const BACKGROUND_GREY_DARK: Color = Color::rgba32(0x31_31_31_ff);

const HEIGHT: f64 = 18.;

#[derive(Debug, Clone, Default)]
pub struct ProgressBar {}

impl Widget<f64> for ProgressBar {
    fn paint(&mut self, paint_ctx: &mut PaintCtx, base_state: &BaseState, data: &f64, _env: &Env) {
        let clamped = data.max(0.0).min(1.0);
        // let rect = Rect::from_origin_size(Point::new(0., base_state.size().height / 2. - HEIGHT / 2.), Size { width: base_state.size().width, height: HEIGHT });
        let rounded_rect = RoundedRect::from_origin_size(
            Point::new(0., base_state.size().height / 2. - HEIGHT / 2.),
            (Size {
                width: base_state.size().width,
                height: HEIGHT,
            })
            .to_vec2(),
            4.,
        );

        //Paint the background
        let gradient_brush = paint_ctx
            .render_ctx
            .gradient(Gradient::Linear(LinearGradient {
                start: rounded_rect.origin().to_vec2(),
                end: (rounded_rect.origin() + Vec2::new(0., HEIGHT)).to_vec2(),
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
        let outline_brush = paint_ctx.render_ctx.solid_brush(BACKGROUND_GREY_LIGHT);

        //TODO: paint this outside the box like in Figma
        paint_ctx
            .render_ctx
            .stroke(rounded_rect, &outline_brush, 2.0, None);
        paint_ctx
            .render_ctx
            .fill(rounded_rect, &gradient_brush, FillRule::NonZero);

        //Paint the bar
        let calculated_bar_width = clamped * rounded_rect.width();
        // let rect = rect.with_size(Size::new(calculated_bar_width, rect.height()));
        let rounded_rect = RoundedRect::from_origin_size(
            Point::new(0., base_state.size().height / 2. - HEIGHT / 2.),
            (Size {
                width: calculated_bar_width,
                height: HEIGHT,
            })
            .to_vec2(),
            4.,
        );
        let gradient_brush = paint_ctx
            .render_ctx
            .gradient(Gradient::Linear(LinearGradient {
                start: rounded_rect.origin().to_vec2(),
                end: (rounded_rect.origin() + Vec2::new(0., HEIGHT)).to_vec2(),
                stops: vec![
                    GradientStop {
                        pos: 0.0,
                        color: PRIMARY_LIGHT,
                    },
                    GradientStop {
                        pos: 1.0,
                        color: PRIMARY_DARK,
                    },
                ],
            }))
            .unwrap();

        paint_ctx
            .render_ctx
            .fill(rounded_rect, &gradient_brush, FillRule::NonZero);
    }

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &f64,
        _env: &Env,
    ) -> Size {
        bc.constrain(bc.max())
    }

    fn event(
        &mut self,
        _event: &Event,
        _ctx: &mut EventCtx,
        _data: &mut f64,
        _env: &Env,
    ) -> Option<Action> {
        None
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: Option<&f64>, _data: &f64, _env: &Env) {
        ctx.invalidate();
    }
}
