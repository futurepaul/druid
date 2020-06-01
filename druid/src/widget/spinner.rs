// Copyright 2020 The xi-editor Authors.
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

//! An example of an animating widget.

use std::f64::consts::PI;

use druid::kurbo::Line;
use druid::widget::prelude::*;
use druid::{theme, Color, Data, KeyOrValue, Point, Vec2};

pub struct Spinner {
    t: f64,
    color: KeyOrValue<Color>,
}

impl Spinner {
    pub fn new() -> Spinner {
        Spinner {
            t: 0.0,
            color: theme::LABEL_COLOR.into(),
        }
    }

    pub fn with_color(mut self, color: impl Into<KeyOrValue<Color>>) -> Self {
        self.color = color.into();
        self
    }

    pub fn set_color(&mut self, color: impl Into<KeyOrValue<Color>>) {
        self.color = color.into();
    }
}

impl<T: Data> Widget<T> for Spinner {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut T, _env: &Env) {}

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, _data: &T, _env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            ctx.request_anim_frame();
        }

        if let LifeCycle::AnimFrame(interval) = event {
            self.t += (*interval as f64) * 1e-9;
            if self.t >= 1.0 {
                self.t = 0.0;
            }
            ctx.request_anim_frame();
        }
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &T, _data: &T, _env: &Env) {}

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &T,
        _env: &Env,
    ) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &T, env: &Env) {
        let t = self.t;
        let (width, height) = (ctx.size().width, ctx.size().height);
        let center = Point::new(width / 2.0, height / 2.0);
        let (r, g, b, _) = Color::as_rgba(&self.color.resolve(env));
        let scale_factor = width.min(height) / 40.0;

        for step in 1..=12 {
            ctx.paint_with_z_index(1, move |ctx| {
                let step = f64::from(step);
                let fade_t = (t * 12.0 + 1.0).trunc();
                let fade = ((fade_t + step).rem_euclid(12.0) / 12.0) + 1.0 / 12.0;
                let angle = Vec2::from_angle(-(step / 12.0) * 2.0 * PI);
                let ambit_start = center + (10.0 * scale_factor * angle);
                let ambit_end = center + (20.0 * scale_factor * angle);
                let color = Color::rgba(r, g, b, fade);

                ctx.stroke(
                    Line::new(ambit_start, ambit_end),
                    &color,
                    3.0 * scale_factor,
                );
            });
        }
    }
}
