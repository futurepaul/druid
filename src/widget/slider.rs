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

use crate::kurbo::{Circle, Point, Rect, RoundedRect, Size};
use crate::piet::{LinearGradient, RenderContext, UnitPoint};
use crate::theme;
use crate::{
    Action, BaseState, BoxConstraints, Env, Event, EventCtx, LayoutCtx, PaintCtx, UpdateCtx, Widget,
};

/// A slider, allowing interactive update of a numeric value.
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
        ((mouse_x + self.x_offset - knob_width / 2.) / (self.width - knob_width))
            .max(0.0)
            .min(1.0)
    }
}

impl Widget<f64> for Slider {
    fn paint(&mut self, paint_ctx: &mut PaintCtx, base_state: &BaseState, data: &f64, env: &Env) {
        let clamped = data.max(0.0).min(1.0);
        let rect = Rect::from_origin_size(Point::ORIGIN, base_state.size());
        let knob_size = env.get(theme::HOW_TALL_THINGS_ARE);
        let track_thickness = 4.;

        //Store the width so we can calulate slider position from mouse events
        self.width = rect.width();

        //Paint the background
        let background_width = rect.width() - knob_size;
        let background_origin = Point::new(knob_size / 2., (knob_size - track_thickness) / 2.);
        let background_size = Size::new(background_width, track_thickness);
        let background_rect =
            RoundedRect::from_origin_size(background_origin, background_size.to_vec2(), 2.);

        let background_gradient = LinearGradient::new(
            UnitPoint::TOP,
            UnitPoint::BOTTOM,
            (
                env.get(theme::BACKGROUND_LIGHT),
                env.get(theme::BACKGROUND_DARK),
            ),
        );

        paint_ctx.stroke(background_rect, &env.get(theme::BORDER), 2.0);

        paint_ctx.fill(background_rect, &background_gradient);

        //Get ready to paint the knob
        let is_active = base_state.is_active();
        let is_hovered = self.knob_hovered;

        let knob_position = (self.width - knob_size) * clamped + knob_size / 2.;
        self.knob_pos = Point::new(knob_position, knob_size / 2.);
        let knob_circle = Circle::new(self.knob_pos, knob_size / 2.);

        let normal_knob_gradient = LinearGradient::new(
            UnitPoint::TOP,
            UnitPoint::BOTTOM,
            (
                env.get(theme::FOREGROUND_LIGHT),
                env.get(theme::FOREGROUND_DARK),
            ),
        );
        let flipped_knob_gradient = LinearGradient::new(
            UnitPoint::TOP,
            UnitPoint::BOTTOM,
            (
                env.get(theme::FOREGROUND_DARK),
                env.get(theme::FOREGROUND_LIGHT),
            ),
        );

        let knob_gradient = if is_active {
            flipped_knob_gradient
        } else {
            normal_knob_gradient
        };

        //Paint the border
        let border_color = if is_hovered || is_active {
            env.get(theme::FOREGROUND_LIGHT)
        } else {
            env.get(theme::FOREGROUND_DARK)
        };

        paint_ctx.stroke(knob_circle, &border_color, 2.);

        //Actually paint the knob
        paint_ctx.fill(knob_circle, &knob_gradient);
    }

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &f64,
        env: &Env,
    ) -> Size {
        bc.check("slider");

        let default_width = 100.0;

        if bc.max().width == std::f64::INFINITY {
            return bc.constrain(Size::new(default_width, env.get(theme::TALLER_THINGS)));
        } else {
            return bc.constrain(Size::new(bc.max().width, env.get(theme::TALLER_THINGS)));
        }
    }

    fn event(
        &mut self,
        event: &Event,
        ctx: &mut EventCtx,
        data: &mut f64,
        env: &Env,
    ) -> Option<Action> {
        let knob_size = env.get(theme::HOW_TALL_THINGS_ARE);

        match event {
            Event::MouseDown(mouse) => {
                ctx.set_active(true);
                if self.knob_hit_test(knob_size, mouse.pos) {
                    self.x_offset = self.knob_pos.x - mouse.pos.x
                } else {
                    self.x_offset = 0.;
                    *data = self.calculate_value(mouse.pos.x, knob_size);
                }
                ctx.invalidate();
            }
            Event::MouseUp(mouse) => {
                if ctx.is_active() {
                    ctx.set_active(false);
                    *data = self.calculate_value(mouse.pos.x, knob_size);
                    ctx.invalidate();
                }
            }
            Event::MouseMoved(mouse) => {
                if ctx.is_active() {
                    *data = self.calculate_value(mouse.pos.x, knob_size);
                }
                if ctx.is_hot() {
                    if self.knob_hit_test(knob_size, mouse.pos) {
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
