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

use std::any::Any;

use crate::kurbo::{Affine, BezPath, Line, Point, Rect, Shape, Size};
use crate::piet::{Color, FillRule, RenderContext};
use crate::{
    Action, BaseState, BoxConstraints, Data, Env, Event, EventCtx, KeyEvent, LayoutCtx, PaintCtx,
    UpdateCtx, Widget,
};

const BACKGROUND_COLOR: Color = Color::rgb24(0x55_55_55);
const BAR_COLOR: Color = Color::rgb24(0xf0_f0_ea);

// #[derive(Debug, Clone)]
// pub struct ProgressBarState {
    
// }

// impl Data for ProgressBarState {
//     fn same(&self, other: &Self) -> bool {
//         self.value == other.value
//     }
// }

// impl ProgressBarState {
//     pub fn new(value: f64) -> Self {
//         ProgressBarState { value }
//     }

//     pub fn set_value(&mut self, value: f64) {
//         self.value = value.max(0.0).min(1.0);
//     }
// }

#[derive(Debug, Clone, Default)]
pub struct ProgressBar {
}

impl Widget<f64> for ProgressBar {
    fn paint(
        &mut self,
        paint_ctx: &mut PaintCtx, 
        base_state: &BaseState,
        data: &f64,
        _env: &Env,
    ) {
        let clamped = data.max(0.0).min(1.0);
        let rect = base_state.layout_rect.with_origin(Point::ORIGIN);

        //Paint the background
        let brush = paint_ctx.render_ctx.solid_brush(BACKGROUND_COLOR);
        paint_ctx.render_ctx.fill(rect, &brush, FillRule::NonZero);

        //Paint the bar
        let brush = paint_ctx.render_ctx.solid_brush(BAR_COLOR);
        let calculated_bar_width = clamped * rect.width();
        let rect = rect.with_size(Size::new(calculated_bar_width, rect.height()));
        paint_ctx.render_ctx.fill(rect, &brush, FillRule::NonZero);
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
        _event: &Event,
        _ctx: &mut EventCtx,
        _data: &mut f64,
        _env: &Env,
    ) -> Option<Action> {
        None
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        _old_data: Option<&f64>,
        _data: &f64,
        _env: &Env,
    ) {
        // self.value = data.max(0.0).min(1.0);
        ctx.invalidate();
    }
}
