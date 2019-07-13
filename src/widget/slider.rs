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

use std::any::Any;

use crate::kurbo::{Affine, BezPath, Line, Point, Rect, Shape, Size};
use crate::piet::{Color, FillRule, RenderContext};
use crate::{
    Action, BaseState, BoxConstraints, Data, Env, Event, EventCtx, KeyEvent, LayoutCtx, PaintCtx,
    UpdateCtx, Widget,
};

const KNOB_WIDTH: f64 = 24.;
const BACKGROUND_COLOR: Color = Color::rgb24(0x55_55_55);
const SLIDER_COLOR: Color = Color::rgb24(0xf0_f0_ea);

#[derive(Debug, Clone, Default)]
pub struct Slider {
    width: f64
}

impl Widget<f64> for Slider {
    fn paint(
        &mut self,
        paint_ctx: &mut PaintCtx, 
        base_state: &BaseState,
        data: &f64,
        _env: &Env,
    ) {
        let clamped = data.max(0.0).min(1.0);
        let rect = base_state.layout_rect.with_origin(Point::ORIGIN);

        //Store the width so we can calulate slider position from mouse events
        self.width = rect.width();

        //Paint the background
        let brush = paint_ctx.render_ctx.solid_brush(BACKGROUND_COLOR);
        paint_ctx.render_ctx.fill(rect, &brush, FillRule::NonZero);

        //Paint the slider
        let brush = paint_ctx.render_ctx.solid_brush(SLIDER_COLOR);
        let slider_absolute_position = (self.width - KNOB_WIDTH) * clamped + KNOB_WIDTH / 2.;
        let full_box = KNOB_WIDTH;

        let mut position = slider_absolute_position - (KNOB_WIDTH / 2.);
        if position < 0. {
            position = 0.;
        } else if (position + KNOB_WIDTH) > self.width {
            position = self.width - KNOB_WIDTH;
        }

        let knob_origin = Point::new(rect.origin().x + position, rect.origin().y);
        let knob_size = Size::new(KNOB_WIDTH, rect.height());
        let knob_rect = Rect::from((knob_origin, knob_size));

        paint_ctx
            .render_ctx
            .fill(knob_rect, &brush, FillRule::NonZero);
    }

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &f64,
        _env: &Env
    ) -> Size {
        bc.constrain((bc.max.width, bc.max.height))
    }

    fn event(
        &mut self,
        event: &Event,
        ctx: &mut EventCtx,
        data: &mut f64,
        _env: &Env,
    ) -> Option<Action> {
        match event {
            Event::MouseDown(_) => {
                ctx.set_active(true);
            }
            Event::MouseUp(mouse) => {
                if ctx.is_active() {
                    ctx.set_active(false);
                    *data = ((mouse.pos.x - KNOB_WIDTH / 2.) / (self.width - KNOB_WIDTH))
                        .max(0.0)
                        .min(1.0);
                    ctx.invalidate();
                }
            }
            Event::MouseMoved(mouse) if mouse.count == 1 => {
                if ctx.is_active() {
                    *data = ((mouse.pos.x - KNOB_WIDTH / 2.) / (self.width - KNOB_WIDTH))
                        .max(0.0)
                        .min(1.0);
                    ctx.invalidate();
                }
            }
            _ => (),
        }
        
        None
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        _old_data: Option<&f64>,
        _data: &f64,
        _env: &Env,
    ) {
        ctx.invalidate();
    }
}
