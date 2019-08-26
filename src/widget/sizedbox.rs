// Copyright 2018 The xi-editor Authors.
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

//! A box with a size (or not).

use crate::{
    Action, BaseState, BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, PaintCtx, Rect, Size,
    UpdateCtx, Widget, WidgetPod,
};

use crate::kurbo::Point;
use crate::piet::{Color, UnitPoint};

/// A widget that aligns its child.
pub struct SizedBox<T: Data> {
    width: Option<f64>,
    height: Option<f64>,
    child: WidgetPod<T, Box<dyn Widget<T>>>,
}

impl<T: Data> SizedBox<T> {
    pub fn new(
        width: Option<f64>,
        height: Option<f64>,
        child: impl Widget<T> + 'static,
    ) -> SizedBox<T> {
        SizedBox {
            width,
            height,
            child: WidgetPod::new(child).boxed(),
        }
    }
}

impl<T: Data> Widget<T> for SizedBox<T> {
    fn paint(&mut self, paint_ctx: &mut PaintCtx, base_state: &BaseState, data: &T, env: &Env) {
        // let dbg_rect = Rect::from_origin_size(Point::ORIGIN, base_state.size());
        // let dbg_color = Color::rgba8(0x00, 0xff, 0xff, 0x33);
        // paint_ctx.fill(dbg_rect, &dbg_color);
        self.child.paint_with_offset(paint_ctx, data, env);
    }

    fn layout(
        &mut self,
        layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &T,
        env: &Env,
    ) -> Size {
        bc.check("sizedbox");
        // dbg!(bc);
        // let new_bc = BoxConstraints::tight(bc.constrain(self.size));
        let size: Size;
        match (self.width, self.height) {
            (None, None) => {
                size = self.child.layout(layout_ctx, &bc, data, env);
            }
            (Some(width), Some(height)) => {
                dbg!(bc);
                let loose_bc = bc.loosen();
                let new_bc = BoxConstraints::tight(loose_bc.constrain(Size::new(width, height)));
                size = self.child.layout(layout_ctx, &new_bc, data, env);
                dbg!(size);
            }
            (None, Some(height)) => {
                let new_bc = BoxConstraints::new(
                    bc.constrain(Size::new(bc.min().width, height)),
                    bc.constrain(Size::new(bc.max().width, height)),
                );
                size = self.child.layout(layout_ctx, &new_bc, data, env);
            }
            (Some(width), None) => {
                let new_bc = BoxConstraints::new(
                    bc.constrain(Size::new(width, bc.min().height)),
                    bc.constrain(Size::new(width, bc.max().height)),
                );
                new_bc.check("uh oh");
                size = self.child.layout(layout_ctx, &new_bc, data, env);
            }
        }

        let mut my_size = size;
        // if bc.is_width_bounded() {
        //     my_size.width = bc.max().width;
        // }
        // if bc.is_height_bounded() {
        //     my_size.height = bc.max().height;
        // }
        // my_size = bc.constrain(my_size);

        self.child
            .set_layout_rect(Rect::from_origin_size(Point::ORIGIN, size));
        my_size
    }

    fn event(
        &mut self,
        event: &Event,
        ctx: &mut EventCtx,
        data: &mut T,
        env: &Env,
    ) -> Option<Action> {
        self.child.event(event, ctx, data, env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: Option<&T>, data: &T, env: &Env) {
        self.child.update(ctx, data, env);
    }
}
