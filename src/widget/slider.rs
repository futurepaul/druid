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

//! A slider widget.

use crate::kurbo::{Circle, Point, Rect, RoundedRect, Size, Vec2};
use crate::piet::{Color, FillRule, Gradient, GradientStop, LinearGradient, RenderContext};
use crate::{
    Action, BaseState, BoxConstraints, Env, Event, EventCtx, LayoutCtx, PaintCtx, UpdateCtx, Widget,
};

const BACKGROUND_GREY_LIGHT: Color = Color::rgba32(0x3a_3a_3a_ff);
const BACKGROUND_GREY_DARK: Color = Color::rgba32(0x31_31_31_ff);

const KNOB_LIGHT: Color = Color::rgba32(0xf9_f9_f9_ff);
const KNOB_DARK: Color = Color::rgba32(0xbf_bf_bf_ff);

const KNOB_WIDTH: f64 = 18.;
const BACKGROUND_THICKNESS: f64 = 4.;

#[derive(Debug, Clone, Default)]
pub struct Slider {
    width: f64,
    knob_pos: Point,
    knob_hovered: bool,
    x_offset: f64,
}

impl Slider {
    fn knob_hit_test(&self, knob_width: f64, mouse_pos: Point) -> bool {
        let knob_circle = Circle::new(self.knob_pos, knob_width / 2.);
        if mouse_pos.distance(knob_circle.center) < knob_circle.radius {
            return true;
        }
        false
    }

    fn calculate_value(&self, mouse_x: f64, knob_width: f64) -> f64 {
        ((mouse_x + self.x_offset - KNOB_WIDTH / 2.) / (self.width - knob_width))
            .max(0.0)
            .min(1.0)
    }
}

impl Widget<f64> for Slider {
    fn paint(&mut self, paint_ctx: &mut PaintCtx, base_state: &BaseState, data: &f64, _env: &Env) {
        let clamped = data.max(0.0).min(1.0);
        let rect = Rect::from_origin_size(Point::ORIGIN, base_state.size());

        //Store the width so we can calulate slider position from mouse events
        self.width = rect.width();

        //Paint the background
        let background_width = rect.width() - KNOB_WIDTH;
        let background_origin =
            Point::new(KNOB_WIDTH / 2., (KNOB_WIDTH - BACKGROUND_THICKNESS) / 2.);
        let background_size = Size::new(background_width, BACKGROUND_THICKNESS);
        let background_rect =
            RoundedRect::from_origin_size(background_origin, background_size.to_vec2(), 2.);

        let gradient_brush = paint_ctx
            .render_ctx
            .gradient(Gradient::Linear(LinearGradient {
                start: background_origin.to_vec2(),
                end: (background_rect.origin() + Vec2::new(0., BACKGROUND_THICKNESS)).to_vec2(),
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
            .fill(background_rect, &gradient_brush, FillRule::NonZero);

        let border_brush = paint_ctx.render_ctx.solid_brush(BACKGROUND_GREY_LIGHT);

        paint_ctx
            .render_ctx
            .stroke(background_rect, &border_brush, 1., None);

        //Paint the slider
        let is_active = base_state.is_active();
        let is_hovered = self.knob_hovered;

        let knob_position = (self.width - KNOB_WIDTH) * clamped + KNOB_WIDTH / 2.;
        self.knob_pos = Point::new(knob_position, KNOB_WIDTH / 2.);
        let knob_circle = Circle::new(self.knob_pos, KNOB_WIDTH / 2.);

        let normal_knob_gradient = Gradient::Linear(LinearGradient {
            start: self.knob_pos.to_vec2() - Vec2::new(0., KNOB_WIDTH / 2.),
            end: self.knob_pos.to_vec2() + Vec2::new(0., KNOB_WIDTH / 2.),
            stops: vec![
                GradientStop {
                    pos: 0.0,
                    color: KNOB_LIGHT,
                },
                GradientStop {
                    pos: 1.0,
                    color: KNOB_DARK,
                },
            ],
        });

        let flipped_knob_gradient = Gradient::Linear(LinearGradient {
            start: self.knob_pos.to_vec2() - Vec2::new(0., KNOB_WIDTH / 2.),
            end: self.knob_pos.to_vec2() + Vec2::new(0., KNOB_WIDTH / 2.),
            stops: vec![
                GradientStop {
                    pos: 0.0,
                    color: KNOB_DARK,
                },
                GradientStop {
                    pos: 1.0,
                    color: KNOB_LIGHT,
                },
            ],
        });

        let knob_color = if is_active {
            flipped_knob_gradient
        } else {
            normal_knob_gradient
        };

        let knob_brush = paint_ctx.render_ctx.gradient(knob_color).unwrap();

        paint_ctx
            .render_ctx
            .fill(knob_circle, &knob_brush, FillRule::NonZero);

        let border_color = if is_hovered || is_active {
            KNOB_LIGHT
        } else {
            KNOB_DARK
        };

        let border_brush = paint_ctx.render_ctx.solid_brush(border_color);

        paint_ctx
            .render_ctx
            .stroke(knob_circle, &border_brush, 1., None);
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
        event: &Event,
        ctx: &mut EventCtx,
        data: &mut f64,
        _env: &Env,
    ) -> Option<Action> {
        match event {
            Event::MouseDown(mouse) => {
                ctx.set_active(true);
                if self.knob_hit_test(KNOB_WIDTH, mouse.pos) {
                    self.x_offset = self.knob_pos.x - mouse.pos.x
                } else {
                    self.x_offset = 0.;
                    *data = self.calculate_value(mouse.pos.x, KNOB_WIDTH);
                }
                ctx.invalidate();
            }
            Event::MouseUp(mouse) => {
                if ctx.is_active() {
                    ctx.set_active(false);
                    *data = self.calculate_value(mouse.pos.x, KNOB_WIDTH);
                    ctx.invalidate();
                }
            }
            Event::MouseMoved(mouse) => {
                if ctx.is_active() {
                    *data = self.calculate_value(mouse.pos.x, KNOB_WIDTH);
                }
                if ctx.is_hot() {
                    if self.knob_hit_test(KNOB_WIDTH, mouse.pos) {
                        self.knob_hovered = true
                    } else {
                        self.knob_hovered = false
                    }
                }
                ctx.invalidate();
            }
            _ => (),
        }
        None
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: Option<&f64>, _data: &f64, _env: &Env) {
        ctx.invalidate();
    }
}
