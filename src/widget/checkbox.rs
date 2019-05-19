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

use crate::widget::Widget;
use crate::{
    BoxConstraints, Geometry, HandlerCtx, Id, LayoutCtx, LayoutResult, MouseEvent, PaintCtx, Ui,
};

use kurbo::{Line, Rect, BezPath};
use piet::{FillRule, RenderContext};

const BOX_HEIGHT: f64 = 19.;

pub struct Checkbox {
    value: bool
}

impl Checkbox {
    pub fn new(value: bool) -> Checkbox {
        Checkbox {
          value 
        }
    }
    pub fn ui(self, ctx: &mut Ui) -> Id {
        ctx.add(self, &[])
    }
}

impl Widget for Checkbox {
    fn paint(&mut self, paint_ctx: &mut PaintCtx, geom: &Geometry) {

        let background_color = 0x55_55_55_ff;
        let foreground_color = 0xf0f0eaff;

        //Paint the background
        let brush = paint_ctx.render_ctx.solid_brush(background_color).unwrap();

        let (x, y) = geom.pos;
        let (x, y) = (x as f64, y as f64);
        let (width, height) = geom.size;
        let rect = Rect::new(
            x,
            y,
            x + BOX_HEIGHT,
            y + BOX_HEIGHT,
        );

        paint_ctx.render_ctx.fill(rect, &brush, FillRule::NonZero);

        //Paint the check 
        let brush = paint_ctx.render_ctx.solid_brush(foreground_color).unwrap();

        let mut path = BezPath::new();
        path.moveto((x + 3.0, y + 9.0));
        path.lineto((x + 8.0, y + 14.0));
        path.lineto((x + 15.0, y + 4.0));

        paint_ctx.render_ctx.stroke(path, &brush, 2.0, None);

        //Cover the check if false
        if (self.value == false) {
            let brush = paint_ctx.render_ctx.solid_brush(background_color).unwrap();

            let rect = Rect::new(
                x + 2.,
                y + 2.,
                x + BOX_HEIGHT - 2.,
                y + BOX_HEIGHT - 2.,
            );

            paint_ctx.render_ctx.fill(rect, &brush, FillRule::NonZero);
        }
    }

    fn layout(
        &mut self,
        bc: &BoxConstraints,
        _children: &[Id],
        _size: Option<(f32, f32)>,
        _ctx: &mut LayoutCtx,
    ) -> LayoutResult {
        LayoutResult::Size(bc.constrain((BOX_HEIGHT as f32, BOX_HEIGHT as f32)))
    }

    fn mouse(&mut self, event: &MouseEvent, ctx: &mut HandlerCtx) -> bool {
        if event.count > 0 {
            ctx.set_active(true);
            self.value = !self.value;
            ctx.send_event(self.value);
        } else {
            ctx.set_active(false);
        }
        ctx.invalidate();
        true
    }

    fn poke(&mut self, payload: &mut Any, ctx: &mut HandlerCtx) -> bool {
        if let Some(value) = payload.downcast_ref::<bool>() {
            self.value = *value;
            ctx.invalidate();
            true
        } else {
            println!("downcast failed");
            false
        }
    }
}
